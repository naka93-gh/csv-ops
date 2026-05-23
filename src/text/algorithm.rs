// 類似度アルゴリズムの選択

use crate::error::ConfigError;

use super::{dice, distance, jaro};

/// 文字列類似度アルゴリズム
/// いずれも 0.0-1.0 のスコアを返す
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// レーベンシュタイン距離ベース (既定)
    #[default]
    Levenshtein,
    /// Damerau-Levenshtein (OSA)。隣接転置に強い
    Damerau,
    /// Jaro-Winkler。先頭一致を重視、短い固有名詞に強い
    JaroWinkler,
    /// Sørensen-Dice (bigram)。語順違い・部分一致に強い
    Dice,
}

impl Algorithm {
    /// アルゴリズム名から解決する。未知の名前はエラー
    pub fn parse(name: &str) -> Result<Self, ConfigError> {
        match name {
            "levenshtein" => Ok(Algorithm::Levenshtein),
            "damerau" => Ok(Algorithm::Damerau),
            "jaro-winkler" => Ok(Algorithm::JaroWinkler),
            "dice" => Ok(Algorithm::Dice),
            other => Err(ConfigError::Validation(format!(
                "未知の類似度アルゴリズム: {} (levenshtein / damerau / jaro-winkler / dice)",
                other
            ))),
        }
    }

    /// 2 文字列の類似度 0.0-1.0 を計算する
    pub fn similarity(self, a: &str, b: &str) -> f64 {
        match self {
            Algorithm::Levenshtein => distance::levenshtein_similarity(a, b),
            Algorithm::Damerau => distance::damerau_similarity(a, b),
            Algorithm::JaroWinkler => jaro::jaro_winkler(a, b),
            Algorithm::Dice => dice::dice_coefficient(a, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_names() {
        assert_eq!(
            Algorithm::parse("levenshtein").unwrap(),
            Algorithm::Levenshtein
        );
        assert_eq!(Algorithm::parse("damerau").unwrap(), Algorithm::Damerau);
        assert_eq!(
            Algorithm::parse("jaro-winkler").unwrap(),
            Algorithm::JaroWinkler
        );
        assert_eq!(Algorithm::parse("dice").unwrap(), Algorithm::Dice);
    }

    #[test]
    fn rejects_unknown_name() {
        assert!(Algorithm::parse("cosine").is_err());
    }

    #[test]
    fn default_is_levenshtein() {
        assert_eq!(Algorithm::default(), Algorithm::Levenshtein);
    }

    #[test]
    fn each_algorithm_scores_identical_as_one() {
        for algo in [
            Algorithm::Levenshtein,
            Algorithm::Damerau,
            Algorithm::JaroWinkler,
            Algorithm::Dice,
        ] {
            assert_eq!(algo.similarity("東京都", "東京都"), 1.0);
        }
    }
}
