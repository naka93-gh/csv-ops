use csv::StringRecord;

use crate::column::{ColumnRef, resolve_indices};
use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

/// マスク処理本体
/// 指定列の各セルを、文字数を保ったまま mask_char で塗り潰す
pub(crate) struct MaskTransform {
    /// マスク対象の列指定 (init で indices に解決する)
    columns: Vec<ColumnRef>,
    /// 塗り潰しに使う文字
    mask_char: char,
    /// 解決済みの対象列インデックス
    indices: Vec<usize>,
    /// 処理統計
    pub stats: Stats,
}

impl MaskTransform {
    pub fn new(columns: Vec<ColumnRef>, mask_char: char) -> Self {
        Self {
            columns,
            mask_char,
            indices: Vec::new(),
            stats: Stats::default(),
        }
    }
}

impl RecordTransform for MaskTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // 列指定をヘッダー照合でインデックスへ解決する
        self.indices = resolve_indices(&self.columns, headers)?;
        // ヘッダー行はマスクせずそのまま出力する
        Ok(headers.cloned())
    }

    fn on_record(&mut self, record: &mut StringRecord, _row: u64) -> Result<(), CsvOpsError> {
        // ヘッダー無し + 列番号指定では init で範囲チェックできないため、行ごとに検証する
        for &i in &self.indices {
            if i >= record.len() {
                return Err(TransformError::IndexOutOfRange {
                    index: i,
                    columns: record.len(),
                }
                .into());
            }
        }

        // 対象列のセルを文字数分の mask_char で塗り潰す
        let mut new_fields: Vec<String> = Vec::with_capacity(record.len());
        let mut masked_any = false;
        for (i, field) in record.iter().enumerate() {
            // 空セルは塗り潰しても変化しないのでマスク対象から除く
            if self.indices.contains(&i) && !field.is_empty() {
                // chars().count() で塗り潰す: バイト長だとマルチバイトで桁数が狂う
                new_fields.push(self.mask_char.to_string().repeat(field.chars().count()));
                self.stats.changes_total += 1;
                masked_any = true;
            } else {
                new_fields.push(field.to_string());
            }
        }
        *record = StringRecord::from(new_fields);
        if masked_any {
            self.stats.rows_changed += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
