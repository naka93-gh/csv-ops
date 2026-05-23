// 全サブコマンド共通の統計型
// mask/replace/flag/extract/similarity が同じ Stats を返し、CLI は emit_report 1 経路で出力する

use serde::Serialize;

use crate::StatsReport;

/// サブコマンド実行の統計 (全サブコマンド共通)
#[derive(Debug, Default, Serialize)]
pub struct Stats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// 1 つ以上ヒット (変更/マッチ/抽出) が入った行数
    pub rows_changed: u64,
    /// ヒットの総数 (mask: セル数 / replace: 置換回数 / 他: per_rule の rows_affected 合計)
    pub changes_total: u64,
    /// ルール毎の統計 (ルール定義順)。mask は空配列
    pub per_rule: Vec<RuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct RuleStat {
    /// ルール識別子 (replace: rule[N] "name" / 他: out_col)
    pub id: String,
    /// このルールでヒットした行数
    pub rows_affected: u64,
    /// マッチ総数 (replace のみ。1 行に複数回置換が入りうるため rows_affected と分離)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<u64>,
}

impl Stats {
    /// per_rule を id 列から 0 初期化
    pub fn with_rule_ids(ids: Vec<String>) -> Self {
        let per_rule = ids
            .into_iter()
            .map(|id| RuleStat {
                id,
                rows_affected: 0,
                matches: None,
            })
            .collect();
        Self {
            rows_processed: 0,
            rows_changed: 0,
            changes_total: 0,
            per_rule,
        }
    }
}

impl StatsReport for Stats {
    fn to_text(&self) -> String {
        let mut lines = vec![
            format!("処理行数: {}", self.rows_processed),
            format!("ヒット行数: {}", self.rows_changed),
            format!("ヒット総数: {}", self.changes_total),
        ];
        if !self.per_rule.is_empty() {
            lines.push("ルール別:".to_string());
            for r in &self.per_rule {
                let mut line = format!("  {}: ヒット {} 行", r.id, r.rows_affected);
                if let Some(m) = r.matches {
                    line.push_str(&format!(" / マッチ {} 件", m));
                }
                lines.push(line);
            }
        }
        lines.join("\n")
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Stats は常にシリアライズできる")
    }
}
