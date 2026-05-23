use serde::Serialize;

/// similarity 実行の統計
#[derive(Debug, Serialize)]
pub struct SimilarityStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// ルール毎の統計 (ルール定義順)
    pub per_rule: Vec<SimilarityRuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug, Serialize)]
pub struct SimilarityRuleStat {
    /// マッチ名を出力する列名
    pub out_col: String,
    /// しきい値以上でマッチした行数
    pub matched_rows: u64,
    /// しきい値未満 (<no match>) の行数
    pub no_match_rows: u64,
    /// 同点が発生した行数
    pub tie_rows: u64,
}

impl SimilarityStats {
    /// out_col のリストからカウンタ 0 で初期化する
    /// per_rule はルール定義順に並び、ルール index でそのまま添字アクセスできる
    pub fn new(out_cols: Vec<String>) -> Self {
        let per_rule = out_cols
            .into_iter()
            .map(|out_col| SimilarityRuleStat {
                out_col,
                matched_rows: 0,
                no_match_rows: 0,
                tie_rows: 0,
            })
            .collect();
        Self {
            rows_processed: 0,
            per_rule,
        }
    }

    /// テキスト形式でフォーマットする
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("処理行数: {}\n", self.rows_processed));
        if !self.per_rule.is_empty() {
            out.push_str("ルール別:\n");
            for r in &self.per_rule {
                out.push_str(&format!(
                    "  {}: マッチ {} 件 / 非マッチ {} 件 (同点 {} 件)\n",
                    r.out_col, r.matched_rows, r.no_match_rows, r.tie_rows
                ));
            }
        }
        out
    }

    /// JSON 形式でフォーマットする
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("SimilarityStats は常にシリアライズできる")
    }
}
