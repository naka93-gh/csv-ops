# csv-ops flag

指定した列を正規表現で判定し、真偽値（true / false）の列を CSV の末尾に追加する。
元の列は一切変更しない。

## 書式

```
# CLI 引数モード（1 ルール）
csv-ops flag -i <IN> -o <OUT> -c <列> --pattern <RE> --out-col <NAME>

# 設定ファイルモード（複数ルール）
csv-ops flag -i <IN> -o <OUT> --config <FILE>
```

対象列は `-c` で必ず指定する（replace のような全カラム横断モードはない）。
複数列を指定した場合、**どれか 1 列でもマッチすれば true** になる。

## オプション

| オプション                  | 既定値   | 説明                                         |
| --------------------------- | -------- | -------------------------------------------- |
| `-i`, `--input <IN>`        | -        | 入力ファイル                                 |
| `-o`, `--output <OUT>`      | -        | 出力ファイル                                 |
| `-c`, `--columns <COLUMNS>` | -        | 対象列（カンマ区切り、列名または列番号）     |
| `--pattern <RE>`            | -        | マッチ判定の正規表現（CLI 引数モード）       |
| `--out-col <NAME>`          | -        | 追加する列の名前（CLI 引数モード）           |
| `--config <FILE>`           | -        | 設定ファイル (TOML)。指定時は `--pattern` 等は無視 |
| `--input-encoding <ENC>`    | `utf-8`  | 入力エンコーディング（utf-8 / shift_jis / euc-jp） |
| `--output-encoding <ENC>`   | `utf-8`  | 出力エンコーディング                         |
| `--delimiter <D>`           | `,`      | 区切り文字                                   |
| `--no-headers`              | off      | ヘッダ行なし CSV として扱う                  |
| `--dry-run`                 | off      | 出力ファイルへ書き込まず統計のみ表示         |
| `--stats-format <FORMAT>`   | `text`   | 統計の出力形式（text / json）                |
| `--stats-file <PATH>`       | 標準出力 | 統計の出力先ファイル                         |

大文字小文字を区別しない判定は、正規表現側で `(?i)` を付けて指定する（例: `(?i)tokyo`）。

## 設定ファイル

冒頭に `version = 1` が必須。`[[rules]]` を複数並べると、その数だけ列が追加される。

| キー          | 必須 | 既定値  | 説明                                          |
| ------------- | ---- | ------- | --------------------------------------------- |
| `column`      | △    | -       | 対象列（単一）。`columns` とどちらか一方      |
| `columns`     | △    | -       | 対象列（複数）。`column` とどちらか一方       |
| `pattern`     | ○    | -       | マッチ判定の正規表現                          |
| `out_col`     | ○    | -       | 追加する列の名前                              |
| `true_value`  |      | `true`  | マッチ時に書き込む値                          |
| `false_value` |      | `false` | 非マッチ時に書き込む値                        |

```toml
version = 1

[[rules]]
column = "city"
pattern = '東京|大阪|京都'
out_col = "has_major_city"
true_value = "○"
false_value = "×"

[[rules]]
columns = ["name", "address"]
pattern = '田中'
out_col = "tanaka_match"
```

## 使用例

### CLI 引数モード

入力 `in.csv`:

```
name,city
田中,東京
佐藤,福岡
```

実行:

```
csv-ops flag -i in.csv -o out.csv -c city --pattern "東京|大阪" --out-col major
```

出力 `out.csv`:

```
name,city,major
田中,東京,true
佐藤,福岡,false
```

### 設定ファイルモード（複数ルール）

`rules.toml` に 2 ルール定義すると、列が 2 つ追加される。

```
name,city,has_major_city,tanaka_match
田中,東京,○,true
```

## 備考

- 元の列は変更せず、判定結果は常に末尾へ新しい列として追加する。
- `out_col` が既存のカラム名や他ルールの `out_col` と重複する場合はエラー停止する。
- `--no-headers` 指定時はヘッダ行がないため `out_col` 名は使われず、列が末尾に追加されるだけになる。
- 統計は処理行数と、ルール（`out_col`）毎の true 件数を出力する。
