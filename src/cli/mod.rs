// CLI 定義のルート
// 各サブコマンドの引数定義は子モジュールで持ち、ここでは Cli/Command の集約と dispatch を行う

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::stats_report::StatsReport;

pub mod convert;
pub mod extract;
pub mod flag;
pub mod info;
pub mod mask;
pub mod replace;
pub mod similarity;

/// 5 サブコマンド (mask/replace/flag/extract/similarity) で共有する入出力系の引数
#[derive(Args, Debug)]
pub struct CommonIoArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp / auto)
    /// 出力エンコーディングは入力と同一になる
    #[arg(long, default_value = "utf-8")]
    pub input_encoding: String,

    /// 区切り文字 (comma / tab / pipe / semicolon)
    #[arg(long, value_name = "ALIAS", default_value = "comma")]
    pub delimiter: String,

    /// ヘッダ行なし CSV
    #[arg(long)]
    pub no_headers: bool,

    /// 出力ファイルへ書き込まず、統計のみ表示する
    #[arg(long)]
    pub dry_run: bool,
}

/// 統計出力形式の共通引数 (convert を含む 6 サブコマンドで使う)
#[derive(Args, Debug)]
pub struct StatsOutputArgs {
    /// 統計の出力形式 (text / json)
    #[arg(long, value_name = "FORMAT", default_value = "text")]
    pub stats_format: String,
}

/// 統計／メタ情報レポートを指定形式でフォーマットし標準出力へ書く
pub fn emit_report<R: StatsReport>(report: &R, format: &str) -> Result<(), Box<dyn Error>> {
    let body = match format {
        "json" => report.to_json(),
        "text" => report.to_text(),
        other => return Err(format!("不明な出力形式: {} (text / json)", other).into()),
    };
    println!("{}", body);
    Ok(())
}

/// 区切り文字エイリアスを 1 バイトに変換する
/// comma / tab / pipe / semicolon のいずれかを受け付ける
pub fn parse_delimiter_alias(alias: &str) -> Result<u8, Box<dyn Error>> {
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
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// サブコマンドの集約
/// Command は CLI 起動時に 1 度だけ生成され dispatch するだけなので、
/// variant 間のサイズ差は実害がない (large_enum_variant は許容)
#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// 指定カラムを文字数保持でマスクする
    Mask(mask::MaskArgs),
    /// 指定カラムを文字置換する (全カラムオプションあり)
    Replace(replace::ReplaceArgs),
    /// 指定カラムをパターン判定して真偽値の列を追加する
    Flag(flag::FlagArgs),
    /// 指定カラムからパターンマッチした文字列を抽出して列を追加する
    Extract(extract::ExtractArgs),
    /// 指定カラムを辞書とベストマッチして列を追加する
    Similarity(similarity::SimilarityArgs),
    /// エンコーディングと区切り文字を変換する
    Convert(convert::ConvertArgs),
    /// CSV のエンコーディング・行数などの情報を表示する
    Info(info::InfoArgs),
}

impl Cli {
    pub fn dispatch(self) -> Result<ExitCode, Box<dyn Error>> {
        match self.command {
            Command::Mask(args) => mask::run(args),
            Command::Replace(args) => replace::run(args),
            Command::Flag(args) => flag::run(args),
            Command::Extract(args) => extract::run(args),
            Command::Similarity(args) => similarity::run(args),
            Command::Convert(args) => convert::run(args),
            Command::Info(args) => info::run(args),
        }
    }
}
