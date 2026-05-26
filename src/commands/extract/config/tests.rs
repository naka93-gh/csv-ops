use csv::StringRecord;

use super::*;

/// 列名解決用のヘッダ
fn headers() -> StringRecord {
    StringRecord::from(vec!["name", "address", "body"])
}

#[test]
fn parses_and_compiles_minimal_config() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = '\d+'
out_col = "num"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].out_col, "num");
    assert_eq!(compiled[0].column, 2);
}

#[test]
fn missing_version_errors() {
    let toml = r#"
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
"#;
    assert!(ExtractConfig::from_toml(toml).is_err());
}

#[test]
fn unsupported_version_errors() {
    let toml = r#"
version = 2
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
"#;
    assert!(ExtractConfig::from_toml(toml).is_err());
}

#[test]
fn invalid_regex_errors() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = "[invalid"
out_col = "c"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
}

#[test]
fn unknown_column_errors() {
    let toml = r#"
version = 1
[[rules]]
column = "missing"
pattern = "x"
out_col = "c"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
}

#[test]
fn column_by_index_resolves() {
    let toml = r#"
version = 1
[[rules]]
column = 1
pattern = "x"
out_col = "c"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].column, 1);
}

#[test]
fn default_separator() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].separator, ";");
}

#[test]
fn custom_separator() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
separator = "|"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].separator, "|");
}

#[test]
fn missing_column_field_errors() {
    // column は必須。欠落は TOML パースエラーになる
    let toml = r#"
version = 1
[[rules]]
pattern = "x"
out_col = "c"
"#;
    assert!(ExtractConfig::from_toml(toml).is_err());
}

#[test]
fn multiple_rules_compile() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = '\d+'
out_col = "num"
[[rules]]
column = "name"
pattern = "田中"
out_col = "tanaka"
"#;
    let cfg = ExtractConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled.len(), 2);
    assert_eq!(compiled[1].out_col, "tanaka");
    assert_eq!(compiled[1].column, 0);
}
