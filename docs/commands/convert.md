# csv-ops convert

CSV のエンコーディングと区切り文字だけを変換して素通しする。
セルの内容は変更せず、ヘッダ行も含めて全行をそのまま通す。

## 書式

```
csv-ops convert -i <IN> -o <OUT> [options]
```

設定ファイルはない（CLI 引数のみ）。

## オプション

| オプション                   | 既定値  | 説明                                               |
| ---------------------------- | ------- | -------------------------------------------------- |
| `-i`, `--input <IN>`         | -       | 入力ファイル                                       |
| `-o`, `--output <OUT>`       | -       | 出力ファイル                                       |
| `--input-encoding <ENC>`     | `utf-8` | 入力エンコーディング（utf-8 / shift_jis / euc-jp / auto） |
| `--output-encoding <ENC>`    | `utf-8` | 出力エンコーディング（utf-8 / shift_jis / euc-jp） |
| `--input-delimiter <ALIAS>`  | `comma` | 入力区切り文字（comma / tab / pipe / semicolon）   |
| `--output-delimiter <ALIAS>` | `comma` | 出力区切り文字（comma / tab / pipe / semicolon）   |
| `--dry-run`                  | off      | 出力ファイルへ書き込まず統計のみ表示               |
| `--stats-format <FORMAT>`    | `text`   | 統計の出力形式（text / json）                      |
| `--stats-file <PATH>`        | 標準出力 | 統計の出力先ファイル                               |

区切り文字はエイリアス指定のみ。任意の 1 文字指定には対応しない。

## 挙動

- **行終端** — 入力の先頭で CRLF / LF を検出し、出力をその終端で統一する。
- **入力 BOM** — UTF-8 BOM があれば自動で除去する。出力に BOM は付与しない。
- **デコード失敗** — 指定した入力エンコーディングで読めないバイト列があればエラー停止する（不正バイトを黙って置換しない）。
- **クォート** — 必要時のみクォートする（quote-minimal）。
- **エンコーディング自動判定** — `--input-encoding auto` を指定すると、入力ファイル先頭から UTF-8 / Shift_JIS を判定する。先頭 64KB のみを見るため、先頭が ASCII のみで後半に非 UTF-8 バイトが現れるファイルでは取り違える場合がある（EUC-JP は自動判定の対象外）。確実に判定したい場合は `csv-ops info` で確認する。

## 使用例

### Shift_JIS から UTF-8 へ

```
csv-ops convert -i sjis.csv -o utf8.csv --input-encoding shift_jis
```

### エンコーディングを自動判定して UTF-8 へ

```
csv-ops convert -i unknown.csv -o utf8.csv --input-encoding auto
```

### 区切り文字をカンマからタブへ

```
csv-ops convert -i in.csv -o out.tsv --output-delimiter tab
```

### エンコーディングと区切り文字を同時に変換

```
csv-ops convert -i sjis.csv -o out.tsv \
  --input-encoding shift_jis --output-delimiter tab
```

## 備考

- セルの内容・列構成・行数は変えない。エンコーディングと区切り文字のみ変換する。
- フィールド数がそろわない行もそのまま通す。
- 統計は変換した行数のみを出力する（ヘッダ行も 1 行として数える）。
- 統計フォーマットは他コマンドと共通の `Stats` 型を使うため、`rows_changed` / `changes_total` は仕様上常に `0` で出る。convert にはヒット概念がないため意味を持たず、形式統一のための定数値として読み流してよい（`rows_processed` のみ実数）。
