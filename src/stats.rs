// 全サブコマンド共通の統計型
// mask/replace/flag/extract/similarity が同じ Stats を返し、CLI は emit_report 1 経路で出力する

use serde::Serialize;

use crate::stats_report::StatsReport;

/// サブコマンド実行の統計 (全サブコマンド共通)
#[derive(Debug, Default, Serialize)]
pub struct Stats {
    /// 処理した行数 (has_headers=true ならヘッダー除く、convert は全行を含む)
    pub rows_processed: u64,
    /// 1 つ以上ヒット (変更/マッチ/抽出) が入った行数
    pub rows_changed: u64,
    /// ヒットの総数 (mask: セル数 / replace: 置換回数 / 他: per_rule の rows_affected 合計)
    pub changes_total: u64,
    /// ルール毎の統計 (ルール定義順)。mask / convert は空配列
    pub per_rule: Vec<RuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct RuleStat {
    /// ルール識別子 (replace: rule[N] "name" / flag / extract / similarity: out_col)
    pub id: String,
    /// このルールでヒットした行数
    pub rows_affected: u64,
    /// マッチ総数 (replace のみ Some。1 行に複数回置換が入りうるため rows_affected と別途集計する)
    /// flag / extract / similarity は 1 行 1 判定なので None を入れる (rows_affected と同値になるため)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<u64>,
}

impl Stats {
    /// per_rule を id 列から 0 初期化する
    /// replace では rule[N] "name" 形式、flag / extract / similarity では out_col を ID として渡す
    pub fn with_rule_ids<I: IntoIterator<Item = String>>(ids: I) -> Self {
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
}

#[cfg(test)]
mod tests;
