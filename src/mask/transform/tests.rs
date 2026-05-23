use super::*;

/// &str スライスから StringRecord を作る
fn rec(fields: &[&str]) -> StringRecord {
    fields.iter().copied().collect()
}

#[test]
fn masks_target_column_by_name() {
    let mut t = MaskTransform::new(vec![ColumnRef::Name("b".to_string())], '*');
    t.init(Some(&rec(&["a", "b", "c"]))).unwrap();
    let mut record = rec(&["foo", "bar", "baz"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record, rec(&["foo", "***", "baz"]));
    assert_eq!(t.stats.changes_total, 1);
    assert_eq!(t.stats.rows_changed, 1);
}

#[test]
fn masks_by_index_without_headers() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(0)], '*');
    t.init(None).unwrap();
    let mut record = rec(&["foo", "bar"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record, rec(&["***", "bar"]));
}

#[test]
fn preserves_character_count_for_multibyte() {
    // "あいう" は 9 バイトだが 3 文字
    let mut t = MaskTransform::new(vec![ColumnRef::Index(0)], '*');
    t.init(None).unwrap();
    let mut record = rec(&["あいう"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.get(0).unwrap(), "***");
}

#[test]
fn emoji_counts_as_one_scalar() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(0)], '*');
    t.init(None).unwrap();
    let mut record = rec(&["🦀"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.get(0).unwrap(), "*");
}

#[test]
fn empty_cell_stays_empty_and_is_not_counted() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(0)], '*');
    t.init(None).unwrap();
    let mut record = rec(&[""]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.get(0).unwrap(), "");
    assert_eq!(t.stats.changes_total, 0);
    assert_eq!(t.stats.rows_changed, 0);
}

#[test]
fn uses_configured_mask_char() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(0)], 'X');
    t.init(None).unwrap();
    let mut record = rec(&["abc"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.get(0).unwrap(), "XXX");
}

#[test]
fn unknown_column_errors() {
    let mut t = MaskTransform::new(vec![ColumnRef::Name("nope".to_string())], '*');
    let err = t.init(Some(&rec(&["a", "b"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::UnknownColumn { .. })
    ));
}

#[test]
fn name_without_headers_errors() {
    let mut t = MaskTransform::new(vec![ColumnRef::Name("a".to_string())], '*');
    let err = t.init(None).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::NameWithoutHeaders(_))
    ));
}

#[test]
fn out_of_range_index_errors_with_headers() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(5)], '*');
    let err = t.init(Some(&rec(&["a", "b"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2
        })
    ));
}

#[test]
fn out_of_range_index_errors_without_headers_per_row() {
    let mut t = MaskTransform::new(vec![ColumnRef::Index(5)], '*');
    t.init(None).unwrap();
    let mut record = rec(&["1", "2"]);
    let err = t.on_record(&mut record, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2
        })
    ));
}
