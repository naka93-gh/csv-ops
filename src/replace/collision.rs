use crate::error::ConfigError;

use super::rule::CompiledRule;

/// 単純置換ルール間の静的衝突を検出する
pub fn detect_static_collisions(
    rules: &[CompiledRule],
    case_insensitive: bool,
) -> Result<(), ConfigError> {
    // ルールの中から単純置換ルールだけ抽出する
    let simples: Vec<(String, String)> = rules
        .iter()
        .filter_map(|r| match r {
            CompiledRule::Simple { id, from, .. } => {
                let key = if case_insensitive {
                    from.to_lowercase()
                } else {
                    from.clone()
                };
                Some((id.to_string(), key))
            }
            CompiledRule::Regex { .. } => None,
        })
        .collect();

    // 総当たりで衝突チェック
    // TODO: ルール数が少なければ問題になりづらいが、多くなると処理時間が急激に増すので要注意
    for i in 0..simples.len() {
        for j in (i + 1)..simples.len() {
            let (id_a, from_a) = &simples[i];
            let (id_b, from_b) = &simples[j];

            // 完全重複
            // TODO: エラーメッセージ用に完全一致を作っているが、不要かもしれない
            if from_a == from_b {
                return Err(ConfigError::RuleCollision {
                    rules: vec![id_a.clone(), id_b.clone()],
                    reason: "完全重複".to_string(),
                });
            }
            // 部分一致
            if from_a.contains(from_b.as_str()) || from_b.contains(from_a.as_str()) {
                return Err(ConfigError::RuleCollision {
                    rules: vec![id_a.clone(), id_b.clone()],
                    reason: "部分文字列関係".to_string(),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
