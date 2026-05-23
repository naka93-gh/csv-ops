use std::path::PathBuf;

use csv::StringRecord;
use serde::Deserialize;

use crate::column::{ColumnRef, resolve_indices};
use crate::error::{ConfigError, CsvOpsError};
use crate::text::algorithm::Algorithm;
use crate::text::normalize::NormalizeSet;

use super::dict::Dictionary;
use super::rule::CompiledSimilarityRule;

/// サポートする設定バージョン
const SUPPORTED_VERSION: u32 = 1;

/// しきい値の既定値
pub const DEFAULT_THRESHOLD: f64 = 0.7;

/// 設定ファイル全体
#[derive(Debug, Deserialize)]
pub struct SimilarityConfig {
    // version 未指定を検出するため Option で受ける (None → VersionMissing)
    version: Option<u32>,
    #[serde(default)]
    rules: Vec<RuleSpec>,
}

/// `[[rules]]` の 1 要素
/// similarity は対象列 1 列固定。out_col / score_col は必須
#[derive(Debug, Deserialize)]
struct RuleSpec {
    column: ColumnRef,
    dict: PathBuf,
    out_col: String,
    score_col: String,
    threshold: Option<f64>,
    normalize: Option<Vec<String>>,
    algorithm: Option<String>,
}

impl SimilarityConfig {
    /// TOML 文字列をパースして検証済みの SimilarityConfig を返す
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: SimilarityConfig = toml::from_str(text)?;
        config.validate_version()?;
        Ok(config)
    }

    /// CLI 引数モード用: 1 ルールから SimilarityConfig を組み立てる
    pub fn from_single_rule(
        column: ColumnRef,
        dict: PathBuf,
        out_col: String,
        score_col: String,
        threshold: f64,
        normalize: Vec<String>,
        algorithm: String,
    ) -> Self {
        let spec = RuleSpec {
            column,
            dict,
            out_col,
            score_col,
            threshold: Some(threshold),
            normalize: Some(normalize),
            algorithm: Some(algorithm),
        };
        SimilarityConfig {
            version: Some(SUPPORTED_VERSION),
            rules: vec![spec],
        }
    }

    /// version フィールドの検証
    fn validate_version(&self) -> Result<(), ConfigError> {
        match self.version {
            None => Err(ConfigError::VersionMissing),
            Some(v) if v != SUPPORTED_VERSION => Err(ConfigError::UnsupportedVersion {
                found: v,
                supported: SUPPORTED_VERSION,
            }),
            Some(_) => Ok(()),
        }
    }

    /// 全 rules を CompiledSimilarityRule のリストに compile する
    /// 列解決・正規化セット構築・辞書ロードをここでまとめて行う
    /// delimiter は CSV 形式の辞書を読むときの区切り文字 (本体 CSV と共通)
    pub fn compile_rules(
        &self,
        headers: Option<&StringRecord>,
        delimiter: u8,
    ) -> Result<Vec<CompiledSimilarityRule>, CsvOpsError> {
        self.rules
            .iter()
            .map(|spec| compile_rule(spec, headers, delimiter))
            .collect()
    }
}

/// 1 つの RuleSpec を CompiledSimilarityRule に compile する
fn compile_rule(
    spec: &RuleSpec,
    headers: Option<&StringRecord>,
    delimiter: u8,
) -> Result<CompiledSimilarityRule, CsvOpsError> {
    // 列名 / 列番号をヘッダ照合で 1 列分解決する
    let resolved = resolve_indices(std::slice::from_ref(&spec.column), headers)?;
    let column = resolved[0];

    // 正規化セット (未指定ならデフォルト)
    let normalize = match &spec.normalize {
        Some(names) => NormalizeSet::from_names(names)?,
        None => NormalizeSet::default_set(),
    };

    // 類似度アルゴリズム (未指定なら既定の Levenshtein)
    let algorithm = match &spec.algorithm {
        Some(name) => Algorithm::parse(name)?,
        None => Algorithm::default(),
    };

    // しきい値 (未指定なら既定値)。範囲外はエラー
    let threshold = spec.threshold.unwrap_or(DEFAULT_THRESHOLD);
    if !(0.0..=1.0).contains(&threshold) {
        return Err(ConfigError::Validation(format!(
            "threshold は 0.0-1.0 で指定してください: {}",
            threshold
        ))
        .into());
    }

    // 辞書をロードし、候補を normalize で正規化する
    let dict = Dictionary::load(&spec.dict, delimiter, &normalize)?;

    Ok(CompiledSimilarityRule {
        column,
        dict,
        out_col: spec.out_col.clone(),
        score_col: spec.score_col.clone(),
        threshold,
        normalize,
        algorithm,
    })
}

#[cfg(test)]
mod tests;
