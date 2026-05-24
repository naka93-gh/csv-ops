# csv-ops replace

指定した列、または全カラムを横断して文字列を一括置換する。単純な文字列置換と
正規表現置換の両方に対応する。

## 書式

```
# 設定ファイルモード（複数ルール）
csv-ops replace -i <IN> -o <OUT> (-c <列> | --all-columns) --config <FILE>

# CLI 引数モード（1 ルール）
csv-ops replace -i <IN> -o <OUT> (-c <列> | --all-columns) --from <S> --to <S> [--regex]
```

対象列は `-c <列>` か `--all-columns` の **いずれか必須**。ルールは `--config`
（複数ルール）か `--from` / `--to`（1 ルール）で指定する。

## オプション

| オプション                  | 既定値   | 説明                                         |
| --------------------------- | -------- | -------------------------------------------- |
| `-i`, `--input <IN>`        | -        | 入力ファイル                                 |
| `-o`, `--output <OUT>`      | -        | 出力ファイル                                 |
| `-c`, `--columns <COLUMNS>` | -        | 対象列（カンマ区切り、列名または列番号）     |
| `--all-columns`             | -        | 全カラムを対象にする（`-c` と排他）          |
| `--config <FILE>`           | -        | 設定ファイル (TOML)。指定時は `--from/--to` は無視 |
| `--from <S>`                | -        | 置換元（CLI 引数モード）                     |
| `--to <S>`                  | -        | 置換先（CLI 引数モード）                     |
| `--regex`                   | off      | `--from` を正規表現として扱う                |
| `--case-insensitive`        | off      | 大文字小文字を区別しない                     |
| `--input-encoding <ENC>`    | `utf-8`  | 入力エンコーディング（utf-8 / shift_jis / euc-jp / auto） |
| `--output-encoding <ENC>`   | `utf-8`  | 出力エンコーディング                         |
| `--delimiter <ALIAS>`       | `comma`  | 区切り文字（comma / tab / pipe / semicolon） |
| `--no-headers`              | off      | ヘッダ行なし CSV として扱う                  |
| `--dry-run`                 | off      | 出力ファイルへ書き込まず統計のみ表示         |
| `--stats-format <FORMAT>`   | `text`   | 統計の出力形式（text / json）                |
| `--stats-file <PATH>`       | 標準出力 | 統計の出力先ファイル                         |

## 設定ファイル

冒頭に `version = 1` が必須。`[[rules]]` を複数並べられる。

```toml
version = 1

[options]
case_insensitive = false

# 単純文字列置換
[[rules]]
name = "status_normalize"
from = "未対応"
to = "open"

# 正規表現置換（regex = true）
[[rules]]
name = "bug_prefix"
pattern = '^bug:\s*'
replacement = "[BUG] "
regex = true
```

- 単純置換は `from` / `to`、正規表現置換は `pattern` / `replacement` + `regex = true`。
- `name` は任意（エラーメッセージや統計でルールを特定するのに使う）。

## 置換の挙動

- 全ルールを元のセルに対して同時評価する（**連鎖なし**＝置換後の文字列に別ルールが再マッチしない）。
- ルール同士のマッチ範囲が重なると衝突として扱う。
  - 単純置換ルール間の重複・部分文字列関係は設定読み込み時にエラー停止。
  - 正規表現が絡む衝突は実行時にセル単位でエラー停止。
- 単純置換は部分一致（完全一致モードはない）。

## 使用例

### CLI 引数モード（単純置換）

入力 `in.csv`:

```
name,status
田中,未対応
佐藤,対応中
```

実行:

```
csv-ops replace -i in.csv -o out.csv -c status --from 未対応 --to open
```

出力 `out.csv`:

```
name,status
田中,open
佐藤,対応中
```

### 全カラム横断

```
csv-ops replace -i in.csv -o out.csv --all-columns --from 株式会社 --to （株）
```

### dry-run（統計のみ）

```
csv-ops replace -i in.csv -o out.csv -c status --config rules.toml --dry-run
```

## 備考

- ヘッダ行は置換対象外（そのまま出力）。
- 正規表現の `replacement` は固定文字列として扱われる。`$1` などのキャプチャ参照展開は現状未対応。
- 統計は処理行数・ヒット行数（置換が入った行数）・ヒット総数（総置換回数）と、ルール毎のヒット（影響行数）・マッチ（置換回数）を出力する。
