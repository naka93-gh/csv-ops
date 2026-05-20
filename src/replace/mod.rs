#![allow(dead_code)]

pub(crate) mod collision;
pub(crate) mod config;
pub(crate) mod rule;
pub mod stats;
pub(crate) mod transform;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::column::{ColumnRef, resolve_indices};
use crate::error::{CsvOpsError, EncodingError};
use crate::io::resolve_encoding;

use collision::detect_static_collisions;
use config::ReplaceConfig;
use stats::ReplaceStats;
use transform::{ReplaceTransform, TargetColumns};

/// ルールの指定方法
/// Config 指定と 引数指定
pub enum RuleSource {
    /// TOML 設定ファイルのパス
    Config(PathBuf),
    /// CLI 引数による 1 ルール (--regex 時は from/to を pattern/replacement として扱う)
    Inline {
        from: String,
        to: String,
        regex: bool,
    },
}

/// 置換対象の列指定
pub enum ColumnTarget {
    /// 全カラム横断 (--all-columns)
    All,
    /// 指定列のみ (-c)
    Specified(Vec<ColumnRef>),
}

/// replace::run に渡す設定一式
/// CLI 引数 / Config ファイルどちらの経路でも、最終的にこの形に集約してから run を呼ぶ
pub struct ReplaceRequest {
    /// ルール指定 (Config ファイル or CLI 引数)
    pub rules: RuleSource,
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp)
    pub input_encoding: String,
    /// 出力エンコーディング名
    pub output_encoding: String,
    /// 区切り文字
    pub delimiter: u8,
    /// ヘッダー行の有無
    pub has_headers: bool,
    /// 大文字小文字を区別しないか (CLI 引数モードで使用、Config モードでは config 側の値が優先)
    pub case_insensitive: bool,
    /// 置換対象の列 (All = 全カラム横断 / Specified = 指定列のみ)
    pub columns: ColumnTarget,
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// replace サブコマンドのエントリポイント
pub fn run(request: ReplaceRequest) -> Result<ReplaceStats, CsvOpsError> {
    let ReplaceRequest {
        rules,
        input,
        output,
        input_encoding,
        output_encoding,
        delimiter,
        has_headers,
        case_insensitive,
        columns,
        dry_run,
    } = request;

    // ルール指定を ReplaceConfig に統一してから compile する
    let cfg = match rules {
        RuleSource::Config(path) => {
            let text = std::fs::read_to_string(&path)?;
            ReplaceConfig::from_toml(&text)?
        }
        RuleSource::Inline { from, to, regex } => {
            ReplaceConfig::from_single_rule(from, to, regex, case_insensitive)
        }
    };
    let compiled = cfg.compile_rules()?;
    // Config ロード時の静的衝突検出 (単純置換ルール間の部分文字列関係 / 完全重複)
    detect_static_collisions(&compiled, cfg.case_insensitive())?;
    // per_rule 統計の初期化用にルール ID を集める (compiled が move される前に)
    let rule_ids: Vec<String> = compiled
        .iter()
        .map(|r: &rule::CompiledRule| r.id().to_string())
        .collect();
    let transform = ReplaceTransform::new(compiled, cfg.case_insensitive());

    // 入力エンコーディングを解決し、DecodeReaderBytes で UTF-8 にデコードしながら読む
    let in_enc = resolve_encoding(&input_encoding)?;
    let file = File::open(&input)?;
    let decoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(in_enc))
        .build(file);
    let reader = BufReader::new(decoded);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_headers)
        .from_reader(reader);

    // 置換結果は一旦 UTF-8 で Vec<u8> に書き出し、後で出力エンコーディングへ変換する
    let mut utf8_buf = Vec::new();
    let mut stats = ReplaceStats::new(rule_ids);
    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(&mut utf8_buf);

        // ヘッダー行は置換せずそのまま出力し、列名解決のため保持
        let headers: Option<csv::StringRecord> = if has_headers {
            let h = rdr.headers()?.clone();
            wtr.write_record(&h)?;
            Some(h)
        } else {
            None
        };

        // 列指定を解決済みインデックスに変換 (ヘッダーと照合)
        let target = match &columns {
            ColumnTarget::All => TargetColumns::All,
            ColumnTarget::Specified(cols) => {
                TargetColumns::Indices(resolve_indices(cols, headers.as_ref())?)
            }
        };

        // 各データ行に置換を適用
        // row は 1-indexed (RuntimeCollision の行番号表示に使う)
        let mut row: u64 = 0;
        for result in rdr.records() {
            row += 1;
            let mut record = result?;
            stats.rows_processed += 1;

            // apply_record はこの行でマッチしたルール index 列を返す (重複あり = マッチ回数分)
            let row_matches =
                transform.apply_record(&mut record, row, headers.as_ref(), &target)?;
            if !row_matches.is_empty() {
                stats.rows_modified += 1;
            }
            stats.total_replacements += row_matches.len() as u64;

            // per_rule 集計:
            // - matches      = マッチ回数なので、各マッチごとにカウント
            // - rows_affected = 影響行数なので、この行で 1 回以上マッチしたルールを 1 回だけカウント
            let mut affected: HashSet<usize> = HashSet::new();
            for &idx in &row_matches {
                stats.per_rule[idx].matches += 1;
                affected.insert(idx);
            }
            for idx in affected {
                stats.per_rule[idx].rows_affected += 1;
            }

            wtr.write_record(&record)?;
        }
        wtr.flush()?;
    }

    // dry-run でなければ、UTF-8 結果を出力エンコーディングに変換してファイルへ書き出す
    // dry-run 時は置換処理・統計集計だけ行い、出力は一切しない
    if !dry_run {
        // utf8_buf は csv writer が書いたものなので必ず valid UTF-8 (from_utf8 は失敗しない)
        let out_enc = resolve_encoding(&output_encoding)?;
        let utf8_str = std::str::from_utf8(&utf8_buf).expect("csv writer の出力は常に valid UTF-8");
        let (encoded, _, had_errors) = out_enc.encode(utf8_str);
        if had_errors {
            return Err(EncodingError::EncodeFailure {
                encoding: out_enc.name().to_string(),
            }
            .into());
        }

        let mut out_file = File::create(&output)?;
        out_file.write_all(&encoded)?;
        out_file.flush()?;
    }

    Ok(stats)
}
