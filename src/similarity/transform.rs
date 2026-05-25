use std::collections::HashSet;

use csv::StringRecord;

use crate::error::{CsvOpsError, TransformError};
use crate::pipeline::RecordTransform;
use crate::stats::Stats;

use super::config::SimilarityConfig;
use super::rule::CompiledSimilarityRule;

/// しきい値未満のときに matched_name 列へ出力する値
const NO_MATCH: &str = "<no match>";

/// similarity マッチ処理本体
/// ルールの compile (列解決 + 辞書ロード) はヘッダーが要るため init で行う
pub(crate) struct SimilarityTransform {
    /// 設定 (init で compile する)
    config: SimilarityConfig,
    /// CSV 形式辞書を読むときの区切り文字 (本体 CSV と共通)
    delimiter: u8,
    /// compile 済みルール (init 後に確定)
    compiled: Vec<CompiledSimilarityRule>,
    /// 処理統計
    pub stats: Stats,
}

impl SimilarityTransform {
    pub fn new(config: SimilarityConfig, delimiter: u8) -> Self {
        Self {
            config,
            delimiter,
            compiled: Vec::new(),
            stats: Stats::default(),
        }
    }
}

impl RecordTransform for SimilarityTransform {
    fn init(
        &mut self,
        headers: Option<&StringRecord>,
    ) -> Result<Option<StringRecord>, CsvOpsError> {
        // ルールを compile (列解決 + 正規化セット構築 + 辞書ロード)
        self.compiled = self.config.compile_rules(headers, self.delimiter)?;

        // out_col / score_col の衝突検査 (ヘッダー有り時のみ)
        if let Some(h) = headers {
            let mut seen: HashSet<String> = h.iter().map(|s| s.to_string()).collect();
            for rule in &self.compiled {
                if !seen.insert(rule.out_col.clone()) {
                    return Err(TransformError::OutputColumnConflict {
                        name: rule.out_col.clone(),
                    }
                    .into());
                }
                if !seen.insert(rule.score_col.clone()) {
                    return Err(TransformError::OutputColumnConflict {
                        name: rule.score_col.clone(),
                    }
                    .into());
                }
            }
        }

        // 統計を out_col 一覧で初期化する (per_rule は ID 列で 0 初期化)
        let out_cols: Vec<String> = self.compiled.iter().map(|r| r.out_col.clone()).collect();
        self.stats = Stats::with_rule_ids(out_cols);

        // ヘッダーがあれば既存カラム + ルール毎の out_col / score_col を出力ヘッダーとする
        match headers {
            Some(h) => {
                let mut extended: Vec<String> = h.iter().map(|s| s.to_string()).collect();
                for rule in &self.compiled {
                    extended.push(rule.out_col.clone());
                    extended.push(rule.score_col.clone());
                }
                Ok(Some(StringRecord::from(extended)))
            }
            None => Ok(None),
        }
    }

    fn on_record(&mut self, record: &mut StringRecord, row: u64) -> Result<(), CsvOpsError> {
        // 範囲チェックの基準に使うので、列追加前のフィールド数を取得
        let original_len = record.len();

        // 既存フィールドをコピーし、ルール毎の結果 (matched_name, score) を push して伸ばす
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

            // 対象セルを正規化し、辞書とベストマッチを取る
            let normalized = rule.normalize.apply(&fields[rule.column]);
            let result = rule.dict.best_match(&normalized, rule.algorithm);

            // 同点は辞書記述順で先勝ち。警告を stderr へ出す (集計はしない)
            if result.tie {
                eprintln!(
                    "警告: 行 {} ({}) で同点マッチ。辞書記述順で先勝ちしました",
                    row, rule.out_col
                );
            }

            // しきい値以上なら canonical、未満なら <no match>。score は実値を出力
            if result.score >= rule.threshold {
                self.stats.per_rule[i].rows_affected += 1;
                self.stats.changes_total += 1;
                row_hit = true;
                fields.push(result.canonical);
            } else {
                fields.push(NO_MATCH.to_string());
            }
            fields.push(format!("{:.4}", result.score));
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
