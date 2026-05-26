use std::process::ExitCode;

use clap::Parser;

mod cli;
mod column;
mod convert;
mod error;
mod extract;
mod flag;
mod info;
mod io;
mod mask;
mod pipeline;
mod replace;
mod rule_id;
mod similarity;
mod stats;
mod stats_report;
mod text;

/// エントリーポイント
/// clap で解析し dispatch
fn main() -> ExitCode {
    let cli_args = cli::Cli::parse();
    match cli_args.dispatch() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}
