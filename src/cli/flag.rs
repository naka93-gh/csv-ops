use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::commands::flag::{FlagRequest, RuleSource};

use super::{CommonIoArgs, StatsOutputArgs, emit_report, parse_delimiter_alias};

/// `csv-ops flag` の引数
#[derive(Args, Debug)]
pub struct FlagArgs {
    #[command(flatten)]
    pub io: CommonIoArgs,

    /// 設定ファイル (TOML)。指定時は --pattern / -c / --out-col は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (カンマ区切り、列名または列番号。CLI 引数モードで必須)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// マッチ判定の正規表現 (CLI 引数モード)
    #[arg(long)]
    pub pattern: Option<String>,

    /// 追加する列の名前 (CLI 引数モード)
    #[arg(long, value_name = "NAME")]
    pub out_col: Option<String>,

    #[command(flatten)]
    pub stats: StatsOutputArgs,
}

/// flag サブコマンドのエントリポイント
pub fn run(args: FlagArgs) -> Result<ExitCode, Box<dyn Error>> {
    // ルール指定の解決
    // --config が優先、なければ --pattern / -c / --out-col の CLI 引数モード
    let rules = match args.config {
        Some(path) => RuleSource::Config(path),
        None => {
            let pattern = args
                .pattern
                .ok_or("--config か --pattern / -c / --out-col のいずれかを指定してください")?;
            let cols = args.columns.ok_or("-c <列> が必要です")?;
            let out_col = args.out_col.ok_or("--out-col が必要です")?;
            let columns = ColumnRef::parse_csv_list(&cols);
            RuleSource::Inline {
                pattern,
                columns,
                out_col,
            }
        }
    };

    let delimiter = parse_delimiter_alias(&args.io.delimiter)?;

    let request = FlagRequest {
        rules,
        input: args.io.input,
        output: args.io.output,
        input_encoding: args.io.input_encoding,
        output_encoding: args.io.output_encoding,
        delimiter,
        has_headers: !args.io.no_headers,
        dry_run: args.io.dry_run,
    };

    let stats = crate::commands::flag::run(request)?;
    emit_report(
        &stats,
        &args.stats.stats_format,
        args.stats.stats_file.as_deref(),
    )?;
    Ok(ExitCode::SUCCESS)
}
