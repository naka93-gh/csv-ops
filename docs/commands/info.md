# csv-ops info

CSV ファイルを解析し、エンコーディング・区切り文字・行数・列数などの情報を表示する。
ファイルは変更しない（読み取り専用）。

## 書式

```
csv-ops info -i <IN> [options]
```

設定ファイルはない（CLI 引数のみ）。

## オプション

| オプション                  | 既定値   | 説明                                               |
| --------------------------- | -------- | -------------------------------------------------- |
| `-i`, `--input <IN>`        | -        | 入力ファイル                                       |
| `--format <FORMAT>`         | `text`   | 出力形式（text / json）                            |
| `--input-delimiter <ALIAS>` | 自動判定 | 区切り文字（comma / tab / pipe / semicolon）       |
| `--input-quote <CHAR>`      | `"`      | クォート文字                                       |

`--input-delimiter` を省略した場合は、ヘッダ行に最も多く出現する候補
（comma / tab / pipe / semicolon）を区切り文字として自動判定する。

## 表示項目

| 項目          | 説明                                               |
| ------------- | -------------------------------------------------- |
| `File`        | ファイル名                                         |
| `Size`        | ファイルサイズ                                     |
| `Encoding`    | 推定エンコーディング（BOM 有無を含む）             |
| `Delimiter`   | 区切り文字                                         |
| `Quote`       | クォート文字                                       |
| `Line ending` | 行終端（LF / CRLF / mixed / none）                 |
| `Rows`        | データ行数（ヘッダ除く）                           |
| `Columns`     | 列数                                               |
| `Headers`     | ヘッダのカラム名一覧                               |

## エンコーディング推定

- UTF-8 BOM があれば UTF-8（BOM あり）。
- BOM がなければ UTF-8 として厳密デコードを試し、成功すれば UTF-8、失敗すれば Shift_JIS とみなす。
- EUC-JP は自動判定の対象外。

## 使用例

入力 `data.csv`:

```
氏名,年齢,部署
田中,30,営業
佐藤,25,開発
```

実行:

```
csv-ops info -i data.csv
```

出力:

```
File:        data.csv
Size:        45 B (45 bytes)
Encoding:    UTF-8 (no BOM)
Delimiter:   ,
Quote:       "
Line ending: LF
Rows:        2 (excluding header)
Columns:     3
Headers:     氏名, 年齢, 部署
```

### JSON 形式で出力

```
csv-ops info -i data.csv --format json
```

## 備考

- 解析のためファイル全体を読み込む。
- フィールド数がそろわない行も止めずに数える。
