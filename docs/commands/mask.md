# csv-ops mask

指定したカラムを文字数保持でマスクする。元の値の文字数分だけマスク文字に
置き換えるため、桁数を保ったまま中身を隠せる。

## 書式

```
# CLI 引数モード
csv-ops mask -i <IN> -o <OUT> -c <列> [--mask-char <CHAR>]

# 設定ファイルモード
csv-ops mask -i <IN> -o <OUT> --config <FILE>
```

対象列は `-c <列>` か `--config` の **いずれか必須**。

## オプション

| オプション                  | 既定値   | 説明                                               |
| --------------------------- | -------- | -------------------------------------------------- |
| `-i`, `--input <IN>`        | -        | 入力ファイル                                       |
| `-o`, `--output <OUT>`      | -        | 出力ファイル                                       |
| `-c`, `--columns <COLUMNS>` | -        | 対象列（カンマ区切り、列名または列番号）           |
| `--config <FILE>`           | -        | 設定ファイル (TOML)。指定時は `-c` / `--mask-char` は無視 |
| `--mask-char <CHAR>`        | `*`      | マスク文字（先頭 1 文字を使用）                    |
| `--input-encoding <ENC>`    | `utf-8`  | 入力エンコーディング（utf-8 / shift_jis / euc-jp） |
| `--output-encoding <ENC>`   | `utf-8`  | 出力エンコーディング                               |
| `--delimiter <ALIAS>`       | `comma`  | 区切り文字（comma / tab / pipe / semicolon）       |
| `--no-headers`              | off      | ヘッダ行なし CSV として扱う                        |
| `--dry-run`                 | off      | 出力ファイルへ書き込まず統計のみ表示               |
| `--stats-format <FORMAT>`   | `text`   | 統計の出力形式（text / json）                      |
| `--stats-file <PATH>`       | 標準出力 | 統計の出力先ファイル                               |

`columns` は列名（文字列）と列番号（0 始まりの整数）を混在指定できる。
列番号は `--no-headers` のファイルでも使える。

## 設定ファイル

冒頭に `version = 1` が必須。

```toml
version = 1
columns = ["name", "email"]

[options]
mask_char = "#"
```

| キー                  | 必須 | 既定値 | 説明                                 |
| --------------------- | ---- | ------ | ------------------------------------ |
| `version`             | ○    | -      | 設定バージョン（`1` 固定）           |
| `columns`             | ○    | -      | マスク対象カラム（列名または列番号） |
| `options.mask_char`   |      | `*`    | マスク文字（先頭 1 文字を使用）      |

## 使用例

入力 `data.csv`:

```
name,email
田中,tanaka@example.com
佐藤,sato@example.com
```

実行:

```
csv-ops mask -i data.csv -o masked.csv -c name
```

出力 `masked.csv`:

```
name,email
**,tanaka@example.com
**,sato@example.com
```

### dry-run（統計のみ）

```
csv-ops mask -i data.csv -o masked.csv -c name --dry-run
```

## 備考

- 空セル / 空文字は変更しない。
- 文字数は Unicode のコードポイント単位で数える（半角・全角を同じ 1 文字として扱う）。
- ヘッダ行はマスク対象外（そのまま出力）。
- 統計は処理行数・マスク行数・マスクセル数を出力する。
