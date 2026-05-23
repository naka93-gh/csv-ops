use assert_cmd::Command;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn convert_utf8_to_sjis() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "名前,年齢\n田中,30\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--output-encoding", "shift_jis"])
        .assert()
        .success();

    let bytes = std::fs::read(&output).unwrap();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&bytes);
    assert!(!had_errors);
    assert_eq!(decoded, "名前,年齢\n田中,30\n");
}

#[test]
fn convert_sjis_to_utf8() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢\n田中,30\n");
    std::fs::write(&input, &bytes).unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--input-encoding", "shift_jis"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "名前,年齢\n田中,30\n");
}

#[test]
fn convert_euc_jp_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "名前\n田中\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--output-encoding", "euc-jp"])
        .assert()
        .success();

    let bytes = std::fs::read(&output).unwrap();
    let (decoded, _, had_errors) = encoding_rs::EUC_JP.decode(&bytes);
    assert!(!had_errors);
    assert_eq!(decoded, "名前\n田中\n");
}

#[test]
fn convert_comma_to_tab() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--output-delimiter", "tab"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "a\tb\n1\t2\n");
}

#[test]
fn convert_tab_to_comma() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a\tb\n1\t2\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--input-delimiter", "tab"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "a,b\n1,2\n");
}

#[test]
fn convert_pipe_delimiter() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--output-delimiter", "pipe"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "a|b\n1|2\n");
}

#[test]
fn convert_preserves_crlf() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a,b\r\n1,2\r\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "a,b\r\n1,2\r\n");
}

#[test]
fn convert_lf_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "a,b\n1,2\n");
    assert!(!out.contains('\r'));
}

#[test]
fn convert_identical_format() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "name,city\n田中,東京\n");
}

#[test]
fn convert_decode_failure_fails() {
    // SJIS バイト列を utf-8 として読もうとするとデコード失敗で停止する
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前\n田中\n");
    std::fs::write(&input, &bytes).unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure();
}

#[test]
fn convert_auto_detects_sjis_input() {
    // --input-encoding auto は SJIS 入力をファイル先頭から判定する
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢\n田中,30\n");
    std::fs::write(&input, &bytes).unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--input-encoding", "auto"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "名前,年齢\n田中,30\n");
}

#[test]
fn convert_auto_detects_utf8_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "名前,年齢\n田中,30\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--input-encoding", "auto"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "名前,年齢\n田中,30\n");
}

#[test]
fn convert_invalid_delimiter_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "a,b\n1,2\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--output-delimiter", "colon"])
        .assert()
        .failure();
}

#[test]
fn convert_row_count() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "h1,h2\na,b\nc,d\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("convert")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicates::str::contains("処理行数: 3"));
}
