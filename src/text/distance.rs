// 編集距離系アルゴリズム (Levenshtein / Damerau-Levenshtein OSA) と類似度
// 日本語を正しく扱うため char (Unicode スカラ) 単位で計算する

/// 2 文字列のレーベンシュタイン距離 (char 単位、2 行 DP)
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    // prev が直前行、curr が計算中の行
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1) // 削除
                .min(curr[j] + 1) // 挿入
                .min(prev[j] + cost); // 置換
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Damerau-Levenshtein 距離 (OSA 変種、char 単位)
/// Levenshtein に加えて隣接 2 文字の転置を 1 操作として数える
/// OSA (Optimal String Alignment) は同じ部分文字列を二重に編集しない制限版
pub fn damerau_osa(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    // OSA は d[i-2][j-2] を参照するため全体の表を持つ
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut val = (d[i - 1][j] + 1) // 削除
                .min(d[i][j - 1] + 1) // 挿入
                .min(d[i - 1][j - 1] + cost); // 置換
            // 隣接転置 (ab → ba)
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                val = val.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = val;
        }
    }
    d[n][m]
}

/// レーベンシュタイン距離による類似度 0.0-1.0
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    edit_similarity(a, b, levenshtein(a, b))
}

/// Damerau-Levenshtein (OSA) による類似度 0.0-1.0
pub fn damerau_similarity(a: &str, b: &str) -> f64 {
    edit_similarity(a, b, damerau_osa(a, b))
}

/// 編集距離を類似度へ変換する。1 - 距離 / 長い方の文字数。両方空なら 1.0
fn edit_similarity(a: &str, b: &str, distance: usize) -> f64 {
    let max = a.chars().count().max(b.chars().count());
    if max == 0 {
        return 1.0;
    }
    1.0 - distance as f64 / max as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_counts_edits() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("東京", "東京都"), 1);
        assert_eq!(levenshtein("東京都", "東京都"), 0);
    }

    #[test]
    fn levenshtein_with_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn damerau_counts_transposition_as_one() {
        // 隣接転置は Levenshtein なら 2、Damerau なら 1
        assert_eq!(levenshtein("ab", "ba"), 2);
        assert_eq!(damerau_osa("ab", "ba"), 1);
        assert_eq!(damerau_osa("東京", "京東"), 1);
    }

    #[test]
    fn damerau_matches_levenshtein_without_transposition() {
        assert_eq!(damerau_osa("kitten", "sitting"), 3);
        assert_eq!(damerau_osa("", "abc"), 3);
    }

    #[test]
    fn similarity_of_identical_is_one() {
        assert_eq!(levenshtein_similarity("大阪府", "大阪府"), 1.0);
        assert_eq!(damerau_similarity("大阪府", "大阪府"), 1.0);
    }

    #[test]
    fn similarity_of_both_empty_is_one() {
        assert_eq!(levenshtein_similarity("", ""), 1.0);
        assert_eq!(damerau_similarity("", ""), 1.0);
    }

    #[test]
    fn similarity_is_between_zero_and_one() {
        let s = levenshtein_similarity("東京", "東京都");
        assert!((s - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn similarity_of_disjoint_is_zero() {
        assert_eq!(levenshtein_similarity("abc", "xyz"), 0.0);
    }
}
