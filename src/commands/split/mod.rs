pub mod transform;

use std::path::PathBuf;

use crate::column::ColumnRef;
use crate::error::CsvOpsError;
use crate::io::resolve_input_encoding;
use crate::pipeline::{PipelineOptions, run_pipeline};
use crate::stats::Stats;

use transform::SplitTransform;

/// split::run に渡す設定一式
pub struct SplitRequest {
    /// 分割対象の列 (単一指定、列名または列番号)
    pub col: ColumnRef,
    /// 区切り文字列
    pub by: String,
    /// 追加する出力列名 (要素数が分割後の列数を兼ねる)
    pub out_cols: Vec<String>,
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp / auto)
    /// 出力は入力と同一エンコーディングで書き出す
    pub input_encoding: String,
    /// 区切り文字
    pub delimiter: u8,
    /// ヘッダー行の有無
    pub has_headers: bool,
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// split サブコマンドのエントリポイント
/// 指定列を区切り文字で複数列へ分割する
pub fn run(request: SplitRequest) -> Result<Stats, CsvOpsError> {
    let SplitRequest {
        col,
        by,
        out_cols,
        input,
        output,
        input_encoding,
        delimiter,
        has_headers,
        dry_run,
    } = request;

    // 入力エンコーディングは auto 指定ならファイル先頭から推定する
    // 出力は入力と同一エンコーディングで書き出す
    let input_encoding = resolve_input_encoding(&input_encoding, &input)?;

    let mut transform = SplitTransform::new(col, by, out_cols);
    let opts = PipelineOptions {
        input,
        output,
        input_encoding,
        output_encoding: input_encoding,
        // split は列構造のみ変えるので入出力で区切り文字は同一
        input_delimiter: delimiter,
        output_delimiter: delimiter,
        has_headers,
        dry_run,
    };

    let rows = run_pipeline(&mut transform, &opts)?;
    let mut stats = transform.stats;
    stats.rows_processed = rows;
    Ok(stats)
}
