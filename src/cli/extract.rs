use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use csv_ops::ColumnRef;
use csv_ops::extract::{ExtractRequest, RuleSource};

/// `csv-ops extract` の引数
#[derive(Args, Debug)]
pub(crate) struct ExtractArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// 設定ファイル (TOML)。指定時は --pattern / -c / --out-col は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (列名または列番号、1 列。CLI 引数モードで必須)
    #[arg(short = 'c', long, value_name = "COLUMN")]
    pub column: Option<String>,

    /// 抽出に使う正規表現 (CLI 引数モード)
    #[arg(long)]
    pub pattern: Option<String>,

    /// 追加する列の名前 (CLI 引数モード)
    #[arg(long, value_name = "NAME")]
    pub out_col: Option<String>,

    /// 複数マッチの区切り文字 (省略時 ";")
    #[arg(long, value_name = "SEP")]
    pub separator: Option<String>,

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

/// extract サブコマンドのエントリポイント
pub(crate) fn run(args: ExtractArgs) -> Result<ExitCode, Box<dyn Error>> {
    // ルール指定の解決
    // --config が優先、なければ --pattern / -c / --out-col の CLI 引数モード
    let rules = match args.config {
        Some(path) => RuleSource::Config(path),
        None => {
            let pattern = args
                .pattern
                .ok_or("--config か --pattern / -c / --out-col のいずれかを指定してください")?;
            let col = args.column.ok_or("-c <列> が必要です")?;
            let out_col = args.out_col.ok_or("--out-col が必要です")?;
            RuleSource::Inline {
                pattern,
                column: ColumnRef::parse(col.trim()),
                out_col,
                separator: args.separator,
            }
        }
    };

    // 区切り文字は先頭バイトのみ採用 (csv crate の API は u8)
    let delimiter = args.delimiter.as_bytes().first().copied().unwrap_or(b',');

    let request = ExtractRequest {
        rules,
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        delimiter,
        has_headers: !args.no_headers,
        dry_run: args.dry_run,
    };

    let stats = csv_ops::extract::run(request)?;

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
