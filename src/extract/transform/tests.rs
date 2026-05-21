use csv::StringRecord;

use super::*;

/// デフォルト区切り文字を持つルールを組み立てるヘルパ
fn rule(pattern: &str, column: usize, out_col: &str) -> CompiledExtractRule {
    CompiledExtractRule {
        pattern: regex::Regex::new(pattern).unwrap(),
        column,
        out_col: out_col.to_string(),
        separator: ";".to_string(),
    }
}

#[test]
fn extracts_whole_match_without_capture() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 0, "num")]);
    let mut rec = StringRecord::from(vec!["abc123def"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![true]);
    assert_eq!(rec.len(), 2);
    assert_eq!(&rec[1], "123");
}

#[test]
fn extracts_capture_group_when_present() {
    // キャプチャグループがあればグループ 1 を採用する
    let t = ExtractTransform::new(vec![rule(r"〒(\d{3}-\d{4})", 0, "postal")]);
    let mut rec = StringRecord::from(vec!["〒123-4567 東京"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[1], "123-4567");
}

#[test]
fn empty_on_no_match() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 0, "num")]);
    let mut rec = StringRecord::from(vec!["abcdef"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![false]);
    assert_eq!(&rec[1], "");
}

#[test]
fn multiple_matches_joined_with_separator() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 0, "num")]);
    let mut rec = StringRecord::from(vec!["a1b22c333"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[1], "1;22;333");
}

#[test]
fn custom_separator() {
    let mut r = rule(r"\d+", 0, "num");
    r.separator = "|".to_string();
    let t = ExtractTransform::new(vec![r]);
    let mut rec = StringRecord::from(vec!["1a2"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[1], "1|2");
}

#[test]
fn multi_rule_appends_in_order() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 0, "num"), rule(r"[A-Z]+", 0, "upper")]);
    let mut rec = StringRecord::from(vec!["abc123XYZ"]);
    let flags = t.apply_record(&mut rec).unwrap();
    assert_eq!(flags, vec![true, true]);
    assert_eq!(rec.len(), 3);
    assert_eq!(&rec[1], "123");
    assert_eq!(&rec[2], "XYZ");
}

#[test]
fn original_columns_untouched() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 0, "num")]);
    let mut rec = StringRecord::from(vec!["x1", "abc"]);
    t.apply_record(&mut rec).unwrap();
    assert_eq!(&rec[0], "x1");
    assert_eq!(&rec[1], "abc");
}

#[test]
fn index_out_of_range_errors() {
    let t = ExtractTransform::new(vec![rule(r"\d+", 5, "num")]);
    let mut rec = StringRecord::from(vec!["a", "b"]);
    assert!(t.apply_record(&mut rec).is_err());
}
