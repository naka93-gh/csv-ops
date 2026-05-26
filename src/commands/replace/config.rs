use serde::Deserialize;

use crate::error::{ConfigError, CsvOpsError, TransformError, validate_version};
use crate::rule_id::RuleId;

use super::rule::CompiledRule;

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
        validate_version(config.version, SUPPORTED_VERSION)?;
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

        // 不正な正規表現は compile 失敗するので TransformError::InvalidRegex に変換
        let pattern =
            build_matcher(pattern, case_insensitive).map_err(|e| TransformError::InvalidRegex {
                rule: id.to_string(),
                source: e,
            })?;

        Ok(CompiledRule::Regex {
            id,
            pattern,
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

        // from をエスケープして正規表現マッチャにする
        // これで単純置換も正規表現と同じ非オーバーラップマッチで扱え、
        // case_insensitive も正規表現側に一元化できる (文字位置のズレが起きない)
        let matcher = build_matcher(&regex::escape(from), case_insensitive).map_err(|e| {
            TransformError::InvalidRegex {
                rule: id.to_string(),
                source: e,
            }
        })?;

        Ok(CompiledRule::Simple {
            id,
            matcher,
            from: from.clone(),
            to: to.clone(),
        })
    }
}

/// 正規表現をコンパイルする (case_insensitive 対応)
fn build_matcher(pattern: &str, case_insensitive: bool) -> Result<regex::Regex, regex::Error> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
}

#[cfg(test)]
mod tests;
