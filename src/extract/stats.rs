use serde::Serialize;

use crate::StatsReport;

/// extract 実行の統計
#[derive(Debug, Serialize)]
pub struct ExtractStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// ルール毎の統計 (ルール定義順)
    pub per_rule: Vec<ExtractRuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct ExtractRuleStat {
    /// 追加した列の名前
    pub out_col: String,
    /// このルールで 1 件以上抽出できた行数
    pub extracted_rows: u64,
}

impl ExtractStats {
    /// out_col のリストからカウンタ 0 で初期化する
    /// per_rule はルール定義順に並び、ルール index でそのまま添字アクセスできる
    pub fn new(out_cols: Vec<String>) -> Self {
        let per_rule = out_cols
            .into_iter()
            .map(|out_col| ExtractRuleStat {
                out_col,
                extracted_rows: 0,
            })
            .collect();
        Self {
            rows_processed: 0,
            per_rule,
        }
    }
}

impl StatsReport for ExtractStats {
    fn to_text(&self) -> String {
        let mut lines = vec![format!("処理行数: {}", self.rows_processed)];
        if !self.per_rule.is_empty() {
            lines.push("ルール別:".to_string());
            for r in &self.per_rule {
                lines.push(format!("  {}: 抽出 {} 件", r.out_col, r.extracted_rows));
            }
        }
        lines.join("\n")
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("ExtractStats は常にシリアライズできる")
    }
}
