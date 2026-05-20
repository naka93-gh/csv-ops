// detect_static_collisions の単体テスト

use super::*;
use crate::replace::rule::RuleId;

fn simple(index: usize, from: &str) -> CompiledRule {
    CompiledRule::Simple {
        id: RuleId { index, name: None },
        from: from.to_string(),
        to: String::new(),
    }
}

/// 衝突がなければ Ok
#[test]
fn no_collision() {
    let rules = vec![simple(0, "abc"), simple(1, "xyz")];
    assert!(detect_static_collisions(&rules, false).is_ok());
}

/// 部分文字列関係を検出する
#[test]
fn detects_substring_relation() {
    let rules = vec![simple(0, "対応"), simple(1, "対応中")];
    let err = detect_static_collisions(&rules, false).unwrap_err();
    match err {
        ConfigError::RuleCollision { reason, .. } => assert_eq!(reason, "部分文字列関係"),
        other => panic!("RuleCollision を期待: {:?}", other),
    }
}

/// 完全重複を検出する
#[test]
fn detects_exact_duplicate() {
    let rules = vec![simple(0, "abc"), simple(1, "abc")];
    let err = detect_static_collisions(&rules, false).unwrap_err();
    match err {
        ConfigError::RuleCollision { reason, .. } => assert_eq!(reason, "完全重複"),
        other => panic!("RuleCollision を期待: {:?}", other),
    }
}

/// case_insensitive 時は大小違いの重複も検出する
#[test]
fn case_insensitive_duplicate() {
    let rules = vec![simple(0, "ABC"), simple(1, "abc")];
    assert!(detect_static_collisions(&rules, true).is_err());
    // case-sensitive ならこの 2 つは衝突しない
    assert!(detect_static_collisions(&rules, false).is_ok());
}
