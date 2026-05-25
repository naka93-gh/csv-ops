use std::path::PathBuf;

use thiserror::Error;

/// csv-ops の最上位エラー型
/// `?` 演算子でサブカテゴリエラーから自動変換される (`#[from]`)
#[derive(Debug, Error)]
pub enum CsvOpsError {
    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV エラー{}: {source}", line.map(|l| format!(" ({} 行目)", l)).unwrap_or_default())]
    Csv {
        line: Option<u64>,
        #[source]
        source: csv::Error,
    },

    #[error(transparent)]
    Encoding(#[from] EncodingError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Transform(#[from] TransformError),

    #[error(transparent)]
    Dict(#[from] DictError),
}

impl From<csv::Error> for CsvOpsError {
    fn from(e: csv::Error) -> Self {
        let line = e.position().map(|p| p.line());
        CsvOpsError::Csv { line, source: e }
    }
}

/// エンコーディング関連エラー
#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("未対応のエンコーディング: {0} (対応: utf-8 / shift_jis / euc-jp)")]
    Unsupported(String),

    #[error("{encoding} でエンコードできない文字が含まれています")]
    EncodeFailure { encoding: String },

    #[error("{encoding} としてデコードできないバイト列が含まれています")]
    DecodeFailure { encoding: String },
}

/// 設定ファイル関連エラー
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("TOML パースエラー: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("設定ファイルに version フィールドがありません")]
    VersionMissing,

    #[error("未対応の設定バージョン: {found} (対応バージョン: {supported})")]
    UnsupportedVersion { found: u32, supported: u32 },

    #[error("設定が不正です: {0}")]
    Validation(String),

    #[error("ルール衝突 ({reason}): {}", rules.join(", "))]
    RuleCollision { rules: Vec<String>, reason: String },
}

/// 設定ファイル / 辞書ファイルの version フィールドを検証する
/// 未指定なら VersionMissing、サポート外なら UnsupportedVersion を返す
pub(crate) fn validate_version(version: Option<u32>, supported: u32) -> Result<(), ConfigError> {
    match version {
        None => Err(ConfigError::VersionMissing),
        Some(v) if v != supported => Err(ConfigError::UnsupportedVersion {
            found: v,
            supported,
        }),
        Some(_) => Ok(()),
    }
}

/// 変換実行時エラー
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("ヘッダに存在しないカラム: {name} (利用可能: {})", available.join(", "))]
    UnknownColumn {
        name: String,
        available: Vec<String>,
    },

    #[error("列番号 {index} は範囲外 (カラム数: {columns})")]
    IndexOutOfRange { index: usize, columns: usize },

    #[error("ヘッダ無し設定では名前指定 ({0}) は使えません。列番号で指定してください")]
    NameWithoutHeaders(String),

    #[error("ルール {rule} の正規表現が不正です: {source}")]
    InvalidRegex {
        rule: String,
        #[source]
        source: regex::Error,
    },

    #[error("実行時のルール衝突 (行 {row}, カラム {column}): {}", rules.join(", "))]
    RuntimeCollision {
        row: u64,
        column: String,
        rules: Vec<String>,
    },

    #[error("出力カラム名 {name} は既存カラムまたは他ルールと衝突しています")]
    OutputColumnConflict { name: String },
}

/// 辞書関連エラー
#[derive(Debug, Error)]
pub enum DictError {
    #[error("辞書にエントリがありません: {0}")]
    Empty(PathBuf),

    #[error("辞書 canonical の重複: {0}")]
    DuplicateCanonical(String),

    #[error("辞書 alias \"{alias}\" が複数の canonical に割り当てられています: {}", canonicals.join(", "))]
    DuplicateAlias {
        alias: String,
        canonicals: Vec<String>,
    },
}
