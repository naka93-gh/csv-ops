// ReplaceTransform の単体テスト

use super::*;
use crate::replace::rule::RuleId;

fn simple(index: usize, from: &str, to: &str) -> CompiledRule {
    CompiledRule::Simple {
        id: RuleId { index, name: None },
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn regex_rule(index: usize, pattern: &str, replacement: &str) -> CompiledRule {
    CompiledRule::Regex {
        id: RuleId { index, name: None },
        pattern: regex::Regex::new(pattern).unwrap(),
        replacement: replacement.to_string(),
    }
}

fn rec(fields: &[&str]) -> StringRecord {
    StringRecord::from(fields.to_vec())
}

/// 単純置換が対象セルに適用される
#[test]
fn applies_simple_replacement() {
    let t = ReplaceTransform::new(vec![simple(0, "未対応", "open")], false);
    let mut r = rec(&["未対応", "データ"]);
    let matched = t
        .apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "open");
    assert_eq!(&r[1], "データ");
    assert_eq!(matched, vec![0]);
}

/// 正規表現置換が適用される
#[test]
fn applies_regex_replacement() {
    let t = ReplaceTransform::new(vec![regex_rule(0, r"\d+", "N")], false);
    let mut r = rec(&["abc123def"]);
    t.apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "abcNdef");
}

/// case_insensitive で大文字小文字を無視してマッチする
#[test]
fn case_insensitive_matches() {
    let t = ReplaceTransform::new(vec![simple(0, "abc", "X")], true);
    let mut r = rec(&["ABC"]);
    t.apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "X");
}

/// マッチなしの行は変更されず、マッチ列も空
#[test]
fn no_match_leaves_unchanged() {
    let t = ReplaceTransform::new(vec![simple(0, "xxx", "yyy")], false);
    let mut r = rec(&["abc", "def"]);
    let matched = t
        .apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "abc");
    assert!(matched.is_empty());
}

/// 範囲が重なるルールは動的衝突エラーになる
#[test]
fn overlapping_rules_collide() {
    let t = ReplaceTransform::new(
        vec![simple(0, "未対応", "X"), simple(1, "対応", "Y")],
        false,
    );
    let mut r = rec(&["未対応"]);
    let err = t
        .apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap_err();
    assert!(matches!(err, TransformError::RuntimeCollision { .. }));
}

/// 列指定 (Indices) で対象外の列は置換されない
#[test]
fn target_columns_limits_scope() {
    let t = ReplaceTransform::new(vec![simple(0, "a", "X")], false);
    let mut r = rec(&["a", "a"]);
    let target = TargetColumns::Indices(vec![1]);
    t.apply_record(&mut r, 1, None, &target).unwrap();
    assert_eq!(&r[0], "a"); // 列 0 は対象外
    assert_eq!(&r[1], "X"); // 列 1 は置換
}

/// 1 セル内の複数マッチが後ろから正しく置換される
#[test]
fn multiple_matches_in_cell() {
    let t = ReplaceTransform::new(vec![simple(0, "ab", "X")], false);
    let mut r = rec(&["ababab"]);
    let matched = t
        .apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "XXX");
    assert_eq!(matched, vec![0, 0, 0]);
}

/// 連鎖なし: ルール 0 の置換結果はルール 1 の評価対象にならない
/// "ab"→"xy" と "xy"→"zz" がある時、連鎖ありなら "zz"、連鎖なしなら "xy"
#[test]
fn no_rule_chaining() {
    let t = ReplaceTransform::new(vec![simple(0, "ab", "xy"), simple(1, "xy", "zz")], false);
    let mut r = rec(&["ab"]);
    t.apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "xy"); // 連鎖なしなので "zz" にはならない
}

/// 空セルは置換対象がなくそのまま
#[test]
fn empty_cell_unchanged() {
    let t = ReplaceTransform::new(vec![simple(0, "a", "b")], false);
    let mut r = rec(&["", "a"]);
    t.apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "");
    assert_eq!(&r[1], "b");
}

/// to が空文字列なら、マッチ部分の削除になる
#[test]
fn replace_to_empty_string() {
    let t = ReplaceTransform::new(vec![simple(0, "x", "")], false);
    let mut r = rec(&["axbxc"]);
    t.apply_record(&mut r, 1, None, &TargetColumns::All)
        .unwrap();
    assert_eq!(&r[0], "abc");
}
