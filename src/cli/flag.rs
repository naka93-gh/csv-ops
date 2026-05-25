use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use crate::column::ColumnRef;
use crate::flag::{FlagRequest, RuleSource};

use super::{emit_report, parse_delimiter_alias};

/// `csv-ops flag` の引数
#[derive(Args, Debug)]
pub(crate) struct FlagArgs {
    /// 入力ファイル
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// 設定ファイル (TOML)。指定時は --pattern / -c / --out-col は無視
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 対象列 (カンマ区切り、列名または列番号。CLI 引数モードで必須)
    #[arg(short = 'c', long, value_name = "COLUMNS")]
    pub columns: Option<String>,

    /// マッチ判定の正規表現 (CLI 引数モード)
    #[arg(long)]
    pub pattern: Option<String>,

    /// 追加する列の名前 (CLI 引数モード)
    #[arg(long, value_name = "NAME")]
    pub out_col: Option<String>,

    /// 入力エンコーディング (utf-8 / shift_jis / euc-jp / auto)
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

/// flag サブコマンドのエントリポイント
pub(crate) fn run(args: FlagArgs) -> Result<ExitCode, Box<dyn Error>> {
    // ルール指定の解決
    // --config が優先、なければ --pattern / -c / --out-col の CLI 引数モード
    let rules = match args.config {
        Some(path) => RuleSource::Config(path),
        None => {
            let pattern = args
                .pattern
                .ok_or("--config か --pattern / -c / --out-col のいずれかを指定してください")?;
            let cols = args.columns.ok_or("-c <列> が必要です")?;
            let out_col = args.out_col.ok_or("--out-col が必要です")?;
            let columns = cols
                .split(',')
                .map(|s| ColumnRef::parse(s.trim()))
                .collect();
            RuleSource::Inline {
                pattern,
                columns,
                out_col,
            }
        }
    };

    let delimiter = parse_delimiter_alias(&args.delimiter)?;

    let request = FlagRequest {
        rules,
        input: args.input,
        output: args.output,
        input_encoding: args.input_encoding,
        output_encoding: args.output_encoding,
        delimiter,
        has_headers: !args.no_headers,
        dry_run: args.dry_run,
    };

    let stats = crate::flag::run(request)?;
    emit_report(&stats, &args.stats_format, args.stats_file.as_deref())?;
    Ok(ExitCode::SUCCESS)
}
