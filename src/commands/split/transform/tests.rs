use super::*;

use crate::error::TransformError;

/// &str スライスから StringRecord を作る
fn rec(fields: &[&str]) -> StringRecord {
    fields.iter().copied().collect()
}

/// テスト用の SplitTransform を組み立てる
fn split(col: ColumnRef, by: &str, out_cols: &[&str]) -> SplitTransform {
    SplitTransform::new(
        col,
        by.to_string(),
        out_cols.iter().map(|s| s.to_string()).collect(),
    )
}

#[test]
fn appends_split_columns_keeping_source() {
    let mut t = split(ColumnRef::Name("氏名".into()), " ", &["姓", "名"]);
    let header = t.init(Some(&rec(&["氏名", "年齢"]))).unwrap().unwrap();
    // 既存列はそのまま、末尾に 姓/名 を追加する
    assert_eq!(header, rec(&["氏名", "年齢", "姓", "名"]));

    let mut r = rec(&["田中 太郎", "30"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["田中 太郎", "30", "田中", "太郎"]));
    assert_eq!(t.stats.rows_changed, 1);
    assert_eq!(t.stats.changes_total, 1);
}

#[test]
fn preserves_existing_columns_and_order() {
    let mut t = split(ColumnRef::Name("名前".into()), " ", &["姓", "名"]);
    let header = t
        .init(Some(&rec(&["id", "名前", "年齢"])))
        .unwrap()
        .unwrap();
    assert_eq!(header, rec(&["id", "名前", "年齢", "姓", "名"]));

    let mut r = rec(&["1", "田中 太郎", "30"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["1", "田中 太郎", "30", "田中", "太郎"]));
}

#[test]
fn extra_parts_join_into_last_column() {
    // out_cols の列数より分割数が多い場合、末尾列が残りを吸収する (無損失)
    let mut t = split(ColumnRef::Index(0), " ", &["姓", "名"]);
    t.init(None).unwrap();
    let mut r = rec(&["田中 太郎 ジュニア"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["田中 太郎 ジュニア", "田中", "太郎 ジュニア"]));
    assert_eq!(t.stats.rows_changed, 1);
}

#[test]
fn missing_parts_padded_with_empty() {
    // 区切りが無い場合、不足列は空文字で埋める。分割は発生していない扱い
    let mut t = split(ColumnRef::Index(0), " ", &["姓", "名"]);
    t.init(None).unwrap();
    let mut r = rec(&["田中"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["田中", "田中", ""]));
    assert_eq!(t.stats.rows_changed, 0);
    assert_eq!(t.stats.changes_total, 0);
}

#[test]
fn splits_by_index_without_headers() {
    let mut t = split(ColumnRef::Index(1), "-", &["年", "月"]);
    assert!(t.init(None).unwrap().is_none());
    let mut r = rec(&["x", "2024-01", "y"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["x", "2024-01", "y", "2024", "01"]));
}

#[test]
fn empty_value_yields_empty_columns() {
    let mut t = split(ColumnRef::Index(0), " ", &["姓", "名"]);
    t.init(None).unwrap();
    let mut r = rec(&[""]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["", "", ""]));
    assert_eq!(t.stats.rows_changed, 0);
}

#[test]
fn supports_multi_char_delimiter() {
    let mut t = split(ColumnRef::Index(0), " - ", &["a", "b"]);
    t.init(None).unwrap();
    let mut r = rec(&["x - y"]);
    t.on_record(&mut r, 1).unwrap();
    assert_eq!(r, rec(&["x - y", "x", "y"]));
}

#[test]
fn unknown_column_errors() {
    let mut t = split(ColumnRef::Name("nope".into()), " ", &["a", "b"]);
    let err = t.init(Some(&rec(&["x", "y"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::UnknownColumn { .. })
    ));
}

#[test]
fn name_without_headers_errors() {
    let mut t = split(ColumnRef::Name("氏名".into()), " ", &["a", "b"]);
    let err = t.init(None).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::NameWithoutHeaders(_))
    ));
}

#[test]
fn out_of_range_index_errors_with_headers() {
    let mut t = split(ColumnRef::Index(5), " ", &["a", "b"]);
    let err = t.init(Some(&rec(&["x", "y"]))).unwrap_err();
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
    let mut t = split(ColumnRef::Index(5), " ", &["a", "b"]);
    t.init(None).unwrap();
    let mut r = rec(&["x", "y"]);
    let err = t.on_record(&mut r, 1).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2
        })
    ));
}

#[test]
fn out_cols_conflicting_with_existing_column_errors() {
    // out_cols 名が既存カラムと衝突する
    let mut t = split(ColumnRef::Name("氏名".into()), " ", &["姓", "年齢"]);
    let err = t.init(Some(&rec(&["氏名", "年齢"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}

#[test]
fn out_cols_conflicting_with_source_column_errors() {
    // 元列は残るので、分割元と同名の out_cols は衝突する
    let mut t = split(ColumnRef::Name("氏名".into()), " ", &["氏名", "名"]);
    let err = t.init(Some(&rec(&["氏名", "年齢"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}

#[test]
fn duplicate_out_cols_names_error() {
    let mut t = split(ColumnRef::Name("氏名".into()), " ", &["姓", "姓"]);
    let err = t.init(Some(&rec(&["氏名", "年齢"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}
