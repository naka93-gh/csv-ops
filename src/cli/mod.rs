// CLI 定義のルート
// 各サブコマンドの引数定義は子モジュールで持ち、ここでは Cli/Command の集約と dispatch を行う

use std::error::Error;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

pub(crate) mod mask;

/// csv-ops のトップレベル CLI
#[derive(Parser, Debug)]
#[command(name = "csv-ops", version, about = "CSV 処理用の Rust 製 CLI ツール", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// サブコマンドの集約
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// 指定カラムを文字数保持でマスクする
    Mask(mask::MaskArgs),
}

impl Cli {
    pub fn dispatch(self) -> Result<ExitCode, Box<dyn Error>> {
        match self.command {
            Command::Mask(args) => mask::run(args),
        }
    }
}
