# csv-ops mask

指定したカラムを文字数保持でマスクする。元の値の文字数分だけ `*` に置き換えるため、
桁数を保ったまま中身を隠せる。

## 書式

```
csv-ops mask -c <CONFIG>
csv-ops mask --config <CONFIG>
```

mask は **設定ファイル専用**のサブコマンドで、入力ファイルやカラムを CLI 引数で
直接指定するモードはない（複数ファイルの業務バッチ処理を主用途とするため）。

## オプション

| オプション              | 既定値        | 説明                       |
| ----------------------- | ------------- | -------------------------- |
| `-c`, `--config <FILE>` | `config.toml` | 設定ファイル (TOML) のパス |

## 出力先

出力ファイルは指定できない。入力ファイルと同じディレクトリに
`<元の名前>_masked.<拡張子>` が自動生成される。

```
data.csv  →  data_masked.csv
```

## 設定ファイル

`[[targets]]` の配列で、マスク対象ファイルを 1 つ以上指定する。

| キー          | 必須 | 既定値  | 説明                                               |
| ------------- | ---- | ------- | -------------------------------------------------- |
| `filepath`    | ○    | -       | 入力ファイルのパス                                 |
| `columns`     | ○    | -       | マスク対象カラム（列名または列番号）               |
| `delimiter`   |      | `,`     | 区切り文字                                         |
| `has_headers` |      | `true`  | ヘッダ行の有無                                     |
| `encoding`    |      | `utf-8` | 入力エンコーディング（utf-8 / shift_jis / euc-jp） |

`columns` は列名（文字列）と列番号（0 始まりの整数）を混在指定できる。
列番号は `has_headers = false` のファイルでも使える。

```toml
[[targets]]
filepath = "data.csv"
columns = ["name", "email"]

[[targets]]
filepath = "users.csv"
columns = [0, 2]
encoding = "shift_jis"
has_headers = true
```

## 使用例

設定ファイル `config.toml`:

```toml
[[targets]]
filepath = "data.csv"
columns = ["name"]
```

入力 `data.csv`:

```
name,email
田中,tanaka@example.com
佐藤,sato@example.com
```

実行:

```
csv-ops mask -c config.toml
```

出力 `data_masked.csv`:

```
name,email
**,tanaka@example.com
**,sato@example.com
```

## 備考

- 空セル / 空文字は変更しない。
- 文字数は UTF-8 のコードポイント単位で数える（半角・全角を同じ 1 文字として扱う）。
- マスク文字は `*` 固定。
