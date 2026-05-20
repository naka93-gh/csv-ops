use serde::Deserialize;

use crate::error::{ConfigError, CsvOpsError, TransformError};

use super::rule::{CompiledRule, RuleId};

/// サポートする設定バージョン
const SUPPORTED_VERSION: u32 = 1;

/// 設定ファイル全体
#[derive(Debug, Deserialize)]
pub struct ReplaceConfig {
    // version 未指定を検出するため Option で受ける (None → VersionMissing)
    version: Option<u32>,
    #[serde(default)]
    options: Options,
    #[serde(default)]
    rules: Vec<RuleSpec>,
}

/// [options] セクション
#[derive(Debug, Deserialize, Default)]
pub struct Options {
    #[serde(default)]
    case_insensitive: bool,
}

/// `[[rules]]` の 1 要素
/// 単純置換 (from + to) と正規表現 (pattern + replacement + regex=true) を
/// 同一 struct で受け、regex フラグとフィールドの組み合わせで判別する
#[derive(Debug, Deserialize)]
pub struct RuleSpec {
    name: Option<String>,
    from: Option<String>,
    to: Option<String>,
    pattern: Option<String>,
    replacement: Option<String>,
    #[serde(default)]
    regex: bool,
}

impl ReplaceConfig {
    /// TOML 文字列をパースして検証済みの ReplaceConfig を返す
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: ReplaceConfig = toml::from_str(text)?;
        config.validate_version()?;
        Ok(config)
    }

    /// CLI 引数モード用: 1 ルールから ReplaceConfig を組み立てる
    /// version は 1 固定。regex = true 時は from/to を pattern/replacement として扱う
    pub fn from_single_rule(from: String, to: String, regex: bool, case_insensitive: bool) -> Self {
        let spec = if regex {
            RuleSpec {
                name: None,
                from: None,
                to: None,
                pattern: Some(from),
                replacement: Some(to),
                regex: true,
            }
        } else {
            RuleSpec {
                name: None,
                from: Some(from),
                to: Some(to),
                pattern: None,
                replacement: None,
                regex: false,
            }
        };
        ReplaceConfig {
            version: Some(SUPPORTED_VERSION),
            options: Options { case_insensitive },
            rules: vec![spec],
        }
    }

    /// version フィールドの検証
    /// 未指定はエラー、サポート外バージョンもエラー (SPECS.md: 厳格運用)
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

    pub fn case_insensitive(&self) -> bool {
        self.options.case_insensitive
    }

    /// 全 rules を CompiledRule のリストに compile する
    /// 1 つでも compile に失敗したら、その時点のエラーを返す
    pub fn compile_rules(&self) -> Result<Vec<CompiledRule>, CsvOpsError> {
        self.rules
            .iter()
            .enumerate()
            .map(|(i, spec)| compile_rule(spec, i, self.options.case_insensitive))
            .collect()
    }
}

/// 1 つの RuleSpec を CompiledRule に compile する
fn compile_rule(
    spec: &RuleSpec,
    index: usize,
    case_insensitive: bool,
) -> Result<CompiledRule, CsvOpsError> {
    // RuleId は index と任意の name から構成。エラーメッセージでルールを特定するのに使う
    let id = RuleId {
        index,
        name: spec.name.clone(),
    };

    if spec.regex {
        // 正規表現ルール時

        // from / to が紛れていたら設定ミスとしてエラー
        if spec.from.is_some() || spec.to.is_some() {
            return Err(ConfigError::Validation(format!(
                "{} は regex = true なので from / to は指定できません",
                id
            ))
            .into());
        }

        // pattern / replacement は必須なので欠けていればエラー
        let pattern = spec
            .pattern
            .as_ref()
            .ok_or_else(|| ConfigError::Validation(format!("{} に pattern がありません", id)))?;
        let replacement = spec.replacement.as_ref().ok_or_else(|| {
            ConfigError::Validation(format!("{} に replacement がありません", id))
        })?;

        // case_insensitive 有無で compile 経路を分ける
        // 不正な正規表現は compile 失敗するので TransformError::InvalidRegex に変換
        let compiled = if case_insensitive {
            regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
        } else {
            regex::Regex::new(pattern)
        };
        let compiled = compiled.map_err(|e| TransformError::InvalidRegex {
            rule: id.to_string(),
            source: e,
        })?;

        Ok(CompiledRule::Regex {
            id,
            pattern: compiled,
            replacement: replacement.clone(),
        })
    } else {
        // 単純置換ルール時

        // pattern / replacement が紛れていたら設定ミスとしてエラー
        if spec.pattern.is_some() || spec.replacement.is_some() {
            return Err(ConfigError::Validation(format!(
                "{} は単純置換なので pattern / replacement は指定できません (regex = true が必要)",
                id
            ))
            .into());
        }

        // from / to は必須なので欠けていればエラー
        let from = spec
            .from
            .as_ref()
            .ok_or_else(|| ConfigError::Validation(format!("{} に from がありません", id)))?;
        let to = spec
            .to
            .as_ref()
            .ok_or_else(|| ConfigError::Validation(format!("{} に to がありません", id)))?;

        Ok(CompiledRule::Simple {
            id,
            from: from.clone(),
            to: to.clone(),
        })
    }
}

#[cfg(test)]
mod tests;
