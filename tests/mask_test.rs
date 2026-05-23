use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn masks_target_column_by_name() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "氏名,年齢,部署\n田中,30,営業\n佐藤,25,開発\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,年齢,部署\n**,30,営業\n**,25,開発\n"
    );
}

#[test]
fn masks_multiple_columns() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b,c\nfoo,bar,baz\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a,c"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "a,b,c\n***,bar,***\n"
    );
}

#[test]
fn masks_by_index() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b,c\nfoo,bar,baz\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "1"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "a,b,c\nfoo,***,baz\n"
    );
}

#[test]
fn masks_without_headers_by_index() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "foo,bar,baz\nhello,world,!\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "0", "--no-headers"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "***,bar,baz\n*****,world,!\n"
    );
}

#[test]
fn uses_configured_mask_char() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\nfoo,bar\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a", "--mask-char", "X"])
        .assert()
        .success();

    assert_eq!(std::fs::read_to_string(&output).unwrap(), "a,b\nXXX,bar\n");
}

#[test]
fn unknown_column_fails_listing_available() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "header1,header2\n1,2\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nope").and(predicate::str::contains("header1")));
    // 失敗時に出力ファイルを残さない
    assert!(!output.exists());
}

#[test]
fn name_without_headers_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "1,2,3\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "氏名", "--no-headers"])
        .assert()
        .failure();
}

#[test]
fn out_of_range_index_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "5"])
        .assert()
        .failure();
}

#[test]
fn handles_tab_delimiter() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a\tb\nfoo\tbar\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "b", "--delimiter", "tab"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "a\tb\nfoo\t***\n"
    );
}

#[test]
fn shift_jis_round_trip() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("氏名,年齢\n田中,30\n");
    std::fs::write(&input, &bytes).unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "氏名",
            "--input-encoding",
            "shift_jis",
            "--output-encoding",
            "shift_jis",
        ])
        .assert()
        .success();

    let out = std::fs::read(&output).unwrap();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&out);
    assert!(!had_errors);
    assert_eq!(decoded, "氏名,年齢\n**,30\n");
}

#[test]
fn decode_failure_fails() {
    // SJIS バイト列を utf-8 として読もうとするとデコード失敗で停止する
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("氏名\n田中\n");
    std::fs::write(&input, &bytes).unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "0", "--no-headers"])
        .assert()
        .failure();
    assert!(!output.exists());
}

#[test]
fn preserves_crlf_line_ending() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\r\nfoo,bar\r\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "a,b\r\n***,bar\r\n"
    );
}

#[test]
fn dry_run_writes_no_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\nfoo,bar\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a", "--dry-run"])
        .assert()
        .success();

    assert!(!output.exists());
}

#[test]
fn stats_text_reports_counts() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\nfoo,bar\n,baz\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("処理行数: 2").and(
            // 空セルの行はマスクされないのでヒット行数は 1
            predicate::str::contains("ヒット行数: 1"),
        ));
}

#[test]
fn stats_json_format() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\nfoo,bar\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "a", "--stats-format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changes_total\": 1"));
}

#[test]
fn config_mode_masks_columns() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    let config = dir.path().join("mask.toml");
    std::fs::write(&input, "氏名,年齢\n田中,30\n").unwrap();
    std::fs::write(
        &config,
        "version = 1\ncolumns = [\"氏名\"]\n\n[options]\nmask_char = \"#\"\n",
    )
    .unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "氏名,年齢\n##,30\n"
    );
}

#[test]
fn config_missing_version_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    let config = dir.path().join("mask.toml");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();
    std::fs::write(&config, "columns = [\"a\"]\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .assert()
        .failure();
}

#[test]
fn requires_columns_or_config() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();

    csv_ops()
        .args(["mask", "-i"])
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure();
}
