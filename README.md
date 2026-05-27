# csv-ops

[![CI](https://github.com/naka93-gh/csv-ops/actions/workflows/ci.yml/badge.svg)](https://github.com/naka93-gh/csv-ops/actions/workflows/ci.yml)
[![Release](https://github.com/naka93-gh/csv-ops/actions/workflows/release.yml/badge.svg)](https://github.com/naka93-gh/csv-ops/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/naka93-gh/csv-ops)](https://github.com/naka93-gh/csv-ops/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/naka93-gh/csv-ops/total)](https://github.com/naka93-gh/csv-ops/releases)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-blue)](https://www.rust-lang.org/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

CSV 処理用の Rust 製 CLI ツール。
Shift_JIS や表記揺れをどうにかするために作っています。

## 特徴

- Shift_JIS / EUC-JP の CSV をそのまま読み書きできる（指定 or 自動判定）
- 業務でよくやる処理を 7 コマンドにまとめてある — マスキング・置換・フラグ列追加・抽出・辞書マッチ・フォーマット変換・情報表示
- CLI 引数でも TOML 設定ファイルでも書ける。複数ルールは設定ファイルにまとめると楽
- 表記揺れを類似度マッチで吸収できる（4 種のアルゴリズム + 日本語正規化）
- 大きい CSV もストリーミングで処理。書き途中で死んでも壊れたファイルが残らない（アトミック出力）
- リリースバイナリは約 2 MB、依存も小さめ

## インストール

### バイナリダウンロード

[Releases](https://github.com/naka93-gh/csv-ops/releases/latest) から OS / アーキテクチャに合うアーカイブを落として展開してください。Linux / macOS (Intel / Apple Silicon) / Windows のバイナリを配布しています。

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

```
csv-ops mask -i data.csv -o masked.csv -c name,email
```

### replace — 一括置換

```
csv-ops replace -i in.csv -o out.csv -c status --from 未対応 --to open
```

### flag — パターン判定で真偽値カラムを追加

```
csv-ops flag -i in.csv -o out.csv -c city --pattern "東京|大阪" --out-col major
```

### extract — パターンにマッチした文字列を抽出してカラムを追加

```
csv-ops extract -i in.csv -o out.csv -c memo --pattern "\d{2,4}-\d{4}-\d{4}" --out-col phone
```

### similarity — 辞書とのベストマッチでカラムを追加

```
csv-ops similarity -i in.csv -o out.csv -c city --dict city-dict.csv
```

### convert — エンコーディング・区切り文字の変換

```
csv-ops convert -i sjis.csv -o utf8.csv --input-encoding shift_jis

# 入力エンコーディングを自動判定する場合
csv-ops convert -i unknown.csv -o utf8.csv --input-encoding auto
```

### info — CSV の情報表示

```
csv-ops info -i data.csv
```

## ドキュメント

- [サブコマンド別の使い方](docs/commands/) — mask / replace / flag / extract / similarity / convert / info の詳細

## ライセンス

[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のデュアルライセンス。
