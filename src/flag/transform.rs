use csv::StringRecord;

use crate::error::TransformError;

use super::rule::CompiledFlagRule;

/// flag 判定処理本体
pub(crate) struct FlagTransform {
    // ルール
    rules: Vec<CompiledFlagRule>,
}

impl FlagTransform {
    /// コンストラクタ
    pub fn new(rules: Vec<CompiledFlagRule>) -> Self {
        Self { rules }
    }

    /// レコード単位の flag 判定
    pub fn apply_record(&self, record: &mut StringRecord) -> Result<Vec<bool>, TransformError> {
        // 数範囲チェックの基準に使うので、列追加前のフィールド数を取得
        let original_len = record.len();

        // 既存フィールドをコピーし、以降ルール毎の判定結果を push して伸ばしていく
        let mut fields: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        let mut flags: Vec<bool> = Vec::with_capacity(self.rules.len());

        for rule in &self.rules {
            // ヘッダ無し + 列番号指定では compile 時に範囲チェックできないため、行ごとに検証する
            for &i in &rule.columns {
                if i >= original_len {
                    return Err(TransformError::IndexOutOfRange {
                        index: i,
                        columns: original_len,
                    });
                }
            }

            // ルール内の対象列のうち 1 つでもマッチすれば true
            let matched = rule
                .columns
                .iter()
                .any(|&i| rule.pattern.is_match(&fields[i]));

            fields.push(if matched {
                rule.true_value.clone()
            } else {
                rule.false_value.clone()
            });
            flags.push(matched);
        }

        // 追加列を含めたフィールドで record を差し替える
        *record = StringRecord::from(fields);
        Ok(flags)
    }
}

#[cfg(test)]
mod tests;
