use std::collections::HashSet;

use csv::StringRecord;

use crate::column::resolve_indices;
use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;

use super::ColumnTarget;
use super::rule::CompiledRule;
use super::stats::ReplaceStats;

/// 解決済みの置換対象列
/// init で ColumnTarget (列名 / 列番号) をインデックスへ解決した結果
pub(crate) enum TargetColumns {
    /// 全カラム横断
    All,
    /// 解決済みの対象列インデックス
    Indices(Vec<usize>),
}

impl TargetColumns {
    /// col_index が置換対象かどうか
    fn includes(&self, col_index: usize) -> bool {
        match self {
            TargetColumns::All => true,
            TargetColumns::Indices(idx) => idx.contains(&col_index),
        }
    }
}

/// 置換処理本体
pub(crate) struct ReplaceTransform {
    /// compile 済みルール
    rules: Vec<CompiledRule>,
    /// 列指定の未解決スペック (init で target へ解決)
    columns: ColumnTarget,
    /// 解決済みの対象列
    target: TargetColumns,
    /// ヘッダー (エラーメッセージのカラム名表示に使う)
    headers: Option<StringRecord>,
    /// 処理統計
    pub stats: ReplaceStats,
}

impl ReplaceTransform {
    pub fn new(rules: Vec<CompiledRule>, columns: ColumnTarget, stats: ReplaceStats) -> Self {
        Self {
            rules,
            columns,
            target: TargetColumns::All,
            headers: None,
            stats,
        }
    }

    /// レコード単位の置換処理
    /// 戻り値はこの行でマッチしたルール index の列 (統計集計用、重複あり = マッチ回数分)
    fn apply(&self, record: &mut StringRecord, row: u64) -> Result<Vec<usize>, TransformError> {
        // ヘッダー無し + 列番号指定では解決時に範囲チェックできないため、ここで検証する
        if let TargetColumns::Indices(idx) = &self.target {
            for &i in idx {
                if i >= record.len() {
                    return Err(TransformError::IndexOutOfRange {
                        index: i,
                        columns: record.len(),
                    });
                }
            }
        }

        // cell ごとに置換し、最後に record を差し替える
        // (csv::StringRecord は cell 単位の差し替えができないため)
        let mut row_matches: Vec<usize> = Vec::new();
        let mut new_fields: Vec<String> = Vec::with_capacity(record.len());
        for (col_index, field) in record.iter().enumerate() {
            // 対象列でなければ元の値をそのまま残す
            if !self.target.includes(col_index) {
                new_fields.push(field.to_string());
                continue;
            }

            // エラーメッセージ用のカラム名 (ヘッダーがあれば名前、なければ列番号)
            let column_name = self
                .headers
                .as_ref()
                .and_then(|h| h.get(col_index))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("col[{}]", col_index));

            let (replaced, matched) = self.apply_cell(field, row, &column_name)?;
            row_matches.extend(matched);
            new_fields.push(replaced);
        }

        *record = StringRecord::from(new_fields);
        Ok(row_matches)
    }

    /// セル単位の置換処理
    /// 戻り値は (置換後の文字列, マッチしたルール index 列)
    fn apply_cell(
        &self,
        cell: &str,
        row: u64,
        column: &str,
    ) -> Result<(String, Vec<usize>), TransformError> {
        // 全ルールのマッチ位置を収集する
        // 単純置換・正規表現どちらも matcher().find_iter で非オーバーラップマッチを得る
        // 元の cell に対して全ルール評価するので、評価の連鎖はしない
        let mut matches: Vec<(usize, usize, &CompiledRule)> = Vec::new();
        for rule in &self.rules {
            for m in rule.matcher().find_iter(cell) {
                matches.push((m.start(), m.end(), rule));
            }
        }

        // マッチがなければ変更しない
        if matches.is_empty() {
            return Ok((cell.to_string(), Vec::new()));
        }

        // 衝突検知と後ろからの置換のため、開始位置でソートする
        matches.sort_by_key(|(start, _, _)| *start);

        // 衝突検知: 前のマッチの end が次のマッチの start を越えていたら範囲が重なる
        for i in 1..matches.len() {
            if matches[i - 1].1 > matches[i].0 {
                return Err(TransformError::RuntimeCollision {
                    row,
                    column: column.to_string(),
                    rules: vec![
                        matches[i - 1].2.id().to_string(),
                        matches[i].2.id().to_string(),
                    ],
                });
            }
        }

        // 後ろから置換し、前方の文字位置がずれないようにする
        let mut result = cell.to_string();
        for (start, end, rule) in matches.iter().rev() {
            result.replace_range(*start..*end, rule.replacement());
        }

        let matched: Vec<usize> = matches.iter().map(|(_, _, rule)| rule.id().index).collect();
        Ok((result, matched))
    }
}

impl RecordTransform for ReplaceTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // 列指定を解決済みインデックスへ変換する
        self.target = match &self.columns {
            ColumnTarget::All => TargetColumns::All,
            ColumnTarget::Specified(cols) => {
                TargetColumns::Indices(resolve_indices(cols, headers)?)
            }
        };
        self.headers = headers.cloned();
        // ヘッダー行は置換せずそのまま出力する
        Ok(headers.cloned())
    }

    fn on_record(&mut self, record: &mut StringRecord, row: u64) -> Result<(), CsvOpsError> {
        let row_matches = self.apply(record, row)?;

        // 統計更新
        if !row_matches.is_empty() {
            self.stats.rows_modified += 1;
        }
        self.stats.total_replacements += row_matches.len() as u64;

        // per_rule 集計:
        // - matches       = マッチ回数なので各マッチごとにカウント
        // - rows_affected = 影響行数なのでこの行で 1 回以上マッチしたルールを 1 回だけカウント
        let mut affected: HashSet<usize> = HashSet::new();
        for &idx in &row_matches {
            self.stats.per_rule[idx].matches += 1;
            affected.insert(idx);
        }
        for idx in affected {
            self.stats.per_rule[idx].rows_affected += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
