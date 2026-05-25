// mask サブコマンドのベンチマーク
// 計測対象: `csv-ops mask` を --dry-run で起動して 1 回実行単位の所要時間を測る
// プロセス spawn 分のオーバーヘッドが乗るが、perf 改善の相対比較目的なので許容

mod common;

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tempfile::TempDir;

use common::{MEDIUM_ROWS, SEED, SMALL_ROWS, gen_csv, write_file};

fn bench_mask(c: &mut Criterion) {
    let dir = TempDir::new().expect("tempdir");
    // 入力ファイルはベンチ外で 1 回だけ用意する
    let inputs: Vec<(usize, PathBuf)> = [SMALL_ROWS, MEDIUM_ROWS]
        .iter()
        .map(|&rows| {
            let path = dir.path().join(format!("mask-{}.csv", rows));
            write_file(&path, &gen_csv(rows, SEED));
            (rows, path)
        })
        .collect();
    // dry-run なので実際には書き込まれないが -o は必須
    let unused_output = dir.path().join("unused.csv");

    let mut group = c.benchmark_group("mask");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                let output = Command::cargo_bin("csv-ops")
                    .expect("csv-ops bin not found")
                    .args([
                        "mask",
                        "-i",
                        input.to_str().expect("input path"),
                        "-o",
                        unused_output.to_str().expect("output path"),
                        "-c",
                        "name",
                        "--dry-run",
                    ])
                    .output()
                    .expect("spawn failed");
                assert!(
                    output.status.success(),
                    "mask 実行に失敗: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_mask);
criterion_main!(benches);
