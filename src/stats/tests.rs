use super::*;

/// per_rule なし (mask / convert 風) の text フォーマット
#[test]
fn to_text_without_per_rule() {
    let stats = Stats {
        rows_processed: 10,
        rows_changed: 5,
        changes_total: 15,
        per_rule: vec![],
    };
    let expected = "\
処理行数: 10
ヒット行数: 5
ヒット総数: 15";
    assert_eq!(stats.to_text(), expected);
}

/// per_rule あり + matches Some (replace 風) の text フォーマット
#[test]
fn to_text_with_per_rule_and_matches() {
    let mut stats = Stats::with_rule_ids(vec!["rule[0]".to_string(), "rule[1]".to_string()]);
    stats.rows_processed = 10;
    stats.rows_changed = 3;
    stats.changes_total = 8;
    stats.per_rule[0].rows_affected = 2;
    stats.per_rule[0].matches = Some(5);
    stats.per_rule[1].rows_affected = 1;
    stats.per_rule[1].matches = Some(3);
    let expected = "\
処理行数: 10
ヒット行数: 3
ヒット総数: 8
ルール別:
  rule[0]: ヒット 2 行 / マッチ 5 件
  rule[1]: ヒット 1 行 / マッチ 3 件";
    assert_eq!(stats.to_text(), expected);
}

/// per_rule あり + matches None (flag / extract / similarity 風) の text フォーマット
#[test]
fn to_text_with_per_rule_without_matches() {
    let mut stats = Stats::with_rule_ids(vec!["is_a".to_string(), "is_b".to_string()]);
    stats.rows_processed = 10;
    stats.rows_changed = 4;
    stats.changes_total = 5;
    stats.per_rule[0].rows_affected = 3;
    stats.per_rule[1].rows_affected = 2;
    let expected = "\
処理行数: 10
ヒット行数: 4
ヒット総数: 5
ルール別:
  is_a: ヒット 3 行
  is_b: ヒット 2 行";
    assert_eq!(stats.to_text(), expected);
}

/// JSON 出力: per_rule なしのとき per_rule は空配列、matches は欠落
#[test]
fn to_json_without_per_rule() {
    let stats = Stats {
        rows_processed: 10,
        rows_changed: 5,
        changes_total: 15,
        per_rule: vec![],
    };
    let v: serde_json::Value = serde_json::from_str(&stats.to_json()).unwrap();
    assert_eq!(v["rows_processed"], 10);
    assert_eq!(v["rows_changed"], 5);
    assert_eq!(v["changes_total"], 15);
    assert_eq!(v["per_rule"], serde_json::json!([]));
}

/// JSON 出力: matches Some は出力される、None は skip_serializing_if で欠落する
#[test]
fn to_json_matches_serialization() {
    let mut stats = Stats::with_rule_ids(vec!["a".to_string(), "b".to_string()]);
    stats.per_rule[0].matches = Some(7);
    // [1] は None のまま
    let v: serde_json::Value = serde_json::from_str(&stats.to_json()).unwrap();
    assert_eq!(v["per_rule"][0]["matches"], 7);
    // None のフィールドは JSON に出ない
    assert!(v["per_rule"][1].get("matches").is_none());
}

/// with_rule_ids は ID リストどおりに per_rule を作り、各カウンタは 0 / None で初期化される
#[test]
fn with_rule_ids_initializes_zero() {
    let stats = Stats::with_rule_ids(vec!["x".to_string(), "y".to_string()]);
    assert_eq!(stats.rows_processed, 0);
    assert_eq!(stats.rows_changed, 0);
    assert_eq!(stats.changes_total, 0);
    assert_eq!(stats.per_rule.len(), 2);
    assert_eq!(stats.per_rule[0].id, "x");
    assert_eq!(stats.per_rule[0].rows_affected, 0);
    assert_eq!(stats.per_rule[0].matches, None);
}
