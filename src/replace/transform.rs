// 文字列置換の本体
// 各セルに対し全ルールを並行評価 (連鎖なし)、マッチ位置の重複検出後に後ろから置換する

use csv::StringRecord;

use crate::error::TransformError;

use super::rule::CompiledRule;

/// 置換処理本体
pub(crate) struct ReplaceTransform {
    // ルール
    rules: Vec<CompiledRule>,
    // 大文字小文字の判別
    case_insensitive: bool,
}

/// 置換対象の列
/// run 側で ColumnRef を解決済みインデックスに変換してから渡す
pub(crate) enum TargetColumns {
    /// 全カラム横断 (--all-columns)
    All,
    /// 解決済みの対象列インデックス (-c 指定)
    Indices(Vec<usize>),
}

impl TargetColumns {
    /// col_index が置換対象かどうか
    fn includes(&self, col_index: usize) -> bool {
        match self {
            TargetColumns::All => true,
            TargetColumns::Indices(idx) => idx.contains(&col_index),
        }
    }
}

impl ReplaceTransform {
    /// コンストラクタ
    /// Config パース / CLI 引数いずれの経路でも、最終的にここで組み立てる
    pub fn new(rules: Vec<CompiledRule>, case_insensitive: bool) -> Self {
        Self {
            rules,
            case_insensitive,
        }
    }

    /// レコード単位の置換処理
    /// target で対象列を絞る (All なら全列、Indices なら指定列のみ)
    pub fn apply_record(
        &self,
        record: &mut StringRecord,
        row: u64,
        headers: Option<&StringRecord>,
        target: &TargetColumns,
    ) -> Result<bool, TransformError> {
        // ヘッダー無し + 列番号指定の場合、解決時に範囲チェックできないため
        // データ行のカラム数に対してここで検証する
        if let TargetColumns::Indices(idx) = target {
            for &i in idx {
                if i >= record.len() {
                    return Err(TransformError::IndexOutOfRange {
                        index: i,
                        columns: record.len(),
                    });
                }
            }
        }

        // cell ごとに置換処理
        // csv::StringRecord は cell 単位の差し替えができないので cell ごとに処理して最後に record を差し替える形をとる
        let mut any_modified = false;
        let mut new_fields: Vec<String> = Vec::with_capacity(record.len());
        for (col_index, field) in record.iter().enumerate() {
            // 対象列でなければ置換せず元の値をそのまま残す
            if !target.includes(col_index) {
                new_fields.push(field.to_string());
                continue;
            }

            // カラム名取得
            // ヘッダーがあればヘッダー文字列、なければ列番号を使う
            let column_name = headers
                .and_then(|h| h.get(col_index))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("col[{}]", col_index));

            // 置換処理
            let (replaced, modified) = self.apply_cell(field, row, &column_name)?;
            if modified {
                any_modified = true;
            }
            new_fields.push(replaced);
        }

        // 置換処理後の cell で record を生成して差し替え
        *record = StringRecord::from(new_fields);
        Ok(any_modified)
    }

    /// セル単位の置換処理
    fn apply_cell(
        &self,
        cell: &str,
        row: u64,
        column: &str,
    ) -> Result<(String, bool), TransformError> {
        // cell に対してルール評価をしてマッチ位置を収集
        // 元の cell に対して全ルール評価するので、評価の連鎖はしない
        let mut matches: Vec<(usize, usize, &CompiledRule)> = Vec::new();
        for rule in &self.rules {
            match rule {
                // 単純置換: from の出現位置をすべて列挙
                CompiledRule::Simple { from, .. } => {
                    self.find_simple_matches(cell, from, rule, &mut matches);
                }
                // 正規表現: pattern.find_iter で非オーバーラップマッチを列挙
                CompiledRule::Regex { pattern, .. } => {
                    self.find_regex_matches(cell, pattern, rule, &mut matches);
                }
            }
        }

        // マッチ位置がなければ変更しないので処理終了
        if matches.is_empty() {
            return Ok((cell.to_string(), false));
        }

        // マッチ位置をソート
        // 後続の複数ルールでのマッチ位置の衝突検知と、後ろからの置換処理のため、開始位置でソートしておく必要がある
        matches.sort_by_key(|(s, _, _)| *s);

        // 衝突検知
        // 前の end > 次の start なら範囲が重なっている = 衝突
        for i in 1..matches.len() {
            if matches[i - 1].1 > matches[i].0 {
                // 衝突した 2 つのルール ID を文字列化して Error に格納
                // RuleId は Display 実装で `rule[N] "name"` のように整形される
                let conflicting_rules: Vec<String> = vec![
                    matches[i - 1].2.id().to_string(),
                    matches[i].2.id().to_string(),
                ];
                return Err(TransformError::RuntimeCollision {
                    row,
                    column: column.to_string(),
                    rules: conflicting_rules,
                });
            }
        }

        // 置換処理
        // 後ろから置換処理を行うことで、前方に置換後の文字列長変更の影響を与えないようにする
        let mut result = cell.to_string();
        for (start, end, rule) in matches.iter().rev() {
            let replacement: String = match rule {
                CompiledRule::Simple { to, .. } => to.clone(),
                // TODO: Regex の replacement は現状固定文字列だが $1 等に展開する
                CompiledRule::Regex { replacement, .. } => replacement.clone(),
            };
            result.replace_range(*start..*end, &replacement);
        }

        Ok((result, true))
    }

    /// セル内の単純置換マッチ位置をすべてマップに追加する
    fn find_simple_matches<'a>(
        &self,
        cell: &str,
        from: &str,
        rule: &'a CompiledRule,
        matches: &mut Vec<(usize, usize, &'a CompiledRule)>,
    ) {
        if self.case_insensitive {
            // 両方小文字化して位置を引く
            // TODO: ASCII だと大文字小文字で位置ズレが起きないが Unicode 特殊文字はズレて
            // panic などを引き起こす可能性があるが、現段階では対応していないので、要対応
            let lower_cell = cell.to_lowercase();
            let lower_from = from.to_lowercase();
            let mut start = 0;

            // find で最初のマッチ位置を取得してマップ格納し、文字長分ずらしてまた find のループ
            // セル内の複数位置を検知するための処理
            while let Some(pos) = lower_cell[start..].find(&lower_from) {
                let absolute = start + pos;
                matches.push((absolute, absolute + lower_from.len(), rule));
                start = absolute + lower_from.len();
            }
        } else {
            // 小文字化が入っていないだけで上記処理は同一
            let mut start = 0;
            while let Some(pos) = cell[start..].find(from) {
                let absolute = start + pos;
                matches.push((absolute, absolute + from.len(), rule));
                start = absolute + from.len();
            }
        }
    }

    /// セル内の正規表現マッチ位置をすべてマップに追加する
    fn find_regex_matches<'a>(
        &self,
        cell: &str,
        pattern: &regex::Regex,
        rule: &'a CompiledRule,
        matches: &mut Vec<(usize, usize, &'a CompiledRule)>,
    ) {
        // pattern.find_iter は非オーバーラップマッチを iterator で返す
        // m.start() / m.end() は UTF-8 境界を尊重したバイト位置
        for m in pattern.find_iter(cell) {
            matches.push((m.start(), m.end(), rule));
        }
    }
}
