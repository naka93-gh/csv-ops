// Jaro / Jaro-Winkler 類似度
// 一致文字数と転置を評価し、先頭一致にボーナスを与える。短い固有名詞に強い
// 日本語を正しく扱うため char (Unicode スカラ) 単位で計算する

/// Jaro-Winkler 類似度 0.0-1.0
/// Jaro スコアに共通接頭辞 (最大 4 文字) のボーナスを加える
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let j = jaro(a, b);
    // 共通接頭辞の長さ (最大 4)
    let prefix = a
        .chars()
        .zip(b.chars())
        .take_while(|(x, y)| x == y)
        .count()
        .min(4);
    // p = 0.1 (Winkler の標準スケーリング係数)
    j + prefix as f64 * 0.1 * (1.0 - j)
}

/// Jaro 類似度 0.0-1.0
fn jaro(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    // 一致とみなす探索範囲
    let max_dist = (a.len().max(b.len()) / 2).saturating_sub(1);

    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];
    let mut matches = 0usize;

    // a の各文字について、範囲内の未使用な一致文字を b から探す
    for (i, &ca) in a.iter().enumerate() {
        let lo = i.saturating_sub(max_dist);
        let hi = (i + max_dist + 1).min(b.len());
        for j in lo..hi {
            if !b_matched[j] && b[j] == ca {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }

    // 転置数: 一致文字を順に並べ、位置がずれている数を数える
    let mut transpositions = 0usize;
    let mut k = 0usize;
    for (i, &ai) in a.iter().enumerate() {
        if a_matched[i] {
            while !b_matched[k] {
                k += 1;
            }
            if ai != b[k] {
                transpositions += 1;
            }
            k += 1;
        }
    }

    let m = matches as f64;
    let t = transpositions as f64 / 2.0;
    (m / a.len() as f64 + m / b.len() as f64 + (m - t) / m) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_one() {
        assert_eq!(jaro_winkler("東京都", "東京都"), 1.0);
    }

    #[test]
    fn both_empty_is_one() {
        assert_eq!(jaro_winkler("", ""), 1.0);
    }

    #[test]
    fn one_empty_is_zero() {
        assert_eq!(jaro_winkler("abc", ""), 0.0);
    }

    #[test]
    fn classic_martha_marhta() {
        // 既知の値: jaro-winkler("MARTHA","MARHTA") ≈ 0.961
        let s = jaro_winkler("MARTHA", "MARHTA");
        assert!((s - 0.9611).abs() < 1e-3, "got {}", s);
    }

    #[test]
    fn prefix_match_raises_score() {
        // 先頭一致ありの方がスコアが高い
        let with_prefix = jaro_winkler("東京都港区", "東京都中央区");
        let no_prefix = jaro_winkler("港区東京都", "中央区東京都");
        assert!(with_prefix > no_prefix);
    }
}
