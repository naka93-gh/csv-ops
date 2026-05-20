// replace サブコマンドの統合テスト
// assert_cmd でビルド済みバイナリを実行し、CLI 全体の振る舞いを検証する

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// 一時ディレクトリに入力 CSV を書き、(dir, input パス) を返す
/// dir は TempDir。スコープを抜けると一時ディレクトリごと消える
fn setup(input_csv: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.csv");
    std::fs::write(&input, input_csv).unwrap();
    (dir, input)
}

/// csv-ops バイナリの Command を作る
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

/// CLI 引数モード (--from / --to) で置換できる
#[test]
fn replace_cli_arg_mode() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "status", "--from", "未対応", "--to", "open"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "name,status\n田中,open\n"
    );
}

/// Config モード (--config) で置換できる (単純置換 + 正規表現)
#[test]
fn replace_config_mode() {
    let (dir, input) = setup("name,status\n田中,未対応\n鈴木,対応中\n");
    let output = dir.path().join("out.csv");
    let config = dir.path().join("rules.toml");
    std::fs::write(
        &config,
        "version = 1\n[[rules]]\nfrom = \"未対応\"\nto = \"open\"\n\
         [[rules]]\npattern = \"対応中\"\nreplacement = \"in_progress\"\nregex = true\n",
    )
    .unwrap();
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .arg("--all-columns")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "name,status\n田中,open\n鈴木,in_progress\n"
    );
}

/// SJIS 入力を読み UTF-8 で出力できる
#[test]
fn replace_sjis_input() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.csv");
    // UTF-8 文字列を SJIS にエンコードして書き込む
    let (sjis, _, _) = encoding_rs::SHIFT_JIS.encode("name,status\n田中,未対応\n");
    std::fs::write(&input, &sjis).unwrap();
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "status",
            "--from",
            "未対応",
            "--to",
            "open",
            "--input-encoding",
            "shift_jis",
        ])
        .assert()
        .success();
    // 出力はデフォルト UTF-8
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "name,status\n田中,open\n"
    );
}

/// UTF-8 入力を SJIS で出力できる
#[test]
fn replace_sjis_output() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "status",
            "--from",
            "未対応",
            "--to",
            "open",
            "--output-encoding",
            "shift_jis",
        ])
        .assert()
        .success();
    // 出力ファイルを SJIS としてデコードして検証
    let out_bytes = std::fs::read(&output).unwrap();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&out_bytes);
    assert!(!had_errors);
    assert_eq!(decoded, "name,status\n田中,open\n");
}

/// 衝突する Config は実行前にエラー終了する (exit code 1)
#[test]
fn replace_collision_fails() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    let config = dir.path().join("rules.toml");
    std::fs::write(
        &config,
        "version = 1\n[[rules]]\nfrom = \"未対応\"\nto = \"open\"\n\
         [[rules]]\nfrom = \"対応\"\nto = \"x\"\n",
    )
    .unwrap();
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .arg("--all-columns")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("ルール衝突"));
}

/// version 未指定の Config はエラー終了する
#[test]
fn replace_missing_version_fails() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    let config = dir.path().join("rules.toml");
    std::fs::write(&config, "[[rules]]\nfrom = \"未対応\"\nto = \"open\"\n").unwrap();
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--config")
        .arg(&config)
        .arg("--all-columns")
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

/// --dry-run 時は出力ファイルが作られない
#[test]
fn replace_dry_run_skips_output() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "status",
            "--from",
            "未対応",
            "--to",
            "open",
            "--dry-run",
        ])
        .assert()
        .success();
    assert!(!output.exists(), "dry-run では出力ファイルを作らない");
}

/// --stats-format json で JSON 統計が出力される
#[test]
fn replace_json_stats() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "-c",
            "status",
            "--from",
            "未対応",
            "--to",
            "open",
            "--stats-format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"rows_processed\""));
}

/// -c で指定した列以外は置換されない
#[test]
fn replace_column_scoping() {
    let (dir, input) = setup("col1,col2\n未対応,データ未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "col2", "--from", "未対応", "--to", "X"])
        .assert()
        .success();
    // col1 の "未対応" は触らず、col2 の "未対応" のみ置換
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "col1,col2\n未対応,データX\n"
    );
}

/// -c も --all-columns も未指定だとエラー終了する
#[test]
fn replace_no_column_target_fails() {
    let (dir, input) = setup("name,status\n田中,未対応\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--from", "未対応", "--to", "open"])
        .assert()
        .failure();
}

/// ヘッダ行は置換されず、データ行のみ対象になる
#[test]
fn replace_header_not_modified() {
    // ヘッダにもデータにも "status" がある
    let (dir, input) = setup("name,status\nstatus,status\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["--all-columns", "--from", "status", "--to", "X"])
        .assert()
        .success();
    // ヘッダの status は不変、データ行の status は X
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "name,status\nX,X\n"
    );
}

/// セル内改行を含むマルチラインセルを扱える
#[test]
fn replace_multiline_cell() {
    let (dir, input) = setup("name,status\n田中,\"未対応\n緊急\"\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args(["-c", "status", "--from", "未対応", "--to", "open"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "name,status\n田中,\"open\n緊急\"\n"
    );
}

/// --no-headers でヘッダ無し CSV を列番号指定で置換できる
#[test]
fn replace_no_headers() {
    let (dir, input) = setup("田中,未対応\n鈴木,対応中\n");
    let output = dir.path().join("out.csv");
    csv_ops()
        .arg("replace")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([
            "--no-headers",
            "-c",
            "1",
            "--from",
            "未対応",
            "--to",
            "open",
        ])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "田中,open\n鈴木,対応中\n"
    );
}
