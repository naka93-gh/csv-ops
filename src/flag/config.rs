use csv::StringRecord;
use serde::Deserialize;

use crate::column::{ColumnRef, resolve_indices};
use crate::error::{ConfigError, CsvOpsError, TransformError, validate_version};

use super::rule::CompiledFlagRule;

/// サポートする設定バージョン
const SUPPORTED_VERSION: u32 = 1;

/// bool 値のデフォルト表現
const DEFAULT_TRUE: &str = "true";
const DEFAULT_FALSE: &str = "false";

/// 設定ファイル全体
#[derive(Debug, Deserialize)]
pub struct FlagConfig {
    // version 未指定を検出するため Option で受ける (None → VersionMissing)
    version: Option<u32>,
    #[serde(default)]
    rules: Vec<RuleSpec>,
}

/// `[[rules]]` の 1 要素
/// 対象列は単一指定 (column) と複数指定 (columns) のどちらか一方で書く。
/// true_value / false_value は省略時にデフォルト値を使う。
#[derive(Debug, Deserialize)]
pub struct RuleSpec {
    column: Option<ColumnRef>,
    columns: Option<Vec<ColumnRef>>,
    pattern: String,
    out_col: String,
    true_value: Option<String>,
    false_value: Option<String>,
}

impl FlagConfig {
    /// TOML 文字列をパースして検証済みの FlagConfig を返す
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: FlagConfig = toml::from_str(text)?;
        validate_version(config.version, SUPPORTED_VERSION)?;
        Ok(config)
    }

    /// CLI 引数モード用: 1 ルールから FlagConfig を組み立てる
    /// version は 1 固定。true/false 値はデフォルト固定 (CLI ではカスタム不可)
    pub fn from_single_rule(pattern: String, columns: Vec<ColumnRef>, out_col: String) -> Self {
        let spec = RuleSpec {
            column: None,
            columns: Some(columns),
            pattern,
            out_col,
            true_value: None,
            false_value: None,
        };
        FlagConfig {
            version: Some(SUPPORTED_VERSION),
            rules: vec![spec],
        }
    }

    /// 全 rules を CompiledFlagRule のリストに compile する
    /// 正規表現の compile と、対象列のヘッダ照合による解決をここでまとめて行う。
    /// 列名解決にヘッダが要るため、replace と違い run 側でヘッダ取得後に呼ぶ。
    pub fn compile_rules(
        &self,
        headers: Option<&StringRecord>,
    ) -> Result<Vec<CompiledFlagRule>, CsvOpsError> {
        self.rules
            .iter()
            .enumerate()
            .map(|(i, spec)| compile_rule(spec, i, headers))
            .collect()
    }
}

/// 1 つの RuleSpec を CompiledFlagRule に compile する
fn compile_rule(
    spec: &RuleSpec,
    index: usize,
    headers: Option<&StringRecord>,
) -> Result<CompiledFlagRule, CsvOpsError> {
    // 対象列は column / columns のどちらか一方のみなので、両方 or なしは設定ミスとしてエラー
    let refs: Vec<ColumnRef> = match (&spec.column, &spec.columns) {
        (Some(_), Some(_)) => {
            return Err(ConfigError::Validation(format!(
                "rule[{}]: column と columns は同時に指定できません",
                index
            ))
            .into());
        }
        (None, None) => {
            return Err(ConfigError::Validation(format!(
                "rule[{}]: column か columns のいずれかが必要です",
                index
            ))
            .into());
        }
        (Some(c), None) => vec![c.clone()],
        (None, Some(cs)) => {
            if cs.is_empty() {
                return Err(
                    ConfigError::Validation(format!("rule[{}]: columns が空です", index)).into(),
                );
            }
            cs.clone()
        }
    };

    // 列名 / 列番号をヘッダ照合で解決する (ヘッダ無し + 列番号は run 側で行ごとに範囲チェック)
    let columns = resolve_indices(&refs, headers)?;

    // 不正な正規表現は compile 失敗するので TransformError::InvalidRegex に変換
    let pattern = regex::Regex::new(&spec.pattern).map_err(|e| TransformError::InvalidRegex {
        rule: format!("rule[{}]", index),
        source: e,
    })?;

    Ok(CompiledFlagRule {
        pattern,
        columns,
        out_col: spec.out_col.clone(),
        true_value: spec
            .true_value
            .clone()
            .unwrap_or_else(|| DEFAULT_TRUE.to_string()),
        false_value: spec
            .false_value
            .clone()
            .unwrap_or_else(|| DEFAULT_FALSE.to_string()),
    })
}

#[cfg(test)]
mod tests;
