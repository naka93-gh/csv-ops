use csv::StringRecord;
use serde::Deserialize;

use crate::column::{ColumnRef, resolve_indices};
use crate::error::{ConfigError, CsvOpsError, TransformError, validate_version};

use super::rule::CompiledExtractRule;

/// サポートする設定バージョン
const SUPPORTED_VERSION: u32 = 1;

/// 複数マッチ連結のデフォルト区切り文字
const DEFAULT_SEPARATOR: &str = ";";

/// 設定ファイル全体
#[derive(Debug, Deserialize)]
pub struct ExtractConfig {
    // version 未指定を検出するため Option で受ける (None → VersionMissing)
    version: Option<u32>,
    #[serde(default)]
    rules: Vec<RuleSpec>,
}

/// `[[rules]]` の 1 要素
/// extract は対象列 1 列固定なので column は単一指定のみ
/// separator は省略時にデフォルト値を使う
#[derive(Debug, Deserialize)]
pub struct RuleSpec {
    column: ColumnRef,
    pattern: String,
    out_col: String,
    separator: Option<String>,
}

impl ExtractConfig {
    /// TOML 文字列をパースして検証済みの ExtractConfig を返す
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: ExtractConfig = toml::from_str(text)?;
        validate_version(config.version, SUPPORTED_VERSION)?;
        Ok(config)
    }

    /// CLI 引数モード用: 1 ルールから ExtractConfig を組み立てる
    /// version は 1 固定。separator は None ならデフォルトを使う
    pub fn from_single_rule(
        pattern: String,
        column: ColumnRef,
        out_col: String,
        separator: Option<String>,
    ) -> Self {
        let spec = RuleSpec {
            column,
            pattern,
            out_col,
            separator,
        };
        ExtractConfig {
            version: Some(SUPPORTED_VERSION),
            rules: vec![spec],
        }
    }

    /// 全 rules を CompiledExtractRule のリストに compile する
    /// 正規表現の compile と、対象列のヘッダ照合による解決をここでまとめて行う
    pub fn compile_rules(
        &self,
        headers: Option<&StringRecord>,
    ) -> Result<Vec<CompiledExtractRule>, CsvOpsError> {
        self.rules
            .iter()
            .enumerate()
            .map(|(i, spec)| compile_rule(spec, i, headers))
            .collect()
    }
}

/// 1 つの RuleSpec を CompiledExtractRule に compile する
fn compile_rule(
    spec: &RuleSpec,
    index: usize,
    headers: Option<&StringRecord>,
) -> Result<CompiledExtractRule, CsvOpsError> {
    // 列名 / 列番号をヘッダ照合で 1 列分だけ解決する
    // (ヘッダ無し + 列番号は run 側で行ごとに範囲チェック)
    let resolved = resolve_indices(std::slice::from_ref(&spec.column), headers)?;
    let column = resolved[0];

    // 不正な正規表現は compile 失敗するので TransformError::InvalidRegex に変換
    let pattern = regex::Regex::new(&spec.pattern).map_err(|e| TransformError::InvalidRegex {
        rule: format!("rule[{}]", index),
        source: e,
    })?;

    Ok(CompiledExtractRule {
        pattern,
        column,
        out_col: spec.out_col.clone(),
        separator: spec
            .separator
            .clone()
            .unwrap_or_else(|| DEFAULT_SEPARATOR.to_string()),
    })
}

#[cfg(test)]
mod tests;
