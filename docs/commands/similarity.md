# csv-ops similarity

指定した列の値を辞書とベストマッチさせ、最も近いエントリの名前（`matched_name`）と
類似度スコア（`score`）の列を CSV の末尾に追加する。元の列は一切変更しない。

表記揺れのある地名・会社名・固有名詞を、正規の名称（canonical）へ寄せる用途を想定する。

## 書式

```
# CLI 引数モード（1 ルール）
csv-ops similarity -i <IN> -o <OUT> -c <列> --dict <FILE>

# 設定ファイルモード（複数ルール）
csv-ops similarity -i <IN> -o <OUT> --config <FILE>
```

対象列は `-c` で 1 列だけ指定する。

## マッチのルール

- **アルゴリズム** — レーベンシュタイン距離（char 単位）。類似度は `1 - 距離 / 長い方の文字数` で 0.0-1.0。
- **比較対象** — 辞書の各エントリの canonical と alias すべてと比較し、最良スコアのエントリを採用する。alias にマッチしても出力されるのは canonical。
- **しきい値** — 最良スコアが `--threshold`（既定 `0.7`）以上なら canonical、未満なら `<no match>` を出力する。`score` 列にはどちらの場合も実値を出す。
- **同点** — 最良スコアを複数エントリが取った場合は辞書の記述順で先勝ちし、警告を標準エラー出力へ出す。
- **正規化** — 入力セルと辞書の候補に同じ正規化を適用してから比較する（後述）。

## オプション

| オプション                | 既定値                     | 説明                                                                 |
| ------------------------- | -------------------------- | -------------------------------------------------------------------- |
| `-i`, `--input <IN>`      | -                          | 入力ファイル                                                         |
| `-o`, `--output <OUT>`    | -                          | 出力ファイル                                                         |
| `-c`, `--column <COLUMN>` | -                          | 対象列（1 列、列名または列番号）                                     |
| `--dict <FILE>`           | -                          | 辞書ファイル（CLI 引数モード）                                       |
| `--out-col <NAME>`        | `matched_name`             | マッチ名を出力する列名                                               |
| `--score-col <NAME>`      | `score`                    | スコアを出力する列名                                                 |
| `--threshold <FLOAT>`     | `0.7`                      | マッチとみなすしきい値（0.0-1.0）                                    |
| `--algorithm <NAME>`      | `levenshtein`              | 類似度アルゴリズム（後述）                                           |
| `--normalize <LIST>`      | `nfkc,casefold,whitespace` | 正規化オプション（カンマ区切り）                                     |
| `--config <FILE>`         | -                          | 設定ファイル (TOML)。指定時は `-c` / `--dict` 等は無視               |
| `--input-encoding <ENC>`  | `utf-8`                    | 入力エンコーディング（utf-8 / shift_jis / euc-jp / auto）            |
| `--output-encoding <ENC>` | `utf-8`                    | 出力エンコーディング                                                 |
| `--delimiter <ALIAS>`     | `comma`                    | 区切り文字（comma / tab / pipe / semicolon）。CSV 形式の辞書にも適用 |
| `--no-headers`            | off                        | ヘッダ行なし CSV として扱う                                          |
| `--dry-run`               | off                        | 出力ファイルへ書き込まず統計のみ表示                                 |
| `--stats-format <FORMAT>` | `text`                     | 統計の出力形式（text / json）                                        |
| `--stats-file <PATH>`     | 標準出力                   | 統計の出力先ファイル                                                 |

## アルゴリズム

`--algorithm`（設定ファイルでは rule 毎の `algorithm`）で類似度アルゴリズムを選ぶ。
いずれも 0.0-1.0 のスコアを返し、char（Unicode スカラ）単位で計算する。

| 名前           | 内容                                                                   |
| -------------- | ---------------------------------------------------------------------- |
| `levenshtein`  | レーベンシュタイン距離ベース（既定）。汎用                             |
| `damerau`      | Damerau-Levenshtein（OSA 変種）。隣接 2 文字の転置に強い（打ち間違い） |
| `jaro-winkler` | Jaro-Winkler。先頭一致を重視。会社名・人名など短い固有名詞に強い       |
| `dice`         | Sørensen-Dice（bigram）。語順違い・部分一致に強い                      |

