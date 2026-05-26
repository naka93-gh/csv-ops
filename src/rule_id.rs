// ルール識別子
// エラーメッセージや統計出力でルールを特定するために使う

use std::fmt;

#[derive(Debug, Clone)]
pub struct RuleId {
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
