use std::collections::HashSet;

use csv::StringRecord;

use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

use super::config::ExtractConfig;
use super::rule::CompiledExtractRule;

/// extract 抽出処理本体
/// ルールの compile (正規表現 + 列解決) はヘッダーが要るため init で行う
pub(crate) struct ExtractTransform {
    /// 設定 (init で compile する)
    config: ExtractConfig,
    /// compile 済みルール (init 後に確定)
    compiled: Vec<CompiledExtractRule>,
    /// 処理統計 (init で out_col 確定後に作る)
    pub stats: Stats,
}

impl ExtractTransform {
    pub fn new(config: ExtractConfig) -> Self {
        Self {
            config,
            compiled: Vec::new(),
            stats: Stats::default(),
        }
    }
}

impl RecordTransform for ExtractTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // ルールを compile (正規表現 compile + 対象列をヘッダー照合でインデックス解決)
        self.compiled = self.config.compile_rules(headers)?;

        // out_col 衝突検査 (ヘッダー有り時のみ)
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

        // 既存フィールドをコピーし、ルール毎の抽出結果を push して伸ばす
        let mut fields: Vec<String> = record.iter().map(|f| f.to_string()).collect();

        // この行で 1 つ以上ヒットしたかを追跡 (rows_changed の集計用)
        let mut row_hit = false;

        for (i, rule) in self.compiled.iter().enumerate() {
            // ヘッダー無し + 列番号指定では compile 時に範囲チェックできないため、行ごとに検証する
            if rule.column >= original_len {
                return Err(TransformError::IndexOutOfRange {
                    index: rule.column,
                    columns: original_len,
                }
                .into());
            }

            // 対象列の全マッチを取り出す
            // キャプチャグループがあれば 1 番目のグループ、なければマッチ全体を採用する
            let matches: Vec<String> = rule
                .pattern
                .captures_iter(&fields[rule.column])
                .map(|caps| {
                    caps.get(1)
                        .or_else(|| caps.get(0))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                })
                .collect();

            // マッチなしは空文字、複数マッチは separator 連結
            if !matches.is_empty() {
                self.stats.per_rule[i].rows_affected += 1;
                self.stats.changes_total += 1;
                row_hit = true;
            }
            fields.push(matches.join(&rule.separator));
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
