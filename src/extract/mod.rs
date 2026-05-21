pub(crate) mod config;
pub(crate) mod rule;
pub mod stats;
pub(crate) mod transform;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::column::ColumnRef;
use crate::error::{CsvOpsError, EncodingError, TransformError};
use crate::io::resolve_encoding;

use config::ExtractConfig;
use stats::ExtractStats;
use transform::ExtractTransform;

/// ルールの指定方法
/// Config 指定と CLI 引数指定
pub enum RuleSource {
    /// TOML 設定ファイルのパス
    Config(PathBuf),
    /// CLI 引数による 1 ルール (separator 未指定ならデフォルト固定)
    Inline {
        pattern: String,
        column: ColumnRef,
        out_col: String,
        /// 複数マッチの区切り文字 (None ならデフォルト)
        separator: Option<String>,
    },
}

/// extract::run に渡す設定一式
/// CLI 引数 / Config ファイルどちらの経路でも、最終的にこの形に集約してから run を呼ぶ
pub struct ExtractRequest {
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
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// extract サブコマンドのエントリポイント
pub fn run(request: ExtractRequest) -> Result<ExtractStats, CsvOpsError> {
    let ExtractRequest {
        rules,
        input,
        output,
        input_encoding,
        output_encoding,
        delimiter,
        has_headers,
        dry_run,
    } = request;

    // ルール指定を ExtractConfig に統一する
    let cfg = match rules {
        RuleSource::Config(path) => {
            let text = std::fs::read_to_string(&path)?;
            ExtractConfig::from_toml(&text)?
        }
        RuleSource::Inline {
            pattern,
            column,
            out_col,
            separator,
        } => ExtractConfig::from_single_rule(pattern, column, out_col, separator),
    };

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

    // ヘッダー行を保持する (列名解決と out_col 衝突検査に使う)
    let headers: Option<csv::StringRecord> = if has_headers {
        Some(rdr.headers()?.clone())
    } else {
        None
    };

    // ルールを compile (正規表現 compile + 対象列をヘッダ照合でインデックス解決)
    let compiled = cfg.compile_rules(headers.as_ref())?;

    // out_col 衝突検査 (ヘッダ有り時のみ)
    // 既存カラム名との衝突と、ルール間の out_col 重複を 1 つの集合でまとめて検出する
    // insert は未登録なら true を返すので、false = 既出 = 衝突
    if let Some(h) = headers.as_ref() {
        let mut seen: HashSet<String> = h.iter().map(|s| s.to_string()).collect();
        for rule in &compiled {
            if !seen.insert(rule.out_col.clone()) {
                return Err(TransformError::OutputColumnConflict {
                    name: rule.out_col.clone(),
                }
                .into());
            }
        }
    }

    // 統計初期化用に out_col 一覧をクローンして残す
    let out_cols: Vec<String> = compiled.iter().map(|r| r.out_col.clone()).collect();
    let transform = ExtractTransform::new(compiled);

    // 変換結果は一旦 UTF-8 で Vec<u8> に書き出し、後で出力エンコーディングへ変換する
    let mut utf8_buf = Vec::new();
    let mut stats = ExtractStats::new(out_cols);
    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(&mut utf8_buf);

        // ヘッダー行は既存カラム + 各ルールの out_col を末尾に追加して出力する
        if let Some(h) = &headers {
            let mut out_header: Vec<String> = h.iter().map(|s| s.to_string()).collect();
            for r in &stats.per_rule {
                out_header.push(r.out_col.clone());
            }
            wtr.write_record(&out_header)?;
        }

        // 各データ行に抽出処理を適用
        for result in rdr.records() {
            let mut record = result?;
            stats.rows_processed += 1;

            // apply_record はルール毎の抽出列を末尾に追加し、抽出有無の bool 列を返す
            let extracted = transform.apply_record(&mut record)?;
            for (i, &found) in extracted.iter().enumerate() {
                if found {
                    stats.per_rule[i].extracted_rows += 1;
                }
            }

            wtr.write_record(&record)?;
        }
        wtr.flush()?;
    }

    // dry-run でなければ、UTF-8 結果を出力エンコーディングに変換してファイルへ書き出す
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
