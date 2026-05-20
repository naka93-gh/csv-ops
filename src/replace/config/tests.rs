// ReplaceConfig の単体テスト

use super::*;

/// 正常な TOML がパースでき、ルールが compile できる
#[test]
fn parses_valid_toml() {
    let toml = "version = 1\n[[rules]]\nfrom = \"a\"\nto = \"b\"\n";
    let config = ReplaceConfig::from_toml(toml).unwrap();
    let compiled = config.compile_rules().unwrap();
    assert_eq!(compiled.len(), 1);
}

/// version 未指定は VersionMissing
#[test]
fn missing_version_errors() {
    let toml = "[[rules]]\nfrom = \"a\"\nto = \"b\"\n";
    let err = ReplaceConfig::from_toml(toml).unwrap_err();
    assert!(matches!(err, ConfigError::VersionMissing));
}

/// 未対応バージョンは UnsupportedVersion
#[test]
fn unsupported_version_errors() {
    let err = ReplaceConfig::from_toml("version = 2\n").unwrap_err();
    assert!(matches!(
        err,
        ConfigError::UnsupportedVersion { found: 2, .. }
    ));
}

/// regex = true なのに from があると compile でエラー
#[test]
fn regex_with_from_errors() {
    let toml = "version = 1\n[[rules]]\nfrom = \"a\"\npattern = \"b\"\nreplacement = \"c\"\nregex = true\n";
    let config = ReplaceConfig::from_toml(toml).unwrap();
    assert!(config.compile_rules().is_err());
}

/// 不正な正規表現は compile に失敗する
#[test]
fn invalid_regex_errors() {
    let toml = "version = 1\n[[rules]]\npattern = \"[\"\nreplacement = \"x\"\nregex = true\n";
    let config = ReplaceConfig::from_toml(toml).unwrap();
    assert!(config.compile_rules().is_err());
}

/// CLI 引数モード: from_single_rule から 1 ルールの config を作れる
#[test]
fn single_rule_config() {
    let config = ReplaceConfig::from_single_rule("a".into(), "b".into(), false, false);
    let compiled = config.compile_rules().unwrap();
    assert_eq!(compiled.len(), 1);
}
