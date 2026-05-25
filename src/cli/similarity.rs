use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::similarity::{RuleSource, SimilarityRequest};

use super::{emit_report, parse_delimiter_alias};

/// `csv-ops similarity` の引数
#[derive(Args, Debug)]
pub(crate) struct SimilarityArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

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
    #[arg(long, default_value_t = crate::similarity::DEFAULT_THRESHOLD)]
    pub threshold: f64,

    /// 類似度アルゴリズム (levenshtein / damerau / jaro-winkler / dice)
    #[arg(long, value_name = "NAME", default_value = "levenshtein")]
    pub algorithm: String,

    /// 正規化オプション (カンマ区切り: nfkc / fullwidth / prolonged / kana / casefold / whitespace)
    #[arg(long, value_name = "LIST", default_value = "nfkc,casefold,whitespace")]
    pub normalize: String,

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp / auto)
    #[arg(long, default_value = "utf-8")]
    pub input_encoding: String,

    /// 出力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub output_encoding: String,

    /// 区切り文字 (comma / tab / pipe / semicolon)。CSV 形式の辞書にも適用される
    #[arg(long, value_name = "ALIAS", default_value = "comma")]
    pub delimiter: String,

    /// ヘッダ行なし CSV
    #[arg(long)]
    pub no_headers: bool,

    /// 出力ファイルへ書き込まず、統計のみ表示する
    #[arg(long)]
    pub dry_run: bool,

    /// 統計の出力形式 (text / json)
    #[arg(long, value_name = "FORMAT", default_value = "text")]
    pub stats_format: String,

    /// 統計の出力先ファイル (未指定なら標準出力)
    #[arg(long, value_name = "PATH")]
    pub stats_file: Option<PathBuf>,
}

/// similarity サブコマンドのエントリポイント
pub(crate) fn run(args: SimilarityArgs) -> Result<ExitCode, Box<dyn Error>> {
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

    let delimiter = parse_delimiter_alias(&args.delimiter)?;

    let request = SimilarityRequest {
        rules,
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        delimiter,
        has_headers: !args.no_headers,
        dry_run: args.dry_run,
    };

    let stats = crate::similarity::run(request)?;
    emit_report(&stats, &args.stats_format, args.stats_file.as_deref())?;
    Ok(ExitCode::SUCCESS)
}
