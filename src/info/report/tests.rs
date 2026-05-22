use super::*;

fn sample() -> InfoReport {
    InfoReport {
        file: "data.csv".to_string(),
        size_bytes: 2048,
        encoding: "utf-8".to_string(),
        bom: false,
        delimiter: ",".to_string(),
        quote: "\"".to_string(),
        line_ending: LineEnding::Lf,
        rows: 1234,
        columns: 3,
        headers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    }
}

#[test]
fn human_size_scales_units() {
    assert_eq!(human_size(512), "512 B");
    assert_eq!(human_size(2048), "2.0 KB");
    assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
}

#[test]
fn group_digits_inserts_commas() {
    assert_eq!(group_digits(0), "0");
    assert_eq!(group_digits(999), "999");
    assert_eq!(group_digits(1234), "1,234");
    assert_eq!(group_digits(1234567), "1,234,567");
}

#[test]
fn to_text_contains_key_fields() {
    let text = sample().to_text();
    assert!(text.contains("File:        data.csv"));
    assert!(text.contains("2.0 KB"));
    assert!(text.contains("1,234"));
    assert!(text.contains("UTF-8 (no BOM)"));
}

#[test]
fn to_json_is_valid_and_has_fields() {
    let json = sample().to_json();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["file"], "data.csv");
    assert_eq!(value["rows"], 1234);
    assert_eq!(value["line_ending"], "lf");
    assert_eq!(value["delimiter"], ",");
}

#[test]
fn json_escapes_quote_character() {
    let json = sample().to_json();
    // クォート文字 " が JSON 文字列として正しくエスケープされる
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["quote"], "\"");
}

#[test]
fn encoding_text_shows_bom() {
    let mut r = sample();
    r.bom = true;
    assert!(r.to_text().contains("UTF-8 (with BOM)"));
}
