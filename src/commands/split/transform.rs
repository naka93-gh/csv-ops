use std::slice;

use csv::StringRecord;

use crate::column::{ColumnRef, check_output_conflicts, ensure_in_range, resolve_indices};
use crate::error::CsvOpsError;
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

/// 列分割処理本体
/// 指定列を区切り文字列で分割し、結果を末尾に複数列追加する
/// 元の列は変更せず、余剰は末尾列へ結合し、不足は空文字で埋める
pub struct SplitTransform {
    /// 分割対象の列指定 (init で src_idx に解決する)
    col: ColumnRef,
    /// 区切り文字列
    by: String,
    /// 追加する出力列名 (要素数が分割後の列数を兼ねる)
    out_cols: Vec<String>,
    /// 解決済みの分割元列インデックス
    src_idx: usize,
    /// 処理統計
    pub stats: Stats,
}

impl SplitTransform {
    pub fn new(col: ColumnRef, by: String, out_cols: Vec<String>) -> Self {
        Self {
            col,
            by,
            out_cols,
            src_idx: 0,
            stats: Stats::default(),
        }
    }

    /// 分割元の値を out_cols の列数 (n) に揃えて分割する
    /// 区切りが n-1 個より多ければ末尾列が残りを吸収し、不足分は空文字で埋める
    /// 戻り値の bool は実際に分割が起きたか (区切りが 1 つ以上見つかったか)
    fn split_value(&self, value: &str) -> (Vec<String>, bool) {
        let n = self.out_cols.len();
        let mut parts: Vec<String> = value
            .splitn(n, self.by.as_str())
            .map(String::from)
            .collect();
        let did_split = parts.len() >= 2;
        parts.resize(n, String::new());
        (parts, did_split)
    }
}

impl RecordTransform for SplitTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // 単一列をヘッダー照合でインデックスへ解決する
        self.src_idx = resolve_indices(slice::from_ref(&self.col), headers)?[0];

        match headers {
            Some(h) => {
                // out_cols 名が既存カラムおよび out_cols 同士と衝突しないか検査する
                check_output_conflicts(Some(h), self.out_cols.iter().map(|s| s.as_str()))?;

                // 既存カラム + out_cols を出力ヘッダーとする (末尾追加)
                let mut out: Vec<String> = h.iter().map(String::from).collect();
                out.extend(self.out_cols.iter().cloned());
                Ok(Some(StringRecord::from(out)))
            }
            None => Ok(None),
        }
    }

    fn on_record(&mut self, record: &mut StringRecord, _row: u64) -> Result<(), CsvOpsError> {
        // ヘッダー無し + 列番号指定では init で範囲チェックできないため行ごとに確認する
        ensure_in_range([self.src_idx], record.len())?;

        let (parts, did_split) = self.split_value(&record[self.src_idx]);

        // 既存フィールドはそのまま、分割結果を末尾に追加する
        let mut fields: Vec<String> = record.iter().map(String::from).collect();
        fields.extend(parts);
        *record = StringRecord::from(fields);

        if did_split {
            self.stats.rows_changed += 1;
            self.stats.changes_total += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
