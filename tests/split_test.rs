use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn splits_column_by_name() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢\n田中 太郎,30\n佐藤 花子,25\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,年齢,姓,名\n田中 太郎,30,田中,太郎\n佐藤 花子,25,佐藤,花子\n"
    );
}

#[test]
fn preserves_other_columns() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "id,名前,年齢\n1,田中 太郎,30\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "名前", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "id,名前,年齢,姓,名\n1,田中 太郎,30,田中,太郎\n"
    );
}

#[test]
fn extra_parts_join_into_last_column() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中 太郎 ジュニア\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,姓,名\n田中 太郎 ジュニア,田中,太郎 ジュニア\n"
    );
}

#[test]
fn missing_parts_padded_with_empty() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,姓,名\n田中,田中,\n"
    );
}

#[test]
fn splits_by_index_without_headers() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "x,2024-01,y\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "1",
            "--by",
            "-",
            "--out-cols",
            "年,月",
            "--no-headers",
        ])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "x,2024-01,y,2024,01\n"
    );
}

#[test]
fn header_only_input_transforms_header() {
    // データ行が無い CSV でもヘッダーは分割後の構造で書き出す
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,年齢,姓,名\n"
    );
}

#[test]
fn handles_tab_delimiter() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\t年齢\n田中 太郎\t30\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "氏名",
            "--by",
            " ",
            "--out-cols",
            "姓,名",
            "--delimiter",
            "tab",
        ])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名\t年齢\t姓\t名\n田中 太郎\t30\t田中\t太郎\n"
    );
}

#[test]
fn preserves_crlf_line_ending() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢\r\n田中 太郎,30\r\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,年齢,姓,名\r\n田中 太郎,30,田中,太郎\r\n"
    );
}

#[test]
fn dry_run_writes_no_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中 太郎\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "氏名",
            "--by",
            " ",
            "--out-cols",
            "姓,名",
            "--dry-run",
        ])
        .assert()
        .success();

    assert!(!output.exists());
}

#[test]
fn stats_text_reports_split_rows() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    // 1 行目は区切りあり (分割成立)、2 行目は区切り無し (空埋め)
    std::fs::write(&input, "氏名\n田中 太郎\n佐藤\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("処理行数: 2").and(predicate::str::contains("ヒット行数: 1")),
        );
}

#[test]
fn empty_by_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中 太郎\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", "", "--out-cols", "姓,名"])
        .assert()
        .failure();
    assert!(!output.exists());
}

#[test]
fn empty_out_cols_name_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中 太郎\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,,名"])
        .assert()
        .failure();
    assert!(!output.exists());
}

#[test]
fn unknown_column_fails_listing_available() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢\n田中 太郎,30\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "nope", "--by", " ", "--out-cols", "姓,名"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nope").and(predicate::str::contains("氏名")));
    assert!(!output.exists());
}

#[test]
fn output_column_conflict_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢\n田中 太郎,30\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,年齢"])
        .assert()
        .failure();
    assert!(!output.exists());
}

#[test]
fn quiet_suppresses_stats() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名\n田中 太郎\n").unwrap();

    csv_ops()
        .args(["split", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--by", " ", "--out-cols", "姓,名", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
