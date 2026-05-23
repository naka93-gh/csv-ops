use serde::Serialize;

use crate::StatsReport;

/// flag 実行の統計
#[derive(Debug, Serialize)]
pub struct FlagStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// ルール毎の統計 (ルール定義順)
    pub per_rule: Vec<FlagRuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct FlagRuleStat {
    /// 追加した列の名前
    pub out_col: String,
    /// このルールが true になった行数
    pub matched_rows: u64,
}

impl FlagStats {
    /// out_col のリストからカウンタ 0 で初期化する
    /// per_rule はルール定義順に並び、ルール index でそのまま添字アクセスできる
    pub fn new(out_cols: Vec<String>) -> Self {
        let per_rule = out_cols
            .into_iter()
            .map(|out_col| FlagRuleStat {
                out_col,
                matched_rows: 0,
            })
            .collect();
        Self {
            rows_processed: 0,
            per_rule,
        }
    }
}

impl StatsReport for FlagStats {
    fn to_text(&self) -> String {
        let mut lines = vec![format!("処理行数: {}", self.rows_processed)];
        if !self.per_rule.is_empty() {
            lines.push("ルール別:".to_string());
            for r in &self.per_rule {
                lines.push(format!("  {}: true {} 件", r.out_col, r.matched_rows));
            }
        }
        lines.join("\n")
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("FlagStats は常にシリアライズできる")
    }
}
