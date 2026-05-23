// replace サブコマンドのベンチマーク
// 3 ケース × 2 サイズ:
//   - simple_1rule : 単純置換 1 ルール
//   - simple_10rules: 単純置換 10 ルール (A-7 Aho-Corasick 化の効果測定対象)
//   - regex_1rule  : 正規表現 1 ルール

mod common;

use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use csv_ops::ColumnRef;
use csv_ops::replace::{ColumnTarget, ReplaceRequest, RuleSource};
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

    // 単純置換 1 ルール
    let mut group = c.benchmark_group("replace/simple_1rule");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                csv_ops::replace::run(ReplaceRequest {
                    rules: RuleSource::Inline {
                        from: "old_".into(),
                        to: "new_".into(),
                        regex: false,
                    },
                    input: input.clone(),
                    output: PathBuf::from("unused.csv"),
                    input_encoding: "utf-8".into(),
                    output_encoding: "utf-8".into(),
                    delimiter: b',',
                    has_headers: true,
                    case_insensitive: false,
                    columns: ColumnTarget::Specified(vec![ColumnRef::Name("note".into())]),
                    dry_run: true,
                })
                .expect("replace 実行に失敗");
            });
        });
    }
    group.finish();

    // 単純置換 10 ルール
    let mut group = c.benchmark_group("replace/simple_10rules");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                csv_ops::replace::run(ReplaceRequest {
                    rules: RuleSource::Config(config_10.clone()),
                    input: input.clone(),
                    output: PathBuf::from("unused.csv"),
                    input_encoding: "utf-8".into(),
                    output_encoding: "utf-8".into(),
                    delimiter: b',',
                    has_headers: true,
                    case_insensitive: false,
                    columns: ColumnTarget::Specified(vec![ColumnRef::Name("note".into())]),
                    dry_run: true,
                })
                .expect("replace 実行に失敗");
            });
        });
    }
    group.finish();

    // 正規表現 1 ルール
    let mut group = c.benchmark_group("replace/regex_1rule");
    for (rows, input) in &inputs {
        group.bench_with_input(BenchmarkId::from_parameter(rows), rows, |b, _| {
            b.iter(|| {
                csv_ops::replace::run(ReplaceRequest {
                    rules: RuleSource::Inline {
                        from: r"old_\w+".into(),
                        to: "new".into(),
                        regex: true,
                    },
                    input: input.clone(),
                    output: PathBuf::from("unused.csv"),
                    input_encoding: "utf-8".into(),
                    output_encoding: "utf-8".into(),
                    delimiter: b',',
                    has_headers: true,
                    case_insensitive: false,
                    columns: ColumnTarget::Specified(vec![ColumnRef::Name("note".into())]),
                    dry_run: true,
                })
                .expect("replace 実行に失敗");
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_replace);
criterion_main!(benches);
