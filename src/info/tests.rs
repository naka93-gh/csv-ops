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