## 正規化オプション

`--normalize` には次の名前をカンマ区切りで指定する。指定順に関わらず固定順で適用される。

| 名前         | 内容                                      |
| ------------ | ----------------------------------------- |
| `nfkc`       | NFKC 正規化（全角英数記号 → 半角 など）   |
| `fullwidth`  | 半角カナ → 全角カナ（濁点・半濁点を合成） |
| `prolonged`  | 長音記号類（`ー ｰ — ― －`）を統一         |
| `kana`       | ひらがな → カタカナ                       |
| `casefold`   | 大文字小文字を統一（小文字へ）            |
| `whitespace` | 空白を除去                                |

## 辞書ファイル

`.toml` 拡張子なら TOML、それ以外は CSV として読む。エンコーディング（UTF-8 / Shift_JIS）は
自動判定する。canonical の重複、同一 alias が複数の canonical に割り当てられている場合は
ロード時にエラー停止する。

### CSV 辞書

1 列目が canonical、2 列目以降が alias（空欄は無視）。ヘッダ行あり。区切り文字は本体 CSV と
同じ `--delimiter` を使う。

```csv
canonical,alias1,alias2
東京都,東京,Tokyo
大阪府,大阪,Osaka
```

### TOML 辞書

冒頭に `version = 1` が必須。

```toml
version = 1

[[entries]]
canonical = "東京都"
aliases = ["東京", "Tokyo"]

[[entries]]
canonical = "大阪府"
aliases = ["大阪", "Osaka"]
```

## 設定ファイル

冒頭に `version = 1` が必須。`[[rules]]` を複数並べると、その数だけ列ペアが追加される。

| キー        | 必須 | 既定値                             | 説明                             |
| ----------- | ---- | ---------------------------------- | -------------------------------- |
| `column`    | ○    | -                                  | 対象列（1 列、列名または列番号） |
| `dict`      | ○    | -                                  | 辞書ファイルのパス               |
| `out_col`   | ○    | -                                  | マッチ名を出力する列名           |
| `score_col` | ○    | -                                  | スコアを出力する列名             |
| `threshold` |      | `0.7`                              | マッチとみなすしきい値           |
| `algorithm` |      | `levenshtein`                      | 類似度アルゴリズム               |
| `normalize` |      | `["nfkc","casefold","whitespace"]` | 正規化オプションのリスト         |

```toml
version = 1

[[rules]]
column = "city"
dict = "city-dict.csv"
out_col = "city_matched"
score_col = "city_score"
threshold = 0.7
normalize = ["nfkc", "casefold", "whitespace"]

[[rules]]
column = "prefecture"
dict = "pref-dict.toml"
out_col = "pref_matched"
score_col = "pref_score"
threshold = 0.8
algorithm = "jaro-winkler"
normalize = ["nfkc", "casefold", "whitespace", "kana", "fullwidth", "prolonged"]
```

## 使用例

入力 `in.csv`:

```
id,name
1,東京
2,xyz
```

辞書 `dict.csv`:

```
canonical,alias1
東京都,東京
大阪府,大阪
```

実行:

```
csv-ops similarity -i in.csv -o out.csv -c name --dict dict.csv
```

出力 `out.csv`:

```
id,name,matched_name,score
1,東京,東京都,1.0000
2,xyz,<no match>,0.0000
```

## 備考

- 元の列は変更せず、結果は常に末尾へ `matched_name` / `score` の 2 列として追加する。
- `out_col` / `score_col` が既存カラム名や他ルールと重複する場合はエラー停止する。
- `--no-headers` 指定時はヘッダ行がないため列名は使われず、列が末尾に追加されるだけになる。
- 統計は処理行数と、ルール毎のマッチ件数・非マッチ件数・同点件数を出力する。
- 自動判定は先頭 64KB を見るため、辞書が大きく先頭が ASCII のみの場合などは取り違える可能性がある。
