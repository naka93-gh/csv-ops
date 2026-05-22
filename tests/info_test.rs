use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn reports_basic_info() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.csv");
    std::fs::write(&input, "氏名,年齢,部署\n田中,30,営業\n佐藤,25,開発\n").unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("File:        data.csv")
                .and(predicate::str::contains("Rows:        2"))
                .and(predicate::str::contains("Columns:     3"))
                .and(predicate::str::contains("UTF-8 (no BOM)"))
                .and(predicate::str::contains("氏名, 年齢, 部署")),
        );
}

#[test]
fn detects_shift_jis() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("sjis.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("氏名,年齢\n田中,30\n");
    std::fs::write(&input, &bytes).unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Shift_JIS"));
}

#[test]
fn auto_detects_tab_delimiter() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.tsv");
    std::fs::write(&input, "a\tb\tc\n1\t2\t3\n").unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Delimiter:   \\t")
                .and(predicate::str::contains("Columns:     3")),
        );
}

#[test]
fn explicit_delimiter_alias() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.csv");
    std::fs::write(&input, "a|b\n1|2\n").unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .args(["--input-delimiter", "pipe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Columns:     2"));
}

#[test]
fn detects_crlf_line_ending() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.csv");
    std::fs::write(&input, "a,b\r\n1,2\r\n").unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Line ending: CRLF"));
}

#[test]
fn json_format_is_valid() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.csv");
    std::fs::write(&input, "a,b\n1,2\n3,4\n").unwrap();

    let output = csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .args(["--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["rows"], 2);
    assert_eq!(value["columns"], 2);
    assert_eq!(value["line_ending"], "lf");
}

#[test]
fn missing_file_fails() {
    csv_ops()
        .args(["info", "-i", "/nonexistent/path/data.csv"])
        .assert()
        .failure();
}

#[test]
fn invalid_format_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("data.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();

    csv_ops()
        .args(["info", "-i"])
        .arg(&input)
        .args(["--format", "xml"])
        .assert()
        .failure();
}
