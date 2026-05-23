pub(crate) mod config;
pub(crate) mod dict;
pub(crate) mod rule;
pub mod stats;
pub(crate) mod transform;

use std::path::PathBuf;

use crate::column::ColumnRef;
use crate::error::CsvOpsError;
use crate::io::{resolve_encoding, resolve_input_encoding};
use crate::pipeline::{PipelineOptions, run_pipeline};

use config::SimilarityConfig;
use stats::SimilarityStats;
use transform::SimilarityTransform;

pub use config::DEFAULT_THRESHOLD;

/// ルールの指定方法
/// Config 指定と CLI 引数指定
pub enum RuleSource {
    /// TOML 設定ファイルのパス
    Config(PathBuf),
    /// CLI 引数による 1 ルール
    Inline {
        column: ColumnRef,
        dict: PathBuf,
        out_col: String,
        score_col: String,
        threshold: f64,
        /// 正規化オプション名のリスト
        normalize: Vec<String>,
        /// 類似度アルゴリズム名
        algorithm: String,
    },
}

/// similarity::run に渡す設定一式
/// CLI 引数 / Config ファイルどちらの経路でも、最終的にこの形に集約してから run を呼ぶ
pub struct SimilarityRequest {
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
    /// 区切り文字 (本体 CSV と CSV 形式辞書に共通)
    pub delimiter: u8,
    /// ヘッダー行の有無
    pub has_headers: bool,
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// similarity サブコマンドのエントリポイント
pub fn run(request: SimilarityRequest) -> Result<SimilarityStats, CsvOpsError> {
    let SimilarityRequest {
        rules,
        input,
        output,
        input_encoding,
        output_encoding,
        delimiter,
        has_headers,
        dry_run,
    } = request;

    // ルール指定を SimilarityConfig に統一する
    let cfg = match rules {
        RuleSource::Config(path) => {
            let text = std::fs::read_to_string(&path)?;
            SimilarityConfig::from_toml(&text)?
        }
        RuleSource::Inline {
            column,
            dict,
            out_col,
            score_col,
            threshold,
            normalize,
            algorithm,
        } => SimilarityConfig::from_single_rule(
            column, dict, out_col, score_col, threshold, normalize, algorithm,
        ),
    };

    // 入力エンコーディングは auto 指定ならファイル先頭から推定する
    let input_encoding = resolve_input_encoding(&input_encoding, &input)?;
    let output_encoding = resolve_encoding(&output_encoding)?;

    // 列解決・辞書ロード・統計集計は SimilarityTransform が担う
    let mut transform = SimilarityTransform::new(cfg, delimiter);
    let opts = PipelineOptions {
        input,
        output,
        input_encoding,
        output_encoding,
        // similarity は列を追加するだけで区切り文字は変えない
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
