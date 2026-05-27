use super::*;

/// テスト用の CSV 辞書を一時ファイルに書き出す
fn temp_dict(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("dict.csv");
    std::fs::write(&path, "canonical,alias1\n東京都,東京\n").unwrap();
    path
}

#[test]
fn from_toml_requires_version() {
    let err = SimilarityConfig::from_toml("[[rules]]\ncolumn = \"city\"\n");
    assert!(err.is_err());
}

#[test]
fn from_toml_parses_valid_config() {
    let text = "version = 1\n\n[[rules]]\ncolumn = \"city\"\ndict = \"d.csv\"\nout_col = \"m\"\nscore_col = \"s\"\n";
    let config = SimilarityConfig::from_toml(text).unwrap();
    assert_eq!(config.rules.len(), 1);
}

#[test]
fn compile_resolves_column_and_loads_dict() {
    let dir = tempfile::tempdir().unwrap();
    let dict = temp_dict(dir.path());
    let config = SimilarityConfig::from_single_rule(
        ColumnRef::Name("city".to_string()),
        dict,
        "matched".to_string(),
        "score".to_string(),
        DEFAULT_THRESHOLD,
        vec!["nfkc".to_string()],
        "levenshtein".to_string(),
    );
    let headers = StringRecord::from(vec!["city", "pref"]);
    let compiled = config.compile_rules(Some(&headers), b',').unwrap();
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].column, 0);
}

#[test]
fn compile_rejects_unknown_normalize_option() {
    let dir = tempfile::tempdir().unwrap();
    let dict = temp_dict(dir.path());
    let config = SimilarityConfig::from_single_rule(
        ColumnRef::Index(0),
        dict,
        "matched".to_string(),
        "score".to_string(),
        DEFAULT_THRESHOLD,
        vec!["bogus".to_string()],
        "levenshtein".to_string(),
    );
    let headers = StringRecord::from(vec!["city"]);
    assert!(config.compile_rules(Some(&headers), b',').is_err());
}

#[test]
fn compile_rejects_threshold_out_of_range() {
    let dir = tempfile::tempdir().unwrap();
    let dict = temp_dict(dir.path());
    let config = SimilarityConfig::from_single_rule(
        ColumnRef::Index(0),
        dict,
        "matched".to_string(),
        "score".to_string(),
        1.5,
        vec!["nfkc".to_string()],
        "levenshtein".to_string(),
    );
    let headers = StringRecord::from(vec!["city"]);
    assert!(config.compile_rules(Some(&headers), b',').is_err());
}

#[test]
fn compile_rejects_unknown_algorithm() {
    let dir = tempfile::tempdir().unwrap();
    let dict = temp_dict(dir.path());
    let config = SimilarityConfig::from_single_rule(
        ColumnRef::Index(0),
        dict,
        "matched".to_string(),
        "score".to_string(),
        DEFAULT_THRESHOLD,
        vec!["nfkc".to_string()],
        "cosine".to_string(),
    );
    let headers = StringRecord::from(vec!["city"]);
    assert!(config.compile_rules(Some(&headers), b',').is_err());
}
