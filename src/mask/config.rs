use serde::Deserialize;

use crate::column::ColumnRef;
use crate::error::ConfigError;

/// サポートする設定バージョン
const SUPPORTED_VERSION: u32 = 1;

/// マスク文字のデフォルト
const DEFAULT_MASK_CHAR: char = '*';

/// mask 設定ファイル全体
#[derive(Debug, Deserialize)]
pub struct MaskConfig {
    // version 未指定を検出するため Option で受ける (None → VersionMissing)
    version: Option<u32>,
    #[serde(default)]
    columns: Vec<ColumnRef>,
    #[serde(default)]
    options: Options,
}

/// [options] セクション
#[derive(Debug, Deserialize, Default)]
struct Options {
    /// マスク文字 (省略時はデフォルト)。複数文字なら先頭 1 文字を採用する
    mask_char: Option<String>,
}

impl MaskConfig {
    /// TOML 文字列をパースして検証済みの MaskConfig を返す
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: MaskConfig = toml::from_str(text)?;
        config.validate_version()?;
        if config.columns.is_empty() {
            return Err(ConfigError::Validation("columns が空です".to_string()));
        }
        Ok(config)
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

    /// マスク対象の列指定
    pub fn columns(&self) -> &[ColumnRef] {
        &self.columns
    }

    /// マスク文字 (未指定ならデフォルト)
    pub fn mask_char(&self) -> char {
        self.options
            .mask_char
            .as_deref()
            .and_then(|s| s.chars().next())
            .unwrap_or(DEFAULT_MASK_CHAR)
    }
}

#[cfg(test)]
mod tests;
