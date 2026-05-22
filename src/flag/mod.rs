pub(crate) mod config;
pub(crate) mod rule;
pub mod stats;
pub(crate) mod transform;

use std::path::PathBuf;

use crate::column::ColumnRef;
use crate::error::CsvOpsError;
use crate::io::{resolve_encoding, resolve_input_encoding};
use crate::pipeline::{PipelineOptions, run_pipeline};

use config::FlagConfig;
use stats::FlagStats;
use transform::FlagTransform;

/// ルールの指定方法
/// Config 指定と CLI 引数指定
pub enum RuleSource {
    /// TOML 設定ファイルのパス
    Config(PathBuf),
    /// CLI 引数による 1 ルール (true/false 値はデフォルト固定)
    Inline {
        pattern: String,
        columns: Vec<ColumnRef>,
        out_col: String,
    },
}

/// flag::run に渡す設定一式
/// CLI 引数 / Config ファイルどちらの経路でも、最終的にこの形に集約してから run を呼ぶ
pub struct FlagRequest {
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
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// flag サブコマンドのエントリポイント
pub fn run(request: FlagRequest) -> Result<FlagStats, CsvOpsError> {
    let FlagRequest {
        rules,
        input,
        output,
        input_encoding,
        output_encoding,
        delimiter,
        has_headers,
        dry_run,
    } = request;

    // ルール指定を FlagConfig に統一する
    let cfg = match rules {
        RuleSource::Config(path) => {
            let text = std::fs::read_to_string(&path)?;
            FlagConfig::from_toml(&text)?
        }
        RuleSource::Inline {
            pattern,
            columns,
            out_col,
        } => FlagConfig::from_single_rule(pattern, columns, out_col),
    };

    // 入力エンコーディングは auto 指定ならファイル先頭から推定する
    let input_encoding = resolve_input_encoding(&input_encoding, &input)?;
    let output_encoding = resolve_encoding(&output_encoding)?;

    // ルールの compile・列解決・統計集計は FlagTransform が担う
    let mut transform = FlagTransform::new(cfg);
    let opts = PipelineOptions {
        input,
        output,
        input_encoding,
        output_encoding,
        // flag は列を追加するだけで区切り文字は変えない
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
