/// extract 実行の統計
#[derive(Debug)]
pub struct ExtractStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// ルール毎の統計 (ルール定義順)
    pub per_rule: Vec<ExtractRuleStat>,
}

/// 1 ルール分の統計
#[derive(Debug)]
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

    /// テキスト形式でフォーマットする
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("処理行数: {}\n", self.rows_processed));
        if !self.per_rule.is_empty() {
            out.push_str("ルール別:\n");
            for r in &self.per_rule {
                out.push_str(&format!("  {}: 抽出 {} 件\n", r.out_col, r.extracted_rows));
            }
        }
        out
    }

    /// JSON 形式でフォーマットする
    /// 依存を増やさないため手書き。文字列はエスケープして埋め込む
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"rows_processed\": {},\n", self.rows_processed));
        out.push_str("  \"per_rule\": [");
        for (i, r) in self.per_rule.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("\n    {");
            out.push_str(&format!("\"out_col\": \"{}\", ", escape_json(&r.out_col)));
            out.push_str(&format!("\"extracted_rows\": {}", r.extracted_rows));
            out.push('}');
        }
        if self.per_rule.is_empty() {
            out.push_str("]\n");
        } else {
            out.push_str("\n  ]\n");
        }
        out.push('}');
        out
    }
}

/// JSON 文字列リテラル用の最小エスケープ
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}
