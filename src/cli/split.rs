// split サブコマンドの CLI 引数定義と実行ハンドラ
// config ファイルは持たず CLI 引数のみで完結する

use std::error::Error;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::commands::split::SplitRequest;

use super::{CommonIoArgs, StatsOutputArgs, emit_report, parse_delimiter_alias};

/// `csv-ops split` の引数
#[derive(Args, Debug)]
pub struct SplitArgs {
    #[command(flatten)]
    pub io: CommonIoArgs,

    /// 分割対象の列 (列名または列番号、単一指定)
    #[arg(short = 'c', long, value_name = "COLUMN")]
    pub col: String,

    /// 区切り文字列 (この文字列で分割する)
    #[arg(long, value_name = "SEP")]
    pub by: String,

    /// 末尾に追加する出力列名 (カンマ区切り。要素数が分割後の列数を兼ねる)
    #[arg(long, value_name = "NAMES")]
    pub out_cols: String,

    #[command(flatten)]
    pub stats: StatsOutputArgs,
}

/// split サブコマンドのエントリポイント
pub fn run(args: SplitArgs) -> Result<ExitCode, Box<dyn Error>> {
    if args.by.is_empty() {
        return Err("--by が空です".into());
    }
    // 出力列名はカンマ区切り。空の列名は許可しない (空要素 / 空文字指定を弾く)
    let out_cols: Vec<String> = args
        .out_cols
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    if out_cols.iter().any(|s| s.is_empty()) {
        return Err("--out-cols に空の列名が含まれています".into());
    }

    let request = SplitRequest {
        col: ColumnRef::parse(args.col.trim()),
        by: args.by,
        out_cols,
        input: args.io.input,
        output: args.io.output,
        input_encoding: args.io.input_encoding,
        delimiter: parse_delimiter_alias(&args.io.delimiter)?,
        has_headers: !args.io.no_headers,
        dry_run: args.io.dry_run,
    };

    let stats = crate::commands::split::run(request)?;
    emit_report(&stats, args.stats.json, args.stats.quiet)?;
    Ok(ExitCode::SUCCESS)
}
