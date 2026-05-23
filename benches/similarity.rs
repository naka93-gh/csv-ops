// similarity サブコマンドのベンチマーク
// (辞書 10 / 100 エントリ) × (入力 小 1K 行 / 中 10K 行)
//
// 中ケースは辞書 × 入力で計算量が爆発するため、SIMILARITY_MEDIUM_ROWS (10K) を使う。
// 100K 行 + 1000 辞書のような大規模ケースは並列化前提のため v0.2.0 以降で扱う。

mod common;

use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use csv_ops::ColumnRef;
use csv_ops::similarity::{RuleSource, SimilarityRequest};
use tempfile::TempDir;

use common::{
    SEED, SIMILARITY_MEDIUM_ROWS, SMALL_ROWS, gen_dict_csv, gen_similarity_input_csv, write_file,
};

fn bench_similarity(c: &mut Criterion) {
    let dir = TempDir::new().expect("tempdir");

    // 辞書ファイル (10 / 100 エントリ)
    let dict_paths: Vec<(usize, PathBuf)> = [10usize, 100]
        .iter()
        .map(|&n| {
            let path = dir.path().join(format!("dict-{}.csv", n));
            write_file(&path, &gen_dict_csv(n));
            (n, path)
        })
        .collect();

    // 入力ファイル (1K 行 / 10K 行)
    // 辞書 100 件想定で値を散らばらせる (10 件の辞書時には未マッチが増えるだけ)
    let inputs: Vec<(usize, PathBuf)> = [SMALL_ROWS, SIMILARITY_MEDIUM_ROWS]
        .iter()
        .map(|&rows| {
            let path = dir.path().join(format!("input-{}.csv", rows));
            write_file(&path, &gen_similarity_input_csv(rows, 100, SEED));
            (rows, path)
        })
        .collect();

    for (dict_n, dict_path) in &dict_paths {
        let mut group = c.benchmark_group(format!("similarity/dict{}", dict_n));
        // 中ケースは 1 反復が長くなりやすいのでサンプル数と計測時間を絞る
        group.sample_size(10);
        group.warm_up_time(Duration::from_secs(1));
        group.measurement_time(Duration::from_secs(20));

        for (rows, input) in &inputs {
            group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
                b.iter(|| {
                    csv_ops::similarity::run(SimilarityRequest {
                        rules: RuleSource::Inline {
                            column: ColumnRef::Name("region".into()),
                            dict: dict_path.clone(),
                            out_col: "matched".into(),
                            score_col: "score".into(),
                            threshold: 0.6,
                            normalize: vec!["nfkc".into(), "casefold".into(), "whitespace".into()],
                            algorithm: "levenshtein".into(),
                        },
                        input: input.clone(),
                        output: PathBuf::from("unused.csv"),
                        input_encoding: "utf-8".into(),
                        output_encoding: "utf-8".into(),
                        delimiter: b',',
                        has_headers: true,
                        dry_run: true,
                    })
                    .expect("similarity 実行に失敗");
                });
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_similarity);
criterion_main!(benches);
