use super::*;

#[test]
fn canonical_name_maps_known_encodings() {
    assert_eq!(canonical_name(encoding_rs::UTF_8), "utf-8");
    assert_eq!(canonical_name(encoding_rs::SHIFT_JIS), "shift_jis");
    assert_eq!(canonical_name(encoding_rs::EUC_JP), "euc-jp");
}

#[test]
fn delimiter_display_shows_tab_as_escape() {
    assert_eq!(delimiter_display(b','), ",");
    assert_eq!(delimiter_display(b'\t'), "\\t");
    assert_eq!(delimiter_display(b'|'), "|");
}

#[test]
fn detect_delimiter_picks_most_frequent() {
    assert_eq!(detect_delimiter("a,b,c\n1,2,3"), b',');
    assert_eq!(detect_delimiter("a\tb\tc\n"), b'\t');
    assert_eq!(detect_delimiter("a|b|c\n"), b'|');
}

#[test]
fn detect_delimiter_defaults_to_comma() {
    // 候補が 1 つも無ければカンマ
    assert_eq!(detect_delimiter("singlecolumn\n"), b',');
}

#[test]
fn detect_delimiter_ignores_stray_candidate_in_header() {
    // ヘッダに ";" が 1 つ混ざっているが本文はカンマ区切り
    // 単純多数決ベースだと 1 行目の ";" に引きずられるが、
    // 列数の安定度で見ると 4 列 (カンマ) が複数行で揃うのでカンマが選ばれる
    let csv = "id,name,note;memo,extra\n\
               1,alice,hello,x\n\
               2,bob,world,y\n\
               3,carol,foo,z\n";
    assert_eq!(detect_delimiter(csv), b',');
}

#[test]
fn detect_delimiter_chooses_stable_columns_across_lines() {
    // 全行で列数が安定する区切り文字を優先する
    let csv = "a;b;c\n1;2;3\n4;5;6\n";
    assert_eq!(detect_delimiter(csv), b';');
}

#[test]
fn detect_delimiter_handles_empty_input() {
    assert_eq!(detect_delimiter(""), b',');
}

