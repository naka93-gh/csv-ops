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
}

/// 設定ファイル関連エラー
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("TOML パースエラー: {0}")]
    Parse(#[from] toml::de::Error),
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
}
