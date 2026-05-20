use csv::StringRecord;

use super::*;

/// デフォルトの true/false 値を持つルールを組み立てるヘルパ
fn rule(pattern: &str, columns: Vec<usize>, out_col: &str) -> CompiledFlagRule {
    CompiledFlagRule {
        pattern: regex::Regex::new(pattern).unwrap(),
        columns,
        out_col: out_col.to_string(),
        true_value: "true".to_string(),
        false_value: "false".to_string(),
    }
}

#[test]
fn flags_true_on_match() {
    let t = FlagTransform::new(vec![rule("東京", vec![0], "has_tokyo")]);
    let mut rec = StringRecord::from(vec!["東京都"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![true]);
    assert_eq!(rec.len(), 2);
    assert_eq!(&rec[1], "true");
}

#[test]
fn flags_false_on_no_match() {
    let t = FlagTransform::new(vec![rule("東京", vec![0], "has_tokyo")]);
    let mut rec = StringRecord::from(vec!["大阪府"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![false]);
    assert_eq!(&rec[1], "false");
}

#[test]
fn multi_column_any_match() {
    // 2 列のうちどちらかでマッチすれば true
    let t = FlagTransform::new(vec![rule("田中", vec![0, 1], "tanaka")]);
    let mut rec = StringRecord::from(vec!["佐藤", "田中ビル"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![true]);
}

#[test]
fn multi_rule_appends_in_order() {
    let t = FlagTransform::new(vec![
        rule("A", vec![0], "has_a"),
        rule("B", vec![0], "has_b"),
    ]);
    let mut rec = StringRecord::from(vec!["AAA"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![true, false]);
    assert_eq!(rec.len(), 3);
    assert_eq!(&rec[1], "true");
    assert_eq!(&rec[2], "false");
}

#[test]
fn original_columns_untouched() {
    let t = FlagTransform::new(vec![rule("x", vec![0], "f")]);
    let mut rec = StringRecord::from(vec!["xyz", "abc"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[0], "xyz");
    assert_eq!(&rec[1], "abc");
}

#[test]
fn custom_true_false_values() {
    let mut r = rule("東京", vec![0], "c");
    r.true_value = "○".to_string();
    r.false_value = "×".to_string();
    let t = FlagTransform::new(vec![r]);
    let mut rec = StringRecord::from(vec!["東京"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[1], "○");
}

#[test]
fn index_out_of_range_errors() {
    let t = FlagTransform::new(vec![rule("x", vec![5], "c")]);
    let mut rec = StringRecord::from(vec!["a", "b"]);
    assert!(t.apply_record(&mut rec).is_err());
}
