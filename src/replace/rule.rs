// compile 済みルールの内部表現
// 単純置換と正規表現を enum で統一、衝突検出と置換実行の両方で使う

use crate::rule_id::RuleId;

/// compile 済みのルール
/// 単純置換・正規表現どちらもマッチ判定は正規表現に統一している
/// 単純置換は from を `regex::escape` でエスケープしてコンパイルするため、
/// case_insensitive の扱いも正規表現側に一元化され、文字位置のズレが起きない
#[derive(Debug)]
pub enum CompiledRule {
    /// 単純文字列置換 (部分一致)
    Simple {
        id: RuleId,
        /// from をエスケープしてコンパイルしたマッチャ
        matcher: regex::Regex,
        /// 元の検索文字列 (静的衝突検出で使う)
        from: String,
        /// 置換後の文字列
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
            CompiledRule::Simple { id, .. } | CompiledRule::Regex { id, .. } => id,
        }
    }

    /// マッチ位置の列挙に使う正規表現
    pub fn matcher(&self) -> &regex::Regex {
        match self {
            CompiledRule::Simple { matcher, .. } => matcher,
            CompiledRule::Regex { pattern, .. } => pattern,
        }
    }

    /// マッチ部分を置き換える文字列
    pub fn replacement(&self) -> &str {
        match self {
            CompiledRule::Simple { to, .. } => to,
            CompiledRule::Regex { replacement, .. } => replacement,
        }
    }
}
