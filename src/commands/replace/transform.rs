use std::borrow::Cow;
use std::collections::HashSet;

use csv::StringRecord;

use crate::column::{build_index_mask, ensure_in_range, resolve_indices};
use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

use super::ColumnTarget;
use super::rule::CompiledRule;

/// 解決済みの置換対象列
/// init で ColumnTarget (列名 / 列番号) をインデックスへ解決した結果
pub enum TargetColumns {
    /// 全カラム横断
    All,
    /// 解決済みの対象列インデックス
    /// list: 順序保持 + 範囲チェック用、mask: O(1) lookup 用ビットマップ
    Indices { list: Vec<usize>, mask: Vec<bool> },
}

impl TargetColumns {
    /// 解決済みのインデックス列から TargetColumns::Indices を組み立てる
    /// list を保持しつつ、max+1 サイズの bool ビットマップで O(1) lookup できるようにする
    fn from_indices(list: Vec<usize>) -> Self {
        let mask = build_index_mask(&list);
        Self::Indices { list, mask }
    }

    /// col_index が置換対象かどうか (O(1))
    fn includes(&self, col_index: usize) -> bool {
        match self {
            TargetColumns::All => true,
            TargetColumns::Indices { mask, .. } => mask.get(col_index).copied().unwrap_or(false),
        }
    }
}

/// 置換処理本体
pub struct ReplaceTransform {
    /// compile 済みルール
    rules: Vec<CompiledRule>,
    /// 列指定の未解決スペック (init で target へ解決)
    columns: ColumnTarget,
    /// 解決済みの対象列
    target: TargetColumns,
    /// ヘッダー (エラーメッセージのカラム名表示に使う)
    headers: Option<StringRecord>,
    /// 処理統計
    pub stats: Stats,
}

impl ReplaceTransform {
    pub fn new(rules: Vec<CompiledRule>, columns: ColumnTarget, stats: Stats) -> Self {
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
        if let TargetColumns::Indices { list, .. } = &self.target {
            ensure_in_range(list.iter().copied(), record.len())?;
        }

        // record を一旦取り出して借用ベースで処理する
        // (csv::StringRecord は cell 単位の差し替えができないため最後に再構築)
        let owned = std::mem::take(record);
        let mut new_fields: Vec<Cow<'_, str>> = Vec::with_capacity(owned.len());
        let mut row_matches: Vec<usize> = Vec::new();

        for (col_index, field) in owned.iter().enumerate() {
            // 対象列でなければ借用のままで素通し (String alloc を避ける)
            if !self.target.includes(col_index) {
                new_fields.push(Cow::Borrowed(field));
                continue;
            }

            // エラーメッセージ用カラム名は衝突検知エラー時のみ format するよう遅延
            let column_name = || {
                self.headers
                    .as_ref()
                    .and_then(|h| h.get(col_index))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("col[{}]", col_index))
            };

            match self.apply_cell(field, row, column_name) {
                Ok((replaced, matched)) => {
                    row_matches.extend(matched);
                    new_fields.push(replaced);
                }
                Err(e) => {
                    // エラー時は record を元の状態に戻す
                    drop(new_fields);
                    *record = owned;
                    return Err(e);
                }
            }
        }

        *record = StringRecord::from(new_fields);
        Ok(row_matches)
    }

    /// セル単位の置換処理
    /// 戻り値は (置換後の文字列 (マッチなしなら借用), マッチしたルール index 列)
    fn apply_cell<'a, F>(
        &self,
        cell: &'a str,
        row: u64,
        column_name: F,
    ) -> Result<(Cow<'a, str>, Vec<usize>), TransformError>
    where
        F: FnOnce() -> String,
    {
        // 全ルールのマッチ位置を収集する
        // 単純置換・正規表現どちらも matcher().find_iter で非オーバーラップマッチを得る
        // 元の cell に対して全ルール評価するので、評価の連鎖はしない
        let mut matches: Vec<(usize, usize, &CompiledRule)> = Vec::new();
        for rule in &self.rules {
            for m in rule.matcher().find_iter(cell) {
                matches.push((m.start(), m.end(), rule));
            }
        }

        // マッチがなければ借用で返す (String allocation を回避)
        if matches.is_empty() {
            return Ok((Cow::Borrowed(cell), Vec::new()));
        }

        // 衝突検知と後ろからの置換のため、開始位置でソートする
        matches.sort_by_key(|(start, _, _)| *start);

        // 衝突検知: 前のマッチの end が次のマッチの start を越えていたら範囲が重なる
        for i in 1..matches.len() {
            if matches[i - 1].1 > matches[i].0 {
                return Err(TransformError::RuntimeCollision {
                    row,
                    column: column_name(),
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
        Ok((Cow::Owned(result), matched))
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
                TargetColumns::from_indices(resolve_indices(cols, headers)?)
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
            self.stats.rows_changed += 1;
        }
        self.stats.changes_total += row_matches.len() as u64;

        // per_rule 集計:
        // - matches       = マッチ回数なので各マッチごとにカウント
        // - rows_affected = 影響行数なのでこの行で 1 回以上マッチしたルールを 1 回だけカウント
        let mut affected: HashSet<usize> = HashSet::new();
        for &idx in &row_matches {
            *self.stats.per_rule[idx].matches.get_or_insert(0) += 1;
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
