use csv::StringRecord;

use crate::error::TransformError;

use super::rule::CompiledExtractRule;

/// extract 抽出処理本体
pub(crate) struct ExtractTransform {
    // ルール
    rules: Vec<CompiledExtractRule>,
}

impl ExtractTransform {
    /// コンストラクタ
    pub fn new(rules: Vec<CompiledExtractRule>) -> Self {
        Self { rules }
    }

    /// レコード単位の抽出処理
    /// 各ルールについて対象列からマッチを抽出し、結果を末尾に 1 列追加する。
    /// 戻り値はルール毎の「1 件以上抽出できたか」の bool 列 (統計用)。
    pub fn apply_record(&self, record: &mut StringRecord) -> Result<Vec<bool>, TransformError> {
        // 範囲チェックの基準に使うので、列追加前のフィールド数を取得
        let original_len = record.len();

        // 既存フィールドをコピーし、以降ルール毎の抽出結果を push して伸ばしていく
        let mut fields: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        let mut extracted_flags: Vec<bool> = Vec::with_capacity(self.rules.len());

        for rule in &self.rules {
            // ヘッダ無し + 列番号指定では compile 時に範囲チェックできないため、行ごとに検証する
            if rule.column >= original_len {
                return Err(TransformError::IndexOutOfRange {
                    index: rule.column,
                    columns: original_len,
                });
            }

            // 対象列の全マッチを取り出す
            // キャプチャグループがあれば 1 番目のグループ、なければマッチ全体を採用する
            let matches: Vec<String> = rule
                .pattern
                .captures_iter(&fields[rule.column])
                .map(|caps| {
                    caps.get(1)
                        .or_else(|| caps.get(0))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                })
                .collect();

            // マッチなしは空文字、複数マッチは separator 連結
            extracted_flags.push(!matches.is_empty());
            fields.push(matches.join(&rule.separator));
        }

        // 追加列を含めたフィールドで record を差し替える
        *record = StringRecord::from(fields);
        Ok(extracted_flags)
    }
}

#[cfg(test)]
mod tests;
