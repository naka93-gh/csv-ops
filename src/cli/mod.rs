// CLI 定義のルート
// 各サブコマンドの引数定義は子モジュールで持ち、ここでは Cli/Command の集約と dispatch を行う

use std::error::Error;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

pub(crate) mod convert;
pub(crate) mod extract;
pub(crate) mod flag;
pub(crate) mod mask;
pub(crate) mod replace;

/// 区切り文字エイリアスを 1 バイトに変換する
/// comma / tab / pipe / semicolon のいずれかを受け付ける
pub(crate) fn parse_delimiter_alias(alias: &str) -> Result<u8, Box<dyn Error>> {
    match alias {
        "comma" => Ok(b','),
        "tab" => Ok(b'\t'),
        "pipe" => Ok(b'|'),
        "semicolon" => Ok(b';'),
        other => Err(format!(
            "不明な区切り文字: {} (comma / tab / pipe / semicolon)",
            other
        )
        .into()),
    }
}

/// csv-ops のトップレベル CLI
#[derive(Parser, Debug)]
#[command(name = "csv-ops", version, about = "CSV 処理用の Rust 製 CLI ツール", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// サブコマンドの集約
/// Command は CLI 起動時に 1 度だけ生成され dispatch するだけなので、
/// variant 間のサイズ差は実害がない (large_enum_variant は許容)
#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Command {
    /// 指定カラムを文字数保持でマスクする
    Mask(mask::MaskArgs),
    /// 指定カラムを文字置換する (全カラムオプションあり)
    Replace(replace::ReplaceArgs),
    /// 指定カラムをパターン判定して真偽値の列を追加する
    Flag(flag::FlagArgs),
    /// 指定カラムからパターンマッチした文字列を抽出して列を追加する
    Extract(extract::ExtractArgs),
    /// エンコーディングと区切り文字を変換する
    Convert(convert::ConvertArgs),
}

impl Cli {
    pub fn dispatch(self) -> Result<ExitCode, Box<dyn Error>> {
        match self.command {
            Command::Mask(args) => mask::run(args),
            Command::Replace(args) => replace::run(args),
            Command::Flag(args) => flag::run(args),
            Command::Extract(args) => extract::run(args),
            Command::Convert(args) => convert::run(args),
        }
    }
}
