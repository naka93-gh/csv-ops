use super::*;

#[test]
fn parses_valid_config() {
    let cfg = MaskConfig::from_toml(
        r#"
version = 1
columns = ["氏名", 2]

[options]
mask_char = "X"
"#,
    )
    .unwrap();
    assert_eq!(cfg.columns().len(), 2);
    assert_eq!(cfg.mask_char(), 'X');
}

#[test]
fn mask_char_defaults_to_asterisk() {
    let cfg = MaskConfig::from_toml("version = 1\ncolumns = [\"氏名\"]\n").unwrap();
    assert_eq!(cfg.mask_char(), '*');
}

#[test]
fn missing_version_errors() {
    let err = MaskConfig::from_toml("columns = [\"氏名\"]\n").unwrap_err();
    assert!(matches!(err, ConfigError::VersionMissing));
}

#[test]
fn unsupported_version_errors() {
    let err = MaskConfig::from_toml("version = 99\ncolumns = [\"氏名\"]\n").unwrap_err();
    assert!(matches!(
        err,
        ConfigError::UnsupportedVersion { found: 99, .. }
    ));
}

#[test]
fn empty_columns_errors() {
    let err = MaskConfig::from_toml("version = 1\ncolumns = []\n").unwrap_err();
    assert!(matches!(err, ConfigError::Validation(_)));
}
