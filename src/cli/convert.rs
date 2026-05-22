use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use csv_ops::convert::ConvertRequest;

use super::parse_delimiter_alias;

/// `csv-ops convert` の引数
#[derive(Args, Debug)]
pub(crate) struct ConvertArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp / auto)
    #[arg(long, default_value = "utf-8")]
    pub input_encoding: String,

    /// 出力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub output_encoding: String,

    /// 入力区切り文字 (comma / tab / pipe / semicolon)
    #[arg(long, default_value = "comma")]
    pub input_delimiter: String,

    /// 出力区切り文字 (comma / tab / pipe / semicolon)
    #[arg(long, default_value = "comma")]
    pub output_delimiter: String,
}

/// convert サブコマンドのエントリポイント
pub(crate) fn run(args: ConvertArgs) -> Result<ExitCode, Box<dyn Error>> {
    let request = ConvertRequest {
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        input_delimiter: parse_delimiter_alias(&args.input_delimiter)?,
        output_delimiter: parse_delimiter_alias(&args.output_delimiter)?,
    };

    let stats = csv_ops::convert::run(request)?;
    println!("変換行数: {}", stats.rows);
    Ok(ExitCode::SUCCESS)
}
