# csv-ops

[![CI](https://github.com/naka93-gh/csv-ops/actions/workflows/ci.yml/badge.svg)](https://github.com/naka93-gh/csv-ops/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

設定駆動のCSV 処理用 Rust 製 CLI ツール。

## インストール

### cargo（ソースから）

```
cargo install --git https://github.com/naka93-gh/csv-ops
```

### ソースをビルド

```
git clone https://github.com/naka93-gh/csv-ops
cd csv-ops
cargo build --release
# 生成物: target/release/csv-ops
```

## クイックスタート

### mask — 文字数保持マスキング

設定ファイル `config.toml`:

```toml
[[targets]]
filepath = "data.csv"
columns = ["name", "email"]
```

```
csv-ops mask -c config.toml   # data.csv → data_masked.csv
```

### replace — 一括置換

```
csv-ops replace -i in.csv -o out.csv -c status --from 未対応 --to open
```

### flag — パターン判定で真偽値カラムを追加

```
csv-ops flag -i in.csv -o out.csv -c city --pattern "東京|大阪" --out-col major
```

## ドキュメント

- [サブコマンド別の使い方](docs/commands/) — mask / replace / flag の詳細

## ライセンス

[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のデュアルライセンス。
