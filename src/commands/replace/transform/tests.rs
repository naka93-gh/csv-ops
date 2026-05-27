// ReplaceTransform の単体テスト

use super::*;
use crate::column::ColumnRef;
use crate::rule_id::RuleId;
use crate::stats::Stats;

/// 単純置換ルールを作る (from をエスケープしてマッチャ化)
fn simple(index: usize, from: &str, to: &str) -> CompiledRule {
    CompiledRule::Simple {
        id: RuleId { index, name: None },
        matcher: regex::Regex::new(&regex::escape(from)).unwrap(),
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// 大文字小文字を区別しない単純置換ルールを作る
fn simple_ci(index: usize, from: &str, to: &str) -> CompiledRule {
    CompiledRule::Simple {
        id: RuleId { index, name: None },
        matcher: regex::RegexBuilder::new(&regex::escape(from))
            .case_insensitive(true)
            .build()
            .unwrap(),
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// 正規表現ルールを作る
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

/// ルール列から ReplaceTransform を組み立てる (統計は自動初期化)
fn transform(rules: Vec<CompiledRule>, columns: ColumnTarget) -> ReplaceTransform {
    let ids: Vec<String> = rules.iter().map(|r| r.id().to_string()).collect();
    ReplaceTransform::new(rules, columns, Stats::with_rule_ids(ids))
}

/// 単純置換が対象セルに適用される
#[test]
fn applies_simple_replacement() {
    let mut t = transform(vec![simple(0, "未対応", "open")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["未対応", "データ"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "open");
    assert_eq!(&r[1], "データ");
    assert_eq!(t.stats.per_rule[0].matches, Some(1));
}

/// 正規表現置換が適用される
#[test]
fn applies_regex_replacement() {
    let mut t = transform(vec![regex_rule(0, r"\d+", "N")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["abc123def"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "abcNdef");
}

/// case_insensitive で大文字小文字を無視してマッチする
#[test]
fn case_insensitive_matches() {
    let mut t = transform(vec![simple_ci(0, "abc", "X")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["ABC"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "X");
}

/// マッチなしの行は変更されず、統計にも計上されない
#[test]
fn no_match_leaves_unchanged() {
    let mut t = transform(vec![simple(0, "xxx", "yyy")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["abc", "def"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "abc");
    assert_eq!(t.stats.rows_changed, 0);
}

/// 範囲が重なるルールは動的衝突エラーになる
#[test]
fn overlapping_rules_collide() {
    let mut t = transform(
        vec![simple(0, "未対応", "X"), simple(1, "対応", "Y")],
        ColumnTarget::All,
    );
    t.init(None).unwrap();
    let mut r = rec(&["未対応"]);
    let err = t.on_record(&mut r, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::RuntimeCollision { .. })
    ));
}

/// 列指定で対象外の列は置換されない
#[test]
fn target_columns_limits_scope() {
    let mut t = transform(
        vec![simple(0, "a", "X")],
        ColumnTarget::Specified(vec![ColumnRef::Index(1)]),
    );
    t.init(None).unwrap();
    let mut r = rec(&["a", "a"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "a"); // 列 0 は対象外
    assert_eq!(&r[1], "X"); // 列 1 は置換
}

/// 1 セル内の複数マッチが後ろから正しく置換される
#[test]
fn multiple_matches_in_cell() {
    let mut t = transform(vec![simple(0, "ab", "X")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["ababab"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "XXX");
    assert_eq!(t.stats.per_rule[0].matches, Some(3));
}

/// 連鎖なし: ルール 0 の置換結果はルール 1 の評価対象にならない
#[test]
fn no_rule_chaining() {
    let mut t = transform(
        vec![simple(0, "ab", "xy"), simple(1, "xy", "zz")],
        ColumnTarget::All,
    );
    t.init(None).unwrap();
    let mut r = rec(&["ab"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "xy"); // 連鎖なしなので "zz" にはならない
}

/// 空セルは置換対象がなくそのまま
#[test]
fn empty_cell_unchanged() {
    let mut t = transform(vec![simple(0, "a", "b")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["", "a"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "");
    assert_eq!(&r[1], "b");
}

/// to が空文字列なら、マッチ部分の削除になる
#[test]
fn replace_to_empty_string() {
    let mut t = transform(vec![simple(0, "x", "")], ColumnTarget::All);
    t.init(None).unwrap();
    let mut r = rec(&["axbxc"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(&r[0], "abc");
}

/// ヘッダ無し + 範囲外の列番号指定は行処理時にエラー
#[test]
fn out_of_range_index_errors_without_headers() {
    let mut t = transform(
        vec![simple(0, "a", "b")],
        ColumnTarget::Specified(vec![ColumnRef::Index(5)]),
    );
    t.init(None).unwrap();
    let mut r = rec(&["1", "2"]);
    let err = t.on_record(&mut r, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2
        })
    ));
}
