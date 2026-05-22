use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding_rs::Encoding;

use crate::error::{CsvOpsError, EncodingError};

/// auto 判定でファイル先頭を読むバイト数
const DETECT_PREFIX_LEN: usize = 65536;

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

/// 入力エンコーディングを解決する
/// "auto" ならファイル先頭を読んで推定し、それ以外は resolve_encoding と同じ。
/// auto 判定は先頭 64KB のみを見るため、先頭が ASCII のみで後半に
/// 非 UTF-8 バイトが現れるファイルでは取り違える場合がある (best-effort)。
pub fn resolve_input_encoding(name: &str, path: &Path) -> Result<&'static Encoding, CsvOpsError> {
    if name == "auto" {
        detect_file_encoding(path)
    } else {
        Ok(resolve_encoding(name)?)
    }
}

/// 入力ファイル先頭を読んでエンコーディングを推定する
fn detect_file_encoding(path: &Path) -> Result<&'static Encoding, CsvOpsError> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; DETECT_PREFIX_LEN];
    let n = file.read(&mut buf)?;
    Ok(detect_encoding(&buf[..n]).0)
}

/// バイト列からエンコーディングを推定する
/// UTF-8 BOM があれば UTF-8、なければ UTF-8 として厳密デコードを試し、成功すれば UTF-8、失敗すれば Shift_JIS とみなす
/// 戻り値は (エンコーディング, BOM 有無)。EUC-JP は自動判定の対象外
pub fn detect_encoding(bytes: &[u8]) -> (&'static Encoding, bool) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (encoding_rs::UTF_8, true);
    }
    if looks_like_utf8(bytes) {
        (encoding_rs::UTF_8, false)
    } else {
        (encoding_rs::SHIFT_JIS, false)
    }
}

/// バイト列が UTF-8 とみなせるか
/// 末尾でマルチバイト列が途切れただけ (error_len() が None) のケースは、
/// ファイル先頭のみを走査したときに起きるため UTF-8 として許容する
fn looks_like_utf8(bytes: &[u8]) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(_) => true,
        Err(e) => e.error_len().is_none(),
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

    #[test]
    fn truncated_utf8_tail_is_utf8() {
        // マルチバイト文字を途中で切ったバイト列 (先頭走査で起きる) は UTF-8 とみなす
        let full = "名前".as_bytes();
        let (enc, _) = detect_encoding(&full[..full.len() - 1]);
        assert_eq!(enc, encoding_rs::UTF_8);
    }

    #[test]
    fn resolve_input_encoding_passes_through_explicit_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dummy.csv");
        std::fs::write(&path, "a,b\n").unwrap();
        let enc = resolve_input_encoding("shift_jis", &path).unwrap();
        assert_eq!(enc, encoding_rs::SHIFT_JIS);
    }

    #[test]
    fn resolve_input_encoding_auto_detects_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("utf8.csv");
        std::fs::write(&path, "名前,年齢\n田中,30\n").unwrap();
        let enc = resolve_input_encoding("auto", &path).unwrap();
        assert_eq!(enc, encoding_rs::UTF_8);
    }

    #[test]
    fn resolve_input_encoding_auto_detects_shift_jis() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sjis.csv");
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢\n田中,30\n");
        std::fs::write(&path, &bytes).unwrap();
        let enc = resolve_input_encoding("auto", &path).unwrap();
        assert_eq!(enc, encoding_rs::SHIFT_JIS);
    }

    #[test]
    fn resolve_input_encoding_rejects_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dummy.csv");
        std::fs::write(&path, "a,b\n").unwrap();
        assert!(resolve_input_encoding("latin-1", &path).is_err());
    }
}
