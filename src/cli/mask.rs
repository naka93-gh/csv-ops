// mask サブコマンドの CLI 引数定義と実行ハンドラ
// Config モード (--config) と CLI 引数モード (-c) の両対応

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use csv_ops::ColumnRef;
use csv_ops::mask::{MaskRequest, MaskSource};

use super::parse_delimiter_alias;

/// `csv-ops mask` の引数
#[derive(Args, Debug)]
pub(crate) struct MaskArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// 設定ファイル (TOML)。指定時は -c / --mask-char は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (カンマ区切り、列名または列番号)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// マスク文字 (先頭 1 文字を使用)
    #[arg(long, value_name = "CHAR", default_value = "*")]
    pub mask_char: String,

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub input_encoding: String,

    /// 出力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub output_encoding: String,

    /// 区切り文字 (comma / tab / pipe / semicolon)
    #[arg(long, value_name = "ALIAS", default_value = "comma")]
    pub delimiter: String,

    /// ヘッダ行なし CSV
    #[arg(long)]
    pub no_headers: bool,

    /// 出力ファイルへ書き込まず、統計のみ表示する
    #[arg(long)]
    pub dry_run: bool,

    /// 統計の出力形式 (text / json)
    #[arg(long, value_name = "FORMAT", default_value = "text")]
    pub stats_format: String,

    /// 統計の出力先ファイル (未指定なら標準出力)
    #[arg(long, value_name = "PATH")]
    pub stats_file: Option<PathBuf>,
}

/// mask サブコマンドのエントリポイント
pub(crate) fn run(args: MaskArgs) -> Result<ExitCode, Box<dyn Error>> {
    // 列指定の解決
    // --config が優先、なければ -c の CLI 引数モード
    let source = match args.config {
        Some(path) => MaskSource::Config(path),
        None => {
            let cols = args
                .columns
                .ok_or("--config か -c <列> のいずれかを指定してください")?;
            let columns: Vec<ColumnRef> = cols
                .split(',')
                .map(|s| ColumnRef::parse(s.trim()))
                .collect();
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
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        delimiter: parse_delimiter_alias(&args.delimiter)?,
        has_headers: !args.no_headers,
        dry_run: args.dry_run,
    };

    let stats = csv_ops::mask::run(request)?;

    // 統計を指定形式でフォーマット
    let formatted = match args.stats_format.as_str() {
        "json" => stats.to_json(),
        "text" => stats.to_text(),
        other => return Err(format!("不明な統計形式: {} (text / json)", other).into()),
    };

    // --stats-file 指定ならファイルへ、なければ標準出力へ
    match args.stats_file {
        Some(path) => std::fs::write(&path, formatted)?,
        None => println!("{}", formatted),
    }
    Ok(ExitCode::SUCCESS)
}
