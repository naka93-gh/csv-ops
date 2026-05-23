pub(crate) mod collision;
pub(crate) mod config;
pub(crate) mod rule;
pub(crate) mod transform;

use std::path::PathBuf;

use crate::column::ColumnRef;
use crate::error::CsvOpsError;
use crate::io::{resolve_encoding, resolve_input_encoding};
use crate::pipeline::{PipelineOptions, run_pipeline};
use crate::stats::Stats;

use collision::detect_static_collisions;
use config::ReplaceConfig;
use transform::ReplaceTransform;

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
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp / auto)
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
pub fn run(request: ReplaceRequest) -> Result<Stats, CsvOpsError> {
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
    let rule_ids: Vec<String> = compiled.iter().map(|r| r.id().to_string()).collect();

    // 入力エンコーディングは auto 指定ならファイル先頭から推定する
    let input_encoding = resolve_input_encoding(&input_encoding, &input)?;
    let output_encoding = resolve_encoding(&output_encoding)?;

    // 置換処理と統計集計は ReplaceTransform が担い、パイプラインが I/O を担う
    let mut transform = ReplaceTransform::new(compiled, columns, Stats::with_rule_ids(rule_ids));
    let opts = PipelineOptions {
        input,
        output,
        input_encoding,
        output_encoding,
        // replace は内容のみ変換するので入出力で区切り文字は同一
        input_delimiter: delimiter,
        output_delimiter: delimiter,
        has_headers,
        dry_run,
    };

    let rows = run_pipeline(&mut transform, &opts)?;
    let mut stats = transform.stats;
    stats.rows_processed = rows;
    Ok(stats)
}
