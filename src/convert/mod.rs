use std::path::PathBuf;

use crate::error::{CsvOpsError, EncodingError};
use crate::io::resolve_encoding;

/// convert::run に渡す設定一式
pub struct ConvertRequest {
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp)
    pub input_encoding: String,
    /// 出力エンコーディング名
    pub output_encoding: String,
    /// 入力区切り文字
    pub input_delimiter: u8,
    /// 出力区切り文字
    pub output_delimiter: u8,
}

/// convert 実行の統計
#[derive(Debug)]
pub struct ConvertStats {
    /// 変換した行数
    pub rows: u64,
}

/// convert サブコマンドのエントリポイント
/// エンコーディングと区切り文字だけを変換して素通しする
pub fn run(request: ConvertRequest) -> Result<ConvertStats, CsvOpsError> {
    let in_enc = resolve_encoding(&request.input_encoding)?;
    let out_enc = resolve_encoding(&request.output_encoding)?;

    // 入力を全読込。行終端の検出（生バイト走査）とデコードの両方に使う
    let raw = std::fs::read(&request.input)?;

    // 行終端を検出する
    // \r \n は ASCII 制御文字で、SJIS / EUC-JP / UTF-8 のどれでも多バイト列の一部にならないため、生バイトのまま走査して安全
    let use_crlf = detect_crlf(&raw);

    // 入力エンコーディングでデコード
    // 不正バイトがあった場合はエラー
    let (decoded, _, had_errors) = in_enc.decode(&raw);
    if had_errors {
        return Err(EncodingError::DecodeFailure {
            encoding: in_enc.name().to_string(),
        }
        .into());
    }

    // csv で読み、出力区切り文字で書き直す
    // 内容を変化させないので全て素通し
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(request.input_delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(decoded.as_bytes());

    // 変換結果は一旦 UTF-8 で書き出し、後で出力エンコーディングへ変換する
    let mut utf8_buf = Vec::new();
    let mut rows: u64 = 0;
    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(request.output_delimiter)
            .flexible(true)
            .terminator(if use_crlf {
                csv::Terminator::CRLF
            } else {
                csv::Terminator::Any(b'\n')
            })
            .from_writer(&mut utf8_buf);

        for result in rdr.records() {
            let record = result?;
            wtr.write_record(&record)?;
            rows += 1;
        }
        wtr.flush()?;
    }

    // UTF-8 結果を出力エンコーディングへ変換して書き出す
    // utf8_buf は csv writer が書いたものなので必ず valid UTF-8
    let utf8_str = std::str::from_utf8(&utf8_buf).expect("csv writer の出力は常に valid UTF-8");
    let (encoded, _, had_errors) = out_enc.encode(utf8_str);
    if had_errors {
        return Err(EncodingError::EncodeFailure {
            encoding: out_enc.name().to_string(),
        }
        .into());
    }
    std::fs::write(&request.output, &encoded)?;

    Ok(ConvertStats { rows })
}

/// 生バイトから最初の改行を探し、CRLF かどうかを返す
/// 改行が無い、または LF のみなら false
fn detect_crlf(bytes: &[u8]) -> bool {
    match bytes.iter().position(|&b| b == b'\n') {
        Some(i) => i > 0 && bytes[i - 1] == b'\r',
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::detect_crlf;

    #[test]
    fn crlf_detected() {
        assert!(detect_crlf(b"a,b\r\n1,2\r\n"));
    }

    #[test]
    fn lf_only_is_not_crlf() {
        assert!(!detect_crlf(b"a,b\n1,2\n"));
    }

    #[test]
    fn no_newline_is_not_crlf() {
        assert!(!detect_crlf(b"a,b"));
    }

    #[test]
    fn leading_newline_is_not_crlf() {
        // 先頭が改行なら直前バイトがないので CRLF 扱いしない
        assert!(!detect_crlf(b"\n1,2\n"));
    }
}
