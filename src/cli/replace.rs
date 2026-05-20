// replace サブコマンドの CLI 引数定義と実行ハンドラ
// Config モード (--config) と CLI 引数モード (--from/--to) の両対応

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use csv_ops::ColumnRef;
use csv_ops::replace::{ColumnTarget, ReplaceRequest, RuleSource};

/// `csv-ops replace` の引数
#[derive(Args, Debug)]
pub(crate) struct ReplaceArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

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

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub input_encoding: String,

    /// 出力エンコーディング (utf-8 / shift_jis / euc-jp)
    #[arg(long, default_value = "utf-8")]
    pub output_encoding: String,

    /// 区切り文字
    #[arg(long, default_value = ",")]
    pub delimiter: String,

    /// ヘッダ行なし CSV
    #[arg(long)]
    pub no_headers: bool,

    /// 対象列 (カンマ区切り、列名または列番号)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// 全カラムを対象にする (-c と排他)
    #[arg(long)]
    pub all_columns: bool,

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

/// replace サブコマンドのエントリポイント
pub(crate) fn run(args: ReplaceArgs) -> Result<ExitCode, Box<dyn Error>> {
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
        (Some(cols), false) => {
            let refs = cols
                .split(',')
                .map(|s| ColumnRef::parse(s.trim()))
                .collect();
            ColumnTarget::Specified(refs)
        }
        (None, true) => ColumnTarget::All,
    };

    // 区切り文字は先頭バイトのみ採用 (csv crate の API は u8)
    let delimiter = args.delimiter.as_bytes().first().copied().unwrap_or(b',');

    let request = ReplaceRequest {
        rules,
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        delimiter,
        has_headers: !args.no_headers,
        case_insensitive: args.case_insensitive,
        columns,
        dry_run: args.dry_run,
    };

    let stats = csv_ops::replace::run(request)?;

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
