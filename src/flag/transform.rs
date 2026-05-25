use csv::StringRecord;

use crate::column::{check_output_conflicts, ensure_in_range};
use crate::error::CsvOpsError;
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

use super::config::FlagConfig;
use super::rule::CompiledFlagRule;

/// flag 判定処理本体
/// ルールの compile (正規表現 + 列解決) はヘッダーが要るため init で行う
pub struct FlagTransform {
    /// 設定 (init で compile する)
    config: FlagConfig,
    /// compile 済みルール (init 後に確定)
    compiled: Vec<CompiledFlagRule>,
    /// 処理統計 (init で out_col 確定後に作る)
    pub stats: Stats,
}

impl FlagTransform {
    pub fn new(config: FlagConfig) -> Self {
        Self {
            config,
            compiled: Vec::new(),
            stats: Stats::default(),
        }
    }
}

impl RecordTransform for FlagTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // ルールを compile (正規表現 compile + 対象列をヘッダー照合でインデックス解決)
        self.compiled = self.config.compile_rules(headers)?;

        // out_col 衝突検査 (ヘッダー有り時のみ既存カラム名 + ルール間の重複をまとめて検出)
        check_output_conflicts(headers, self.compiled.iter().map(|r| r.out_col.as_str()))?;

        // 統計を out_col 一覧で初期化する (per_rule は ID 列で 0 初期化)
        let out_cols: Vec<String> = self.compiled.iter().map(|r| r.out_col.clone()).collect();
        self.stats = Stats::with_rule_ids(out_cols.clone());

        // ヘッダーがあれば既存カラム + 各ルールの out_col を出力ヘッダーとする
        match headers {
            Some(h) => {
                let mut extended: Vec<String> = h.iter().map(|s| s.to_string()).collect();
                extended.extend(out_cols);
                Ok(Some(StringRecord::from(extended)))
            }
            None => Ok(None),
        }
    }

    fn on_record(&mut self, record: &mut StringRecord, _row: u64) -> Result<(), CsvOpsError> {
        // 範囲チェックの基準に使うので、列追加前のフィールド数を取得
        let original_len = record.len();

        // 既存フィールドをコピーし、ルール毎の判定結果を push して伸ばす
        let mut fields: Vec<String> = record.iter().map(|f| f.to_string()).collect();

        // この行で 1 つ以上ヒットしたかを追跡 (rows_changed の集計用)
        let mut row_hit = false;

        for (i, rule) in self.compiled.iter().enumerate() {
            ensure_in_range(rule.columns.iter().copied(), original_len)?;

            // ルール内の対象列のうち 1 つでもマッチすれば true
            let matched = rule
                .columns
                .iter()
                .any(|&c| rule.pattern.is_match(&fields[c]));

            fields.push(if matched {
                rule.true_value.clone()
            } else {
                rule.false_value.clone()
            });
            if matched {
                self.stats.per_rule[i].rows_affected += 1;
                self.stats.changes_total += 1;
                row_hit = true;
            }
        }

        if row_hit {
            self.stats.rows_changed += 1;
        }

        *record = StringRecord::from(fields);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
