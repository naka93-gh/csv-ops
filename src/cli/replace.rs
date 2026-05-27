// replace サブコマンドの CLI 引数定義と実行ハンドラ
// Config モード (--config) と CLI 引数モード (--from/--to) の両対応

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::commands::replace::{ColumnTarget, ReplaceRequest, RuleSource};

use super::{CommonIoArgs, StatsOutputArgs, emit_report, parse_delimiter_alias};

/// `csv-ops replace` の引数
#[derive(Args, Debug)]
pub struct ReplaceArgs {
    #[command(flatten)]
    pub io: CommonIoArgs,

    /// 設定ファイル (TOML)。指定時は --from / --to は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 置換元 (CLI 引数モード)
    #[arg(long)]
    pub from: Option<String>,

    /// 置換先 (CLI 引数モード)
    #[arg(long)]
    pub to: Option<String>,

    /// --from を正規表現として扱う
    #[arg(long)]
    pub regex: bool,

    /// 大文字小文字を区別しない
    #[arg(long)]
    pub case_insensitive: bool,

    /// 対象列 (カンマ区切り、列名または列番号)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// 全カラムを対象にする (-c と排他)
    #[arg(long)]
    pub all_columns: bool,

    #[command(flatten)]
    pub stats: StatsOutputArgs,
}

/// replace サブコマンドのエントリポイント
pub fn run(args: ReplaceArgs) -> Result<ExitCode, Box<dyn Error>> {
    // ルール指定の解決
    // --config が優先、なければ --from / --to の CLI 引数モード
    let rules = match args.config {
        Some(path) => RuleSource::Config(path),
        None => {
            let from = args
                .from
                .ok_or("--config か --from/--to のいずれかを指定してください")?;
            let to = args.to.ok_or("--to が必要です")?;
            RuleSource::Inline {
                from,
                to,
                regex: args.regex,
            }
        }
    };

    // 対象列の解決
    // -c か --all-columns のいずれか必須
    let columns = match (args.columns, args.all_columns) {
        (Some(_), true) => {
            return Err("-c と --all-columns は同時に指定できません".into());
        }
        (None, false) => {
            return Err("-c <列> か --all-columns のいずれかを指定してください".into());
        }
        (Some(cols), false) => ColumnTarget::Specified(ColumnRef::parse_csv_list(&cols)),
        (None, true) => ColumnTarget::All,
    };

    let delimiter = parse_delimiter_alias(&args.io.delimiter)?;

    let request = ReplaceRequest {
        rules,
        input: args.io.input,
        output: args.io.output,
        input_encoding: args.io.input_encoding,
        delimiter,
        has_headers: !args.io.no_headers,
        case_insensitive: args.case_insensitive,
        columns,
        dry_run: args.io.dry_run,
    };

    let stats = crate::commands::replace::run(request)?;
    emit_report(&stats, args.stats.json, args.stats.quiet)?;
    Ok(ExitCode::SUCCESS)
}
