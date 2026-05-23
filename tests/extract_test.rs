use assert_cmd::Command;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn extract_cli_arg_mode() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,memo\n田中,連絡先 03-1234-5678\n佐藤,なし\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "memo",
            "--pattern",
            r"\d{2,4}-\d{4}-\d{4}",
            "--out-col",
            "phone",
        ])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(
        out,
        "name,memo,phone\n田中,連絡先 03-1234-5678,03-1234-5678\n佐藤,なし,\n"
    );
}

#[test]
fn extract_capture_group() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "addr\n〒123-4567 東京\n").unwrap();
    let config = dir.path().join("rules.toml");
    std::fs::write(
        &config,
        "version = 1\n[[rules]]\ncolumn = \"addr\"\npattern = '〒(\\d{3}-\\d{4})'\nout_col = \"postal\"\n",
    )
    .unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "addr,postal\n〒123-4567 東京,123-4567\n");
}

#[test]
fn extract_multiple_matches_joined() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "body\na1b22c333\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "body", "--pattern", r"\d+", "--out-col", "nums"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "body,nums\na1b22c333,1;22;333\n");
}

#[test]
fn extract_custom_separator() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "body\n1a2a3\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "body", "--pattern", r"\d+", "--out-col", "nums"])
        .args(["--separator", "|"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "body,nums\n1a2a3,1|2|3\n");
}

#[test]
fn extract_no_headers() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "田中,03-1234-5678\n佐藤,なし\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "1",
            "--pattern",
            r"\d{2}-\d{4}-\d{4}",
            "--out-col",
            "phone",
            "--no-headers",
        ])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "田中,03-1234-5678,03-1234-5678\n佐藤,なし,\n");
}

#[test]
fn extract_out_col_conflict_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,memo\n田中,123\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "memo", "--pattern", r"\d+", "--out-col", "name"])
        .assert()
        .failure();
}

#[test]
fn extract_dry_run_skips_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,memo\n田中,123\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "memo",
            "--pattern",
            r"\d+",
            "--out-col",
            "num",
            "--dry-run",
        ])
        .assert()
        .success();

    assert!(!output.exists());
}

#[test]
fn extract_sjis_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("name,memo\n田中,電話 03-1234-5678\n");
    std::fs::write(&input, &bytes).unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "memo",
            "--pattern",
            r"\d{2}-\d{4}-\d{4}",
            "--out-col",
            "phone",
        ])
        .args(["--input-encoding", "shift_jis"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(
        out,
        "name,memo,phone\n田中,電話 03-1234-5678,03-1234-5678\n"
    );
}

#[test]
fn extract_json_stats() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,memo\n田中,123\n佐藤,なし\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "memo", "--pattern", r"\d+", "--out-col", "num"])
        .args(["--stats-format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"rows_processed\": 2"))
        .stdout(predicates::str::contains("\"rows_affected\": 1"));
}

#[test]
fn extract_missing_pattern_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,memo\n田中,123\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("extract")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "memo", "--out-col", "num"])
        .assert()
        .failure();
}
