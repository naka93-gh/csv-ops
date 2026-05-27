use assert_cmd::Command;
use tempfile::tempdir;

/// csv-ops バイナリの Command を返す
fn csv_ops() -> Command {
    Command::cargo_bin("csv-ops").unwrap()
}

/// 標準的な CSV 辞書 (canonical,alias1)
const DICT_CSV: &str = "canonical,alias1\n東京都,東京\n大阪府,大阪\n";

#[test]
fn similarity_basic_match_and_no_match() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "id,name\n1,東京\n2,xyz\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(
        out,
        "id,name,matched_name,score\n1,東京,東京都,1.0000\n2,xyz,<no match>,0.0000\n"
    );
}

#[test]
fn similarity_threshold_controls_match() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    // canonical のみ。"東京" との類似度は 2/3 ≈ 0.6667
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, "canonical\n東京都\n大阪府\n").unwrap();

    // 既定しきい値 0.7 では届かず <no match>
    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .success();
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("<no match>")
    );

    // しきい値を下げると東京都にマッチする
    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--threshold", "0.6"])
        .assert()
        .success();
    let out = std::fs::read_to_string(&output).unwrap();
    assert!(out.contains("東京都"));
    assert!(out.contains("0.6667"));
}

#[test]
fn similarity_normalize_enables_kana_match() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    // 入力はカタカナ、辞書 alias はひらがな
    std::fs::write(&input, "name\nトウキョウ\n").unwrap();
    std::fs::write(&dict, "canonical,alias1\n東京都,とうきょう\n").unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--normalize", "nfkc,kana"])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert!(out.contains("東京都"));
}

#[test]
fn similarity_config_mode() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let config = dir.path().join("rules.toml");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();
    std::fs::write(
        &config,
        format!(
            "version = 1\n\n[[rules]]\ncolumn = \"name\"\ndict = \"{}\"\nout_col = \"m\"\nscore_col = \"s\"\nthreshold = 0.7\n",
            dict.display()
        ),
    )
    .unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["--config".as_ref(), config.as_os_str()])
        .assert()
        .success();

    let out = std::fs::read_to_string(&output).unwrap();
    assert_eq!(out, "name,m,s\n東京,東京都,1.0000\n");
}

#[test]
fn similarity_sjis_input_and_dict() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    let (in_bytes, _, _) = encoding_rs::SHIFT_JIS.encode("name\n東京\n");
    let (dict_bytes, _, _) = encoding_rs::SHIFT_JIS.encode(DICT_CSV);
    std::fs::write(&input, &in_bytes).unwrap();
    std::fs::write(&dict, &dict_bytes).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--input-encoding", "shift_jis"])
        .assert()
        .success();

    // 出力は入力と同一の SJIS。辞書のエンコーディングは自動判定される
    let out_bytes = std::fs::read(&output).unwrap();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&out_bytes);
    assert!(!had_errors);
    assert!(decoded.contains("東京都"));
}

#[test]
fn similarity_toml_dictionary() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.toml");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(
        &dict,
        "version = 1\n\n[[entries]]\ncanonical = \"東京都\"\naliases = [\"東京\"]\n",
    )
    .unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .success();

    assert!(std::fs::read_to_string(&output).unwrap().contains("東京都"));
}

#[test]
fn similarity_json_stats() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\nxyz\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .arg("--json")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"rows_affected\""));
}

#[test]
fn similarity_dry_run_skips_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .arg("--dry-run")
        .assert()
        .success();

    assert!(!output.exists());
}

#[test]
fn similarity_output_column_conflict_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    // 既存カラムと out_col が衝突する
    std::fs::write(&input, "name,matched_name\n東京,x\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .failure();
}

#[test]
fn similarity_duplicate_canonical_dict_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, "canonical,alias1\n東京都,東京\n東京都,TOKYO\n").unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .failure();
}

#[test]
fn similarity_algorithm_damerau_handles_transposition() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    // "あいえうお" は "あいうえお" の隣接転置 (う/え)
    std::fs::write(&input, "name\nあいえうお\n").unwrap();
    std::fs::write(&dict, "canonical\nあいうえお\n").unwrap();

    // 既定 (levenshtein) では距離 2 → 類似度 0.6 で <no match>
    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .assert()
        .success();
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("<no match>")
    );

    // damerau では転置が距離 1 → 類似度 0.8 でマッチする
    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--algorithm", "damerau"])
        .assert()
        .success();
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("あいうえお")
    );
}

#[test]
fn similarity_algorithm_jaro_winkler_and_dice_run() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    for algo in ["jaro-winkler", "dice"] {
        let output = dir.path().join(format!("out-{}.csv", algo));
        csv_ops()
            .arg("similarity")
            .args(["-i".as_ref(), input.as_os_str()])
            .args(["-o".as_ref(), output.as_os_str()])
            .args(["-c", "name"])
            .args(["--dict".as_ref(), dict.as_os_str()])
            .args(["--algorithm", algo])
            .assert()
            .success();
        assert!(std::fs::read_to_string(&output).unwrap().contains("東京都"));
    }
}

#[test]
fn similarity_invalid_algorithm_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--algorithm", "cosine"])
        .assert()
        .failure();
}

#[test]
fn similarity_invalid_normalize_option_fails() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.csv");
    let dict = dir.path().join("dict.csv");
    let output = dir.path().join("out.csv");
    std::fs::write(&input, "name\n東京\n").unwrap();
    std::fs::write(&dict, DICT_CSV).unwrap();

    csv_ops()
        .arg("similarity")
        .args(["-i".as_ref(), input.as_os_str()])
        .args(["-o".as_ref(), output.as_os_str()])
        .args(["-c", "name"])
        .args(["--dict".as_ref(), dict.as_os_str()])
        .args(["--normalize", "bogus"])
        .assert()
        .failure();
}
