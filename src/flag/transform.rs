use std::collections::HashSet;

use csv::StringRecord;

use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;

use super::config::FlagConfig;
use super::rule::CompiledFlagRule;
use super::stats::FlagStats;

/// flag 判定処理本体
/// ルールの compile (正規表現 + 列解決) はヘッダーが要るため init で行う
pub(crate) struct FlagTransform {
    /// 設定 (init で compile する)
    config: FlagConfig,
    /// compile 済みルール (init 後に確定)
    compiled: Vec<CompiledFlagRule>,
    /// 処理統計 (init で out_col 確定後に作る)
    pub stats: FlagStats,
}

impl FlagTransform {
    pub fn new(config: FlagConfig) -> Self {
        Self {
            config,
            compiled: Vec::new(),
            stats: FlagStats::new(Vec::new()),
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

        // out_col 衝突検査 (ヘッダー有り時のみ)
        // 既存カラム名との衝突と、ルール間の out_col 重複を 1 つの集合でまとめて検出する
        if let Some(h) = headers {
            let mut seen: HashSet<String> = h.iter().map(|s| s.to_string()).collect();
            for rule in &self.compiled {
                if !seen.insert(rule.out_col.clone()) {
                    return Err(TransformError::OutputColumnConflict {
                        name: rule.out_col.clone(),
                    }
                    .into());
                }
            }
        }

        // 統計を out_col 一覧で初期化する
        let out_cols: Vec<String> = self.compiled.iter().map(|r| r.out_col.clone()).collect();
        self.stats = FlagStats::new(out_cols.clone());

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

        for (i, rule) in self.compiled.iter().enumerate() {
            // ヘッダー無し + 列番号指定では compile 時に範囲チェックできないため、行ごとに検証する
            for &c in &rule.columns {
                if c >= original_len {
                    return Err(TransformError::IndexOutOfRange {
                        index: c,
                        columns: original_len,
                    }
                    .into());
                }
            }

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
                self.stats.per_rule[i].matched_rows += 1;
            }
        }

        *record = StringRecord::from(fields);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
