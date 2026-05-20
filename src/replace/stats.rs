/// replace 実行の統計
#[derive(Debug)]
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
#[derive(Debug)]
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

    /// テキスト形式でフォーマットする
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("処理行数:   {}\n", self.rows_processed));
        out.push_str(&format!("変更行数:   {}\n", self.rows_modified));
        out.push_str(&format!("総置換回数: {}\n", self.total_replacements));
        if !self.per_rule.is_empty() {
            out.push_str("ルール別:\n");
            for r in &self.per_rule {
                out.push_str(&format!(
                    "  {}: マッチ {}, 影響行数 {}\n",
                    r.rule_id, r.matches, r.rows_affected
                ));
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
        out.push_str(&format!("  \"rows_modified\": {},\n", self.rows_modified));
        out.push_str(&format!(
            "  \"total_replacements\": {},\n",
            self.total_replacements
        ));
        out.push_str("  \"per_rule\": [");
        for (i, r) in self.per_rule.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("\n    {");
            out.push_str(&format!("\"rule\": \"{}\", ", escape_json(&r.rule_id)));
            out.push_str(&format!("\"matches\": {}, ", r.matches));
            out.push_str(&format!("\"rows_affected\": {}", r.rows_affected));
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
