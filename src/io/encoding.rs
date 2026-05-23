use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding_rs::Encoding;

use crate::error::{CsvOpsError, EncodingError};

/// auto 判定でファイル先頭を読むバイト数
const DETECT_PREFIX_LEN: usize = 65536;

/// 先頭が全 ASCII だった場合、追加で末尾から読むバイト数
/// 「ASCII ヘッダ + 後半 SJIS」の業務 CSV パターンで誤判定するのを防ぐためのフォールバック
const DETECT_TAIL_LEN: usize = 65536;

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
/// "auto" ならファイル先頭を読んで推定し、それ以外は resolve_encoding と同じ
/// auto 判定はサンプルベースなので、サンプルされた範囲外に判定材料がある場合は取り違える可能性がある (best-effort)
pub fn resolve_input_encoding(name: &str, path: &Path) -> Result<&'static Encoding, CsvOpsError> {
    if name == "auto" {
        detect_file_encoding(path)
    } else {
        Ok(resolve_encoding(name)?)
    }
}

/// 入力ファイル先頭を読んでエンコーディングを推定する
/// 先頭サンプルが全 ASCII だった場合のみ、末尾サンプルも追加で読んで判定する
/// (「ASCII ヘッダ + 後半 SJIS」の業務 CSV パターンで誤判定するのを防ぐ)
fn detect_file_encoding(path: &Path) -> Result<&'static Encoding, CsvOpsError> {
    let mut file = File::open(path)?;
    let mut sample = vec![0u8; DETECT_PREFIX_LEN];
    let n = file.read(&mut sample)?;
    sample.truncate(n);

    if is_all_ascii(&sample) {
        let file_len = file.metadata()?.len();
        // 末尾が prefix を超える範囲にあるときだけ追加で読む
        if file_len > n as u64 {
            use std::io::{Seek, SeekFrom};
            // prefix と被らない位置から末尾の DETECT_TAIL_LEN バイトを読む
            let tail_start = file_len
                .saturating_sub(DETECT_TAIL_LEN as u64)
                .max(n as u64);
            file.seek(SeekFrom::Start(tail_start))?;
            let mut tail = vec![0u8; DETECT_TAIL_LEN];
            let m = file.read(&mut tail)?;
            sample.extend_from_slice(&tail[..m]);
        }
    }
    Ok(detect_encoding(&sample).0)
}

/// バイト列が ASCII 範囲 (0x00-0x7F) のみで構成されているか
fn is_all_ascii(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b < 0x80)
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
    fn resolve_input_encoding_auto_handles_ascii_header_then_sjis_body() {
        // 業務 CSV でよくある「ASCII ヘッダ + 後半 SJIS 本文」パターン
        // 先頭サンプル (64KB) が全 ASCII でも、末尾サンプルで SJIS と判定できる
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ascii_then_sjis.csv");
        // ASCII ヘッダ + 大量の ASCII データ行で 64KB を超え、最後に SJIS 本文を入れる
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"id,name,email\n");
        for i in 0..6000 {
            bytes.extend_from_slice(format!("{},user{},user{}@example.com\n", i, i, i).as_bytes());
        }
        let (sjis_tail, _, _) = encoding_rs::SHIFT_JIS.encode("9999,田中太郎,tanaka@example.com\n");
        bytes.extend_from_slice(&sjis_tail);
        assert!(
            bytes.len() > super::DETECT_PREFIX_LEN,
            "テスト前提として 64KB を超える長さで作る必要がある"
        );
        std::fs::write(&path, &bytes).unwrap();

        let enc = resolve_input_encoding("auto", &path).unwrap();
        assert_eq!(enc, encoding_rs::SHIFT_JIS);
    }

    #[test]
    fn resolve_input_encoding_auto_keeps_utf8_for_all_ascii_file() {
        // 全 ASCII のファイルは UTF-8 とみなされる (既存挙動の維持)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("all_ascii.csv");
        std::fs::write(&path, "id,name\n1,alice\n2,bob\n").unwrap();
        let enc = resolve_input_encoding("auto", &path).unwrap();
        assert_eq!(enc, encoding_rs::UTF_8);
    }

    #[test]
    fn resolve_input_encoding_rejects_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dummy.csv");
        std::fs::write(&path, "a,b\n").unwrap();
        assert!(resolve_input_encoding("latin-1", &path).is_err());
    }
}
