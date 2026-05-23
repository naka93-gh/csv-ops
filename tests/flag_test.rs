use assert_cmd::Command;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

#[test]
fn flag_cli_arg_mode() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京都\n佐藤,大阪府\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "city", "--pattern", "東京", "--out-col", "has_tokyo"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(
        out,
        "name,city,has_tokyo\n田中,東京都,true\n佐藤,大阪府,false\n"
    );
}

#[test]
fn flag_config_mode() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n佐藤,大阪\n").unwrap();
    let config = dir.path().join("rules.toml");
    std::fs::write(
        &config,
        "version = 1\n[[rules]]\ncolumn = \"city\"\npattern = \"東京|大阪\"\nout_col = \"major\"\ntrue_value = \"○\"\nfalse_value = \"×\"\n",
    )
    .unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "name,city,major\n田中,東京,○\n佐藤,大阪,○\n");
}

#[test]
fn flag_multi_column() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,address\n田中,大阪\n佐藤,田中町\n").unwrap();
    let config = dir.path().join("rules.toml");
    std::fs::write(
        &config,
        "version = 1\n[[rules]]\ncolumns = [\"name\", \"address\"]\npattern = \"田中\"\nout_col = \"tanaka\"\n",
    )
    .unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(
        out,
        "name,address,tanaka\n田中,大阪,true\n佐藤,田中町,true\n"
    );
}

#[test]
fn flag_no_headers() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "田中,東京\n佐藤,大阪\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "1",
            "--pattern",
            "東京",
            "--out-col",
            "f",
            "--no-headers",
        ])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "田中,東京,true\n佐藤,大阪,false\n");
}

#[test]
fn flag_out_col_conflict_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "city", "--pattern", "東京", "--out-col", "name"])
        .assert()
        .failure();
}

#[test]
fn flag_dry_run_skips_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "city",
            "--pattern",
            "東京",
            "--out-col",
            "f",
            "--dry-run",
        ])
        .assert()
        .success();

    assert!(!output.exists());
}

#[test]
fn flag_sjis_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("name,city\n田中,東京\n");
    std::fs::write(&input, &bytes).unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "city", "--pattern", "東京", "--out-col", "f"])
        .args(["--input-encoding", "shift_jis"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "name,city,f\n田中,東京,true\n");
}

#[test]
fn flag_json_stats() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n佐藤,大阪\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "city", "--pattern", "東京", "--out-col", "f"])
        .args(["--stats-format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"rows_processed\": 2"))
        .stdout(predicates::str::contains("\"rows_affected\": 1"));
}

#[test]
fn flag_missing_pattern_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, "name,city\n田中,東京\n").unwrap();
    let output = dir.path().join("out.csv");

    csv_ops()
        .arg("flag")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "city", "--out-col", "f"])
        .assert()
        .failure();
}
