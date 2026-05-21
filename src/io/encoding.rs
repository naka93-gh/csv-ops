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

/// バイト列からエンコーディングを推定する
/// UTF-8 BOM があれば UTF-8、なければ UTF-8 として厳密デコードを試し、成功すれば UTF-8、失敗すれば Shift_JIS とみなす
/// 戻り値は (エンコーディング, BOM 有無)。EUC-JP は自動判定の対象外
#[allow(dead_code)]
pub fn detect_encoding(bytes: &[u8]) -> (&'static Encoding, bool) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (encoding_rs::UTF_8, true);
    }
    match std::str::from_utf8(bytes) {
        Ok(_) => (encoding_rs::UTF_8, false),
        Err(_) => (encoding_rs::SHIFT_JIS, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_encodings() {
        assert!(resolve_encoding("utf-8").is_ok());
        assert!(resolve_encoding("shift_jis").is_ok());
        assert!(resolve_encoding("euc-jp").is_ok());
    }

    #[test]
    fn rejects_unknown_encoding() {
        assert!(resolve_encoding("latin-1").is_err());
    }

    #[test]
    fn detects_utf8_without_bom() {
        let (enc, bom) = detect_encoding("名前,年齢".as_bytes());
        assert_eq!(enc, encoding_rs::UTF_8);
        assert!(!bom);
    }

    #[test]
    fn detects_utf8_with_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("名前".as_bytes());
        let (enc, bom) = detect_encoding(&bytes);
        assert_eq!(enc, encoding_rs::UTF_8);
        assert!(bom);
    }

    #[test]
    fn falls_back_to_shift_jis() {
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢");
        let (enc, bom) = detect_encoding(&bytes);
        assert_eq!(enc, encoding_rs::SHIFT_JIS);
        assert!(!bom);
    }

    #[test]
    fn empty_is_utf8() {
        let (enc, bom) = detect_encoding(b"");
        assert_eq!(enc, encoding_rs::UTF_8);
        assert!(!bom);
    }
}
