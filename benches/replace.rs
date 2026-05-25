// replace サブコマンドのベンチマーク
// 計測対象: `csv-ops replace` を --dry-run で起動して 1 回実行単位の所要時間を測る
// 3 ケース × 2 サイズ:
//   - simple_1rule : 単純置換 1 ルール
//   - simple_10rules: 単純置換 10 ルール (A-7 Aho-Corasick 化の効果測定対象)
//   - regex_1rule  : 正規表現 1 ルール

mod common;

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tempfile::TempDir;

use common::{MEDIUM_ROWS, SEED, SMALL_ROWS, gen_csv, gen_replace_10_rules_toml, write_file};

fn bench_replace(c: &mut Criterion) {
    let dir = TempDir::new().expect("tempdir");

    // 入力ファイル (両サイズ共通)
    let inputs: Vec<(usize, PathBuf)> = [SMALL_ROWS, MEDIUM_ROWS]
        .iter()
        .map(|&rows| {
            let path = dir.path().join(format!("replace-{}.csv", rows));
            write_file(&path, &gen_csv(rows, SEED));
            (rows, path)
        })
        .collect();

    // 10 ルール用の TOML 設定ファイル
    let config_10 = dir.path().join("replace-10.toml");
    write_file(&config_10, &gen_replace_10_rules_toml());

    // dry-run なので実際には書き込まれないが -o は必須
    let unused_output = dir.path().join("unused.csv");

    // 単純置換 1 ルール
    let mut group = c.benchmark_group("replace/simple_1rule");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                let output = Command::cargo_bin("csv-ops")
                    .expect("csv-ops bin not found")
                    .args([
                        "replace",
                        "-i",
                        input.to_str().expect("input path"),
                        "-o",
                        unused_output.to_str().expect("output path"),
                        "--from",
                        "old_",
                        "--to",
                        "new_",
                        "-c",
                        "note",
                        "--dry-run",
                    ])
                    .output()
                    .expect("spawn failed");
                assert!(
                    output.status.success(),
                    "replace 実行に失敗: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            });
        });
    }
    group.finish();

    // 単純置換 10 ルール
    let mut group = c.benchmark_group("replace/simple_10rules");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                let output = Command::cargo_bin("csv-ops")
                    .expect("csv-ops bin not found")
                    .args([
                        "replace",
                        "-i",
                        input.to_str().expect("input path"),
                        "-o",
                        unused_output.to_str().expect("output path"),
                        "--config",
                        config_10.to_str().expect("config path"),
                        "-c",
                        "note",
                        "--dry-run",
                    ])
                    .output()
                    .expect("spawn failed");
                assert!(
                    output.status.success(),
                    "replace 実行に失敗: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            });
        });
    }
    group.finish();

    // 正規表現 1 ルール
    let mut group = c.benchmark_group("replace/regex_1rule");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                let output = Command::cargo_bin("csv-ops")
                    .expect("csv-ops bin not found")
                    .args([
                        "replace",
                        "-i",
                        input.to_str().expect("input path"),
                        "-o",
                        unused_output.to_str().expect("output path"),
                        "--from",
                        r"old_\w+",
                        "--to",
                        "new",
                        "--regex",
                        "-c",
                        "note",
                        "--dry-run",
                    ])
                    .output()
                    .expect("spawn failed");
                assert!(
                    output.status.success(),
                    "replace 実行に失敗: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_replace);
criterion_main!(benches);
