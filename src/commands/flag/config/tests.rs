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
pattern = "東京"
out_col = "has_tokyo"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].out_col, "has_tokyo");
    assert_eq!(compiled[0].columns, vec![2]);
}

#[test]
fn missing_version_errors() {
    let toml = r#"
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
"#;
    assert!(FlagConfig::from_toml(toml).is_err());
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
    assert!(FlagConfig::from_toml(toml).is_err());
}

#[test]
fn column_and_columns_both_set_errors() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
columns = ["name"]
pattern = "x"
out_col = "c"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
}

#[test]
fn neither_column_set_errors() {
    let toml = r#"
version = 1
[[rules]]
pattern = "x"
out_col = "c"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
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
    let cfg = FlagConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
}

#[test]
fn multi_column_resolves_all() {
    let toml = r#"
version = 1
[[rules]]
columns = ["name", "address"]
pattern = "x"
out_col = "c"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].columns, vec![0, 1]);
}

#[test]
fn default_true_false_values() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].true_value, "true");
    assert_eq!(compiled[0].false_value, "false");
}

#[test]
fn custom_true_false_values() {
    let toml = r#"
version = 1
[[rules]]
column = "body"
pattern = "x"
out_col = "c"
true_value = "○"
false_value = "×"
"#;
    let cfg = FlagConfig::from_toml(toml).unwrap();
    let compiled = cfg.compile_rules(Some(&headers())).unwrap();
    assert_eq!(compiled[0].true_value, "○");
    assert_eq!(compiled[0].false_value, "×");
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
    let cfg = FlagConfig::from_toml(toml).unwrap();
    assert!(cfg.compile_rules(Some(&headers())).is_err());
}
