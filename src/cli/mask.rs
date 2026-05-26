// mask サブコマンドの CLI 引数定義と実行ハンドラ
// Config モード (--config) と CLI 引数モード (-c) の両対応

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::mask::{MaskRequest, MaskSource};

use super::{CommonIoArgs, StatsOutputArgs, emit_report, parse_delimiter_alias};

/// `csv-ops mask` の引数
#[derive(Args, Debug)]
pub struct MaskArgs {
    #[command(flatten)]
    pub io: CommonIoArgs,

    /// 設定ファイル (TOML)。指定時は -c / --mask-char は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (カンマ区切り、列名または列番号)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// マスク文字 (先頭 1 文字を使用)
    #[arg(long, value_name = "CHAR", default_value = "*")]
    pub mask_char: String,

    #[command(flatten)]
    pub stats: StatsOutputArgs,
}

/// mask サブコマンドのエントリポイント
pub fn run(args: MaskArgs) -> Result<ExitCode, Box<dyn Error>> {
    // 列指定の解決
    // --config が優先、なければ -c の CLI 引数モード
    let source = match args.config {
        Some(path) => MaskSource::Config(path),
        None => {
            let cols = args
                .columns
                .ok_or("--config か -c <列> のいずれかを指定してください")?;
            let columns = ColumnRef::parse_csv_list(&cols);
            let mask_char = args
                .mask_char
                .chars()
                .next()
                .ok_or("--mask-char が空です")?;
            MaskSource::Inline { columns, mask_char }
        }
    };

    let request = MaskRequest {
        source,
        input: args.io.input,
        output: args.io.output,
        input_encoding: args.io.input_encoding,
        output_encoding: args.io.output_encoding,
        delimiter: parse_delimiter_alias(&args.io.delimiter)?,
        has_headers: !args.io.no_headers,
        dry_run: args.io.dry_run,
    };

    let stats = crate::mask::run(request)?;
    emit_report(
        &stats,
        &args.stats.stats_format,
        args.stats.stats_file.as_deref(),
    )?;
    Ok(ExitCode::SUCCESS)
}
