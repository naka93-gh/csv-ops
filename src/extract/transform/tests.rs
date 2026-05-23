use super::*;

/// TOML から ExtractTransform を組み立てるヘルパ
fn transform(toml: &str) -> ExtractTransform {
    ExtractTransform::new(ExtractConfig::from_toml(toml).unwrap())
}

fn rec(fields: &[&str]) -> StringRecord {
    fields.iter().copied().collect()
}

#[test]
fn extracts_whole_match_without_capture() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["abc123def"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 2);
    assert_eq!(&record[1], "123");
    assert_eq!(t.stats.per_rule[0].rows_affected, 1);
}

#[test]
fn extracts_capture_group_when_present() {
    // キャプチャグループがあればグループ 1 を採用する
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '〒(\d{3}-\d{4})'
out_col = "postal"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["〒123-4567 東京"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "123-4567");
}

#[test]
fn empty_on_no_match() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["abcdef"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "");
    assert_eq!(t.stats.per_rule[0].rows_affected, 0);
}

#[test]
fn multiple_matches_joined_with_separator() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["a1b22c333"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "1;22;333");
}

#[test]
fn custom_separator() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
separator = "|"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["1a2"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "1|2");
}

#[test]
fn multi_rule_appends_in_order() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
[[rules]]
column = 0
pattern = '[A-Z]+'
out_col = "upper"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["abc123XYZ"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 3);
    assert_eq!(&record[1], "123");
    assert_eq!(&record[2], "XYZ");
}

#[test]
fn original_columns_untouched() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 0
pattern = '\d+'
out_col = "num"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["x1", "abc"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[0], "x1");
    assert_eq!(&record[1], "abc");
}

#[test]
fn index_out_of_range_errors() {
    let mut t = transform(
        r#"version = 1
[[rules]]
column = 5
pattern = '\d+'
out_col = "num"
"#,
    );
    t.init(None).unwrap();
    let mut record = rec(&["a", "b"]);
    let err = t.on_record(&mut record, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange { .. })
    ));
}
