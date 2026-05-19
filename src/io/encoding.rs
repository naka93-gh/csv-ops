use encoding_rs::Encoding;

use crate::error::EncodingError;

/// 設定文字列からエンコーディングを解決する
/// サポート: "utf-8" / "shift_jis" / "euc-jp"
pub fn resolve_encoding(name: &str) -> Result<&'static Encoding, EncodingError> {
    match name {
        "utf-8" => Ok(encoding_rs::UTF_8),
        "shift_jis" => Ok(encoding_rs::SHIFT_JIS),
        "euc-jp" => Ok(encoding_rs::EUC_JP),
        _ => Err(EncodingError::Unsupported(name.to_string())),
    }
}
