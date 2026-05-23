use serde::Serialize;

use crate::StatsReport;

/// replace 実行の統計
#[derive(Debug, Serialize)]
pub struct ReplaceStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// 置換が入った行数
    pub rows_modified: u64,
    /// 総置換回数 (全ルール・全セルのマッチ数合計)
    pub total_replacements: u64,
    /// ルール毎の統計 (ルール定義順)
    pub per_rule: Vec<RuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct RuleStat {
    /// ルール識別子 (rule[N] "name" 形式)
    pub rule_id: String,
    /// マッチ回数 (このルールがマッチした総数)
    pub matches: u64,
    /// 影響行数 (このルールが 1 回以上マッチした行数)
    pub rows_affected: u64,
}

impl ReplaceStats {
    /// ルール ID のリストからカウンタ 0 で初期化する
    /// per_rule はルール定義順に並び、ルール index でそのまま添字アクセスできる
    pub fn new(rule_ids: Vec<String>) -> Self {
        let per_rule = rule_ids
            .into_iter()
            .map(|rule_id| RuleStat {
                rule_id,
                matches: 0,
                rows_affected: 0,
            })
            .collect();
        Self {
            rows_processed: 0,
            rows_modified: 0,
            total_replacements: 0,
            per_rule,
        }
    }
}

impl StatsReport for ReplaceStats {
    fn to_text(&self) -> String {
        let mut lines = vec![
            format!("処理行数:   {}", self.rows_processed),
            format!("変更行数:   {}", self.rows_modified),
            format!("総置換回数: {}", self.total_replacements),
        ];
        if !self.per_rule.is_empty() {
            lines.push("ルール別:".to_string());
            for r in &self.per_rule {
                lines.push(format!(
                    "  {}: マッチ {}, 影響行数 {}",
                    r.rule_id, r.matches, r.rows_affected
                ));
            }
        }
        lines.join("\n")
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("ReplaceStats は常にシリアライズできる")
    }
}
