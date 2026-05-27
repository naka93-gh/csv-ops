use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::commands::info::InfoRequest;

use super::{emit_report, parse_delimiter_alias};

/// `csv-ops info` の引数
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 統計を JSON 形式で出力する (未指定なら text)
    #[arg(long)]
    pub json: bool,

    /// 区切り文字 (comma / tab / pipe / semicolon)。未指定ならヘッダー行から自動判定
    #[arg(long, value_name = "ALIAS")]
    pub input_delimiter: Option<String>,

    /// クォート文字
    #[arg(long, value_name = "CHAR", default_value = "\"")]
    pub input_quote: String,
}

/// info サブコマンドのエントリポイント
pub fn run(args: InfoArgs) -> Result<ExitCode, Box<dyn Error>> {
    // 区切り文字は指定があればエイリアスを解決、なければ None (自動判定)
    let delimiter = match args.input_delimiter {
        Some(alias) => Some(parse_delimiter_alias(&alias)?),
        None => None,
    };

    // クォート文字は先頭バイトのみ採用 (csv crate の API は u8)
    let quote = args.input_quote.as_bytes().first().copied().unwrap_or(b'"');

    let request = InfoRequest {
        input: args.input,
        delimiter,
        quote,
    };

    let report = crate::commands::info::run(request)?;
    // info は主出力が統計なので --quiet 非対応 (常に false)
    emit_report(&report, args.json, false)?;
    Ok(ExitCode::SUCCESS)
}
