// Sørensen-Dice 係数 (bigram ベース)
// 隣接 2 文字の集合の重なりで比較する。語順違い・部分一致に強い
// 日本語を正しく扱うため char (Unicode スカラ) 単位で計算する

use std::collections::HashMap;

/// Dice 係数による類似度 0.0-1.0
/// 2 * |共通 bigram| / (|bigram_a| + |bigram_b|)
pub fn dice_coefficient(a: &str, b: &str) -> f64 {
    // 完全一致は 1.0 (1 文字以下で bigram が作れないケースも拾う)
    if a == b {
        return 1.0;
    }
    let ba = bigrams(a);
    let bb = bigrams(b);
    // 一方でも bigram が無ければ (1 文字以下) 一致しない
    if ba.is_empty() || bb.is_empty() {
        return 0.0;
    }

    // bigram は多重集合として扱う。a 側の出現数を数える
    let mut counts: HashMap<(char, char), i32> = HashMap::new();
    for g in &ba {
        *counts.entry(*g).or_insert(0) += 1;
    }
    // b 側で消し込み、共通要素数を数える
    let mut intersection = 0usize;
    for g in &bb {
        if let Some(c) = counts.get_mut(g)
            && *c > 0
        {
            *c -= 1;
            intersection += 1;
        }
    }

    2.0 * intersection as f64 / (ba.len() + bb.len()) as f64
}

/// 文字列の隣接 2 文字ペア (bigram) を列挙する
fn bigrams(s: &str) -> Vec<(char, char)> {
    let chars: Vec<char> = s.chars().collect();
    chars.windows(2).map(|w| (w[0], w[1])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_one() {
        assert_eq!(dice_coefficient("東京都", "東京都"), 1.0);
    }

    #[test]
    fn both_empty_is_one() {
        assert_eq!(dice_coefficient("", ""), 1.0);
    }

    #[test]
    fn single_char_only_exact_match() {
        assert_eq!(dice_coefficient("x", "x"), 1.0);
        assert_eq!(dice_coefficient("x", "y"), 0.0);
    }

    #[test]
    fn classic_night_nacht() {
        // 既知の値: dice("night","nacht") = 0.25 (共通 bigram は "ht" のみ)
        assert!((dice_coefficient("night", "nacht") - 0.25).abs() < 1e-9);
    }

    #[test]
    fn disjoint_is_zero() {
        assert_eq!(dice_coefficient("abcd", "wxyz"), 0.0);
    }

    #[test]
    fn partial_overlap_is_between() {
        let s = dice_coefficient("東京都港区", "東京都");
        assert!(s > 0.0 && s < 1.0);
    }
}
