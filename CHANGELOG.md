# Changelog

csv-ops の変更履歴。

形式は [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に準拠し、
バージョニングは [Semantic Versioning](https://semver.org/lang/ja/) に従う
（v0.x 期間はマイナーで機能追加・破壊的変更の双方を許容する）。

## [0.1.1] - 2026-05-25

### 破壊的変更

- Stats 出力フィールド名・ラベルを全サブコマンドで統一 (#4)。旧名 (`rows_masked` / `tie_rows` 等) でパースする script は要修正
- `info` の `--format` を `--stats-format` に変更 (#6)

### 修正

- 区切り文字 / エンコーディング / 行終端の自動判定を堅牢化 (#3)

### 性能改善

- replace / mask のホットパスを高速化 (#7)

[0.1.1]: https://github.com/naka93-gh/csv-ops/releases/tag/v0.1.1

## [0.1.0] - 2026-05-23

### 追加

- `mask` サブコマンド — 文字数保持マスキング（設定ファイル / CLI 引数モード）
- `replace` サブコマンド — 列指定 / 全カラム横断の一括置換（単純置換・正規表現、衝突検出、統計出力）
- `flag` サブコマンド — 指定列のパターン判定による真偽値カラムの追加（複数列・複数ルール対応）
- `extract` サブコマンド — 指定列からパターンマッチした文字列を抽出してカラムを追加（キャプチャグループ・複数マッチ連結・複数ルール対応）
- `similarity` サブコマンド — 指定列を辞書とベストマッチしてマッチ名・スコアのカラムを追加（4 種の類似度アルゴリズム、日本語正規化、CSV / TOML 辞書、複数ルール対応）
- 類似度アルゴリズム — `--algorithm` で levenshtein / damerau / jaro-winkler / dice を選択
- `convert` サブコマンド — エンコーディング・区切り文字の変換（utf-8 / shift_jis / euc-jp、行終端の保持、デコード失敗時はエラー停止）
- `info` サブコマンド — CSV のエンコーディング・区切り文字・行終端・行数・列数・ヘッダの表示（text / json、エンコーディングと区切り文字の自動判定）
- 入出力エンコーディング対応 — UTF-8 / Shift_JIS / EUC-JP
- `--input-encoding auto` — 全 transform コマンドで入力エンコーディングの自動判定に対応
- GitHub Actions による CI（fmt / clippy / test / build）
- Dependabot による依存更新

[0.1.0]: https://github.com/naka93-gh/csv-ops/releases/tag/v0.1.0
