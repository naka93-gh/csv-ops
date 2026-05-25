use super::*;

use std::path::Path;

use crate::error::TransformError;

use tempfile::TempDir;

/// 辞書 CSV を一時ファイルに書き、そのパスを返す
fn write_dict(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// 1 ルールの SimilarityConfig TOML を組み立てる
/// 辞書パスは write_dict で先に作っておく前提
fn build_toml(
    column: &str,
    dict_path: &Path,
    out_col: &str,
    score_col: &str,
    threshold: f64,
    algorithm: &str,
) -> String {
    format!(
        "version = 1\n\
         [[rules]]\n\
         column = \"{column}\"\n\
         dict = \"{dict}\"\n\
         out_col = \"{out_col}\"\n\
         score_col = \"{score_col}\"\n\
         threshold = {threshold}\n\
         algorithm = \"{algorithm}\"\n\
         normalize = []\n",
        column = column,
        dict = dict_path.display(),
        out_col = out_col,
        score_col = score_col,
        threshold = threshold,
        algorithm = algorithm,
    )
}

fn rec(fields: &[&str]) -> StringRecord {
    fields.iter().copied().collect()
}

#[test]
fn init_errors_on_out_col_conflict_with_existing_header() {
    // out_col "matched_name" が既存ヘッダーに既にある → OutputColumnConflict
    let dir = TempDir::new().unwrap();
    let dict = write_dict(dir.path(), "dict.csv", "canonical,alias1\n東京都,東京\n");
    let toml = build_toml("name", &dict, "matched_name", "score", 0.7, "levenshtein");
    let config = SimilarityConfig::from_toml(&toml).unwrap();
    let mut t = SimilarityTransform::new(config, b',');

    let err = t.init(Some(&rec(&["name", "matched_name"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}

#[test]
fn init_errors_on_score_col_conflict_with_existing_header() {
    // score_col "score" が既存ヘッダーに既にある → OutputColumnConflict
    let dir = TempDir::new().unwrap();
    let dict = write_dict(dir.path(), "dict.csv", "canonical,alias1\n東京都,東京\n");
    let toml = build_toml("name", &dict, "matched_name", "score", 0.7, "levenshtein");
    let config = SimilarityConfig::from_toml(&toml).unwrap();
    let mut t = SimilarityTransform::new(config, b',');

    let err = t.init(Some(&rec(&["name", "score"]))).unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::OutputColumnConflict { .. })
    ));
}

#[test]
fn threshold_branch_hit_writes_canonical() {
    // threshold = 0.5 で「東京」入力 (辞書 "東京都" との Levenshtein は 0.66...) はヒット扱い
    let dir = TempDir::new().unwrap();
    let dict = write_dict(dir.path(), "dict.csv", "canonical,alias1\n東京都,\n");
    let toml = build_toml("name", &dict, "matched_name", "score", 0.5, "levenshtein");
    let config = SimilarityConfig::from_toml(&toml).unwrap();
    let mut t = SimilarityTransform::new(config, b',');
    t.init(Some(&rec(&["name"]))).unwrap();

    let mut record = rec(&["東京"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 3);
    assert_eq!(&record[1], "東京都");
    assert_eq!(t.stats.per_rule[0].rows_affected, 1);
    assert_eq!(t.stats.changes_total, 1);
    assert_eq!(t.stats.rows_changed, 1);
}

#[test]
fn threshold_branch_miss_writes_no_match() {
    // threshold = 0.99 で「大阪」入力 (辞書 "東京都" との Levenshtein は低い) は未ヒット → <no match>
    let dir = TempDir::new().unwrap();
    let dict = write_dict(dir.path(), "dict.csv", "canonical,alias1\n東京都,\n");
    let toml = build_toml("name", &dict, "matched_name", "score", 0.99, "levenshtein");
    let config = SimilarityConfig::from_toml(&toml).unwrap();
    let mut t = SimilarityTransform::new(config, b',');
    t.init(Some(&rec(&["name"]))).unwrap();

    let mut record = rec(&["大阪"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(record.len(), 3);
    assert_eq!(&record[1], "<no match>");
    assert_eq!(t.stats.per_rule[0].rows_affected, 0);
    assert_eq!(t.stats.changes_total, 0);
    assert_eq!(t.stats.rows_changed, 0);
}

#[test]
fn tie_picks_first_dictionary_entry() {
    // 「CCC」入力で辞書 ["AAA", "BBB"] は両方とも Levenshtein score 0.0 で同点
    // threshold = 0.0 でヒット扱いにし、辞書記述順で先勝ち = "AAA" が選ばれることを確認
    // (tie 警告は stderr に出るが、ここでは「先勝ち動作 + score が同値」で tie path を通った証拠とする)
    let dir = TempDir::new().unwrap();
    let dict = write_dict(dir.path(), "dict.csv", "canonical,alias1\nAAA,\nBBB,\n");
    let toml = build_toml("name", &dict, "matched_name", "score", 0.0, "levenshtein");
    let config = SimilarityConfig::from_toml(&toml).unwrap();
    let mut t = SimilarityTransform::new(config, b',');
    t.init(Some(&rec(&["name"]))).unwrap();

    let mut record = rec(&["CCC"]);
    t.on_record(&mut record, 1).unwrap();
    assert_eq!(&record[1], "AAA");
    // score も 0.0000 になる (AAA と BBB で同値)
    assert_eq!(&record[2], "0.0000");
}
