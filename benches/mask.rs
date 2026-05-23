// mask サブコマンドのベンチマーク
// 計測対象: csv_ops::mask::run を dry_run = true で呼び、純粋な変換時間を測る

mod common;

use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use csv_ops::ColumnRef;
use csv_ops::mask::{MaskRequest, MaskSource};
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

    let mut group = c.benchmark_group("mask");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                csv_ops::mask::run(MaskRequest {
                    source: MaskSource::Inline {
                        columns: vec![ColumnRef::Name("name".to_string())],
                        mask_char: '*',
                    },
                    input: input.clone(),
                    // dry_run なので使われないが型として必要
                    output: PathBuf::from("unused.csv"),
                    input_encoding: "utf-8".into(),
                    output_encoding: "utf-8".into(),
                    delimiter: b',',
                    has_headers: true,
                    dry_run: true,
                })
                .expect("mask 実行に失敗");
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_mask);
criterion_main!(benches);
