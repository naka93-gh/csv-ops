use super::*;

/// TOML から FlagTransform を組み立てるヘルパ
fn transform(toml: &str) -> FlagTransform {
    FlagTransform::new(FlagConfig::from_toml(toml).unwrap())
}

fn rec(fields: &[&str]) -> StringRecord {
    fields.iter().copied().collect()
}

#[test]
fn flags_true_on_match() {
    let mut t = transform(
        "version = 1\n[[rules]]\ncolumn = 0\npattern = \"東京\"\nout_col = \"has_tokyo\"\n",
    );
    t.init(None).unwrap();
    let mut record = rec(&["東京都"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 2);
    assert_eq!(&record[1], "true");
    assert_eq!(t.stats.per_rule[0].matched_rows, 1);
}

#[test]
fn flags_false_on_no_match() {
    let mut t = transform(
        "version = 1\n[[rules]]\ncolumn = 0\npattern = \"東京\"\nout_col = \"has_tokyo\"\n",
    );
    t.init(None).unwrap();
    let mut record = rec(&["大阪府"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "false");
    assert_eq!(t.stats.per_rule[0].matched_rows, 0);
}

#[test]
fn multi_column_any_match() {
    // 2 列のうちどちらかでマッチすれば true
    let mut t = transform(
        "version = 1\n[[rules]]\ncolumns = [0, 1]\npattern = \"田中\"\nout_col = \"tanaka\"\n",
    );
    t.init(None).unwrap();
    let mut record = rec(&["佐藤", "田中ビル"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[2], "true");
}

#[test]
fn multi_rule_appends_in_order() {
    let mut t = transform(
        "version = 1\n\
         [[rules]]\ncolumn = 0\npattern = \"A\"\nout_col = \"has_a\"\n\
         [[rules]]\ncolumn = 0\npattern = \"B\"\nout_col = \"has_b\"\n",
    );
    t.init(None).unwrap();
    let mut record = rec(&["AAA"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 3);
    assert_eq!(&record[1], "true");
    assert_eq!(&record[2], "false");
}

#[test]
fn original_columns_untouched() {
    let mut t = transform("version = 1\n[[rules]]\ncolumn = 0\npattern = \"x\"\nout_col = \"f\"\n");
    t.init(None).unwrap();
    let mut record = rec(&["xyz", "abc"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[0], "xyz");
    assert_eq!(&record[1], "abc");
}

#[test]
fn custom_true_false_values() {
    let mut t = transform(
        "version = 1\n[[rules]]\ncolumn = 0\npattern = \"東京\"\nout_col = \"c\"\n\
         true_value = \"○\"\nfalse_value = \"×\"\n",
    );
    t.init(None).unwrap();
    let mut record = rec(&["東京"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "○");
}

#[test]
fn index_out_of_range_errors() {
    let mut t = transform("version = 1\n[[rules]]\ncolumn = 5\npattern = \"x\"\nout_col = \"c\"\n");
    t.init(None).unwrap();
    let mut record = rec(&["a", "b"]);
    let err = t.on_record(&mut record, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange { .. })
    ));
}

#[test]
fn out_col_conflict_with_existing_header_errors() {
    let mut t =
        transform("version = 1\n[[rules]]\ncolumn = 0\npattern = \"x\"\nout_col = \"既存\"\n");
    let err = t.init(Some(&rec(&["a", "既存"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}
