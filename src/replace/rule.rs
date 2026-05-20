// compile 済みルールの内部表現
// 単純置換と正規表現を enum で統一、衝突検出と置換実行の両方で使う

use std::fmt;

/// ルール識別子
/// エラーメッセージや統計出力でルールを特定するために使う
#[derive(Debug, Clone)]
pub(crate) struct RuleId {
    pub index: usize,
    pub name: Option<String>,
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "rule[{}] \"{}\"", self.index, name),
            None => write!(f, "rule[{}]", self.index),
        }
    }
}

/// compile 済みのルール
/// case_insensitive オプションは ReplaceTransform 側で持ち、ここでは個別のルール定義のみ
#[derive(Debug)]
pub(crate) enum CompiledRule {
    /// 単純文字列置換 (部分一致)
    Simple {
        id: RuleId,
        from: String,
        to: String,
    },
    /// 正規表現置換
    Regex {
        id: RuleId,
        pattern: regex::Regex,
        replacement: String,
    },
}

impl CompiledRule {
    pub fn id(&self) -> &RuleId {
        match self {
            CompiledRule::Simple { id, .. } => id,
            CompiledRule::Regex { id, .. } => id,
        }
    }
}
