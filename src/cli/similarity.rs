use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::commands::similarity::{RuleSource, SimilarityRequest};

use super::{CommonIoArgs, StatsOutputArgs, emit_report, parse_delimiter_alias};

/// `csv-ops similarity` の引数
/// delimiter は通常の CSV 区切り文字に加え、`--dict` に CSV 形式を渡した場合の辞書側にも適用される
#[derive(Args, Debug)]
pub struct SimilarityArgs {
    #[command(flatten)]
    pub io: CommonIoArgs,

    /// 設定ファイル (TOML)。指定時は -c / --dict 等は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (列名または列番号、1 列。CLI 引数モードで必須)
    #[arg(short = 'c', long, value_name = "COLUMN")]
    pub column: Option<String>,

    /// 辞書ファイル (.toml は TOML、それ以外は CSV。CLI 引数モードで必須)
    #[arg(long, value_name = "FILE")]
    pub dict: Option<PathBuf>,

    /// マッチ名を出力する列名 (CLI 引数モード)
    #[arg(long, value_name = "NAME", default_value = "matched_name")]
    pub out_col: String,

    /// スコアを出力する列名 (CLI 引数モード)
    #[arg(long, value_name = "NAME", default_value = "score")]
    pub score_col: String,

    /// マッチとみなすしきい値 (0.0-1.0)
    #[arg(long, default_value_t = crate::commands::similarity::DEFAULT_THRESHOLD)]
    pub threshold: f64,

    /// 類似度アルゴリズム (levenshtein / damerau / jaro-winkler / dice)
    #[arg(long, value_name = "NAME", default_value = "levenshtein")]
    pub algorithm: String,

    /// 正規化オプション (カンマ区切り: nfkc / fullwidth / prolonged / kana / casefold / whitespace)
    #[arg(long, value_name = "LIST", default_value = "nfkc,casefold,whitespace")]
    pub normalize: String,

    #[command(flatten)]
    pub stats: StatsOutputArgs,
}

/// similarity サブコマンドのエントリポイント
pub fn run(args: SimilarityArgs) -> Result<ExitCode, Box<dyn Error>> {
    // ルール指定の解決。--config が優先、なければ -c / --dict の CLI 引数モード
    let rules = match args.config {
        Some(path) => RuleSource::Config(path),
        None => {
            let column = args
                .column
                .ok_or("--config か -c <列> / --dict <FILE> のいずれかを指定してください")?;
            let dict = args.dict.ok_or("--dict <FILE> が必要です")?;
            // カンマ区切りの正規化指定を名前リストへ分解する
            let normalize: Vec<String> = args
                .normalize
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            RuleSource::Inline {
                column: ColumnRef::parse(column.trim()),
                dict,
                out_col: args.out_col,
                score_col: args.score_col,
                threshold: args.threshold,
                normalize,
                algorithm: args.algorithm,
            }
        }
    };

    let delimiter = parse_delimiter_alias(&args.io.delimiter)?;

    let request = SimilarityRequest {
        rules,
        input: args.io.input,
        output: args.io.output,
        input_encoding: args.io.input_encoding,
        delimiter,
        has_headers: !args.io.no_headers,
        dry_run: args.io.dry_run,
    };

    let stats = crate::commands::similarity::run(request)?;
    emit_report(&stats, &args.stats.stats_format)?;
    Ok(ExitCode::SUCCESS)
}
