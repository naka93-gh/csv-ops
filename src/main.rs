use std::process::ExitCode;

use clap::Parser;

mod cli;

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
