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
| `--input-encoding <ENC>`     | `utf-8` | 入力エンコーディング（utf-8 / shift_jis / euc-jp） |
| `--output-encoding <ENC>`    | `utf-8` | 出力エンコーディング（utf-8 / shift_jis / euc-jp） |
| `--input-delimiter <ALIAS>`  | `comma` | 入力区切り文字（comma / tab / pipe / semicolon）   |
| `--output-delimiter <ALIAS>` | `comma` | 出力区切り文字（comma / tab / pipe / semicolon）   |

区切り文字はエイリアス指定のみ。任意の 1 文字指定には対応しない。

## 挙動

- **行終端** — 入力の先頭で CRLF / LF を検出し、出力をその終端で統一する。
- **入力 BOM** — UTF-8 BOM があれば自動で除去する。出力に BOM は付与しない。
- **デコード失敗** — 指定した入力エンコーディングで読めないバイト列があればエラー停止する（不正バイトを黙って置換しない）。
- **クォート** — 必要時のみクォートする（quote-minimal）。
- エンコーディングの自動判定（`auto`）は持たない。エンコーディングが不明な場合は `csv-ops info`（実装予定）で確認する。

## 使用例

### Shift_JIS から UTF-8 へ

```
csv-ops convert -i sjis.csv -o utf8.csv --input-encoding shift_jis
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
