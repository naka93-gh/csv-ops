use std::path::PathBuf;

use csv::StringRecord;

use crate::error::CsvOpsError;
use crate::io::{resolve_encoding, resolve_input_encoding};
use crate::pipeline::{PipelineOptions, RecordTransform, run_pipeline};
use crate::stats::Stats;

/// convert::run に渡す設定一式
pub struct ConvertRequest {
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp / auto)
    pub input_encoding: String,
    /// 出力エンコーディング名
    pub output_encoding: String,
    /// 入力区切り文字
    pub input_delimiter: u8,
    /// 出力区切り文字
    pub output_delimiter: u8,
    /// dry-run (true なら出力ファイルへ書き込まず、行数のみ集計する)
    pub dry_run: bool,
}

/// convert は内容を変えず素通しするだけなので、行を書き換えない transform を使う
struct PassThrough;

impl RecordTransform for PassThrough {
    fn init(
        &mut self,
        _headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        Ok(None)
    }

    fn on_record(&mut self, _record: &mut StringRecord, _row: u64) -> Result<(), CsvOpsError> {
        Ok(())
    }
}

/// convert サブコマンドのエントリポイント
/// エンコーディングと区切り文字だけを変換して素通しする
/// ヒット概念がないため rows_changed / changes_total は 0 のまま返す
pub fn run(request: ConvertRequest) -> Result<Stats, CsvOpsError> {
    // 入力エンコーディングは auto 指定ならファイル先頭から推定する
    let input_encoding = resolve_input_encoding(&request.input_encoding, &request.input)?;
    let output_encoding = resolve_encoding(&request.output_encoding)?;

    let opts = PipelineOptions {
        input: request.input,
        output: request.output,
        input_encoding,
        output_encoding,
        input_delimiter: request.input_delimiter,
        output_delimiter: request.output_delimiter,
        // convert は全行を等価に素通しするのでヘッダー概念を持たない
        has_headers: false,
        dry_run: request.dry_run,
    };
    let rows = run_pipeline(&mut PassThrough, &opts)?;
    Ok(Stats {
        rows_processed: rows,
        ..Default::default()
    })
}
