use super::*;
use crate::text::algorithm::Algorithm;

/// テスト用の辞書を一時ファイルに書き出す
fn write_temp(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn loads_csv_dictionary() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(
        dir.path(),
        "dict.csv",
        "canonical,alias1,alias2\n東京都,東京,Tokyo\n大阪府,大阪,\n",
    );
    let dict = Dictionary::load(&path, b',', &NormalizeSet::default_set()).unwrap();
    assert_eq!(dict.canonicals, vec!["東京都", "大阪府"]);
}

#[test]
fn loads_toml_dictionary() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(
        dir.path(),
        "dict.toml",
        "version = 1\n\n[[entries]]\ncanonical = \"東京都\"\naliases = [\"東京\", \"Tokyo\"]\n",
    );
    let dict = Dictionary::load(&path, b',', &NormalizeSet::default_set()).unwrap();
    assert_eq!(dict.canonicals, vec!["東京都"]);
}

#[test]
fn toml_dictionary_requires_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(
        dir.path(),
        "dict.toml",
        "[[entries]]\ncanonical = \"東京都\"\n",
    );
    assert!(Dictionary::load(&path, b',', &NormalizeSet::default_set()).is_err());
}

#[test]
fn rejects_empty_dictionary() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(dir.path(), "dict.csv", "canonical,alias1\n");
    let err = Dictionary::load(&path, b',', &NormalizeSet::default_set()).unwrap_err();
    assert!(matches!(err, CsvOpsError::Dict(DictError::Empty(_))));
}

#[test]
fn rejects_duplicate_canonical() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(
        dir.path(),
        "dict.csv",
        "canonical,alias1\n東京都,東京\n東京都,TOKYO\n",
    );
    let err = Dictionary::load(&path, b',', &NormalizeSet::default_set()).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Dict(DictError::DuplicateCanonical(_))
    ));
}

#[test]
fn rejects_duplicate_alias_across_canonicals() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp(
        dir.path(),
        "dict.csv",
        "canonical,alias1\n東京都,みやこ\n京都府,みやこ\n",
    );
    let err = Dictionary::load(&path, b',', &NormalizeSet::default_set()).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Dict(DictError::DuplicateAlias { .. })
    ));
}

#[test]
fn best_match_picks_closest_entry() {
    let entries = vec![
        DictEntry {
            canonical: "東京都".to_string(),
            aliases: vec!["東京".to_string()],
        },
        DictEntry {
            canonical: "大阪府".to_string(),
            aliases: vec!["大阪".to_string()],
        },
    ];
    let dict = Dictionary::build(entries, &NormalizeSet::default_set());
    let result = dict.best_match("東京", Algorithm::Levenshtein);
    assert_eq!(result.canonical, "東京都");
    assert_eq!(result.score, 1.0);
    assert!(!result.tie);
}

#[test]
fn best_match_matches_against_alias() {
    let entries = vec![DictEntry {
        canonical: "東京都".to_string(),
        aliases: vec!["Tokyo".to_string()],
    }];
    // alias にマッチしても返るのは canonical
    let dict = Dictionary::build(entries, &NormalizeSet::default_set());
    let result = dict.best_match("tokyo", Algorithm::Levenshtein);
    assert_eq!(result.canonical, "東京都");
    assert_eq!(result.score, 1.0);
}

#[test]
fn best_match_detects_tie() {
    let entries = vec![
        DictEntry {
            canonical: "AAA".to_string(),
            aliases: vec![],
        },
        DictEntry {
            canonical: "BBB".to_string(),
            aliases: vec![],
        },
    ];
    let dict = Dictionary::build(entries, &NormalizeSet::default_set());
    // "xxx" は両方から等距離。記述順で先勝ち + tie
    let result = dict.best_match("xxx", Algorithm::Levenshtein);
    assert_eq!(result.canonical, "AAA");
    assert!(result.tie);
}
