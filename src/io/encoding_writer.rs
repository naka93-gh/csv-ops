// 出力エンコーディングへ逐次変換する Write ラッパ

use std::io::{self, Write};

use encoding_rs::Encoding;

/// UTF-8 のバイト列を受け取り、指定エンコーディングへ変換しながら inner へ書き出す
/// csv::Writer の出力 (常に valid UTF-8) をストリーミングで変換するために使う
/// write 呼び出しが UTF-8 文字の途中で分断されても、末尾の不完全バイトを次回へ持ち越すため、全出力をメモリに溜めずに変換できる。
pub struct EncodingWriter<W: Write> {
    inner: W,
    encoding: &'static Encoding,
    /// 前回 write で UTF-8 文字の途中だった末尾バイト
    leftover: Vec<u8>,
    /// 変換不能な文字に遭遇したか (置換文字で出力されている)
    had_errors: bool,
}

impl<W: Write> EncodingWriter<W> {
    pub fn new(inner: W, encoding: &'static Encoding) -> Self {
        Self {
            inner,
            encoding,
            leftover: Vec::new(),
            had_errors: false,
        }
    }

    /// 出力エンコーディングで表現できない文字があったか
    pub fn had_errors(&self) -> bool {
        self.had_errors
    }
}

impl<W: Write> Write for EncodingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 前回持ち越した不完全バイトと結合する
        let mut data = std::mem::take(&mut self.leftover);
        data.extend_from_slice(buf);

        // valid な UTF-8 プレフィックスだけ変換し、末尾の不完全バイトは持ち越す。
        // 入力は csv::Writer 由来で全体としては valid UTF-8 なので、
        // 分断の原因は常に「文字の途中で切れた末尾」に限られる。
        let valid_up_to = match std::str::from_utf8(&data) {
            Ok(_) => data.len(),
            Err(e) => e.valid_up_to(),
        };
        let (valid, rest) = data.split_at(valid_up_to);

        // SAFETY: valid_up_to までは UTF-8 として妥当
        let text = unsafe { std::str::from_utf8_unchecked(valid) };
        let (encoded, _, had_errors) = self.encoding.encode(text);
        if had_errors {
            self.had_errors = true;
        }
        self.inner.write_all(&encoded)?;

        self.leftover = rest.to_vec();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 指定エンコーディングで text を 1 度に書き込んだ結果のバイト列を返す
    fn write_all_at_once(text: &str, encoding: &'static Encoding) -> (Vec<u8>, bool) {
        let mut writer = EncodingWriter::new(Vec::new(), encoding);
        writer.write_all(text.as_bytes()).unwrap();
        writer.flush().unwrap();
        (writer.inner.clone(), writer.had_errors())
    }

    #[test]
    fn utf8_passthrough() {
        let (bytes, had_errors) = write_all_at_once("名前,年齢", encoding_rs::UTF_8);
        assert_eq!(bytes, "名前,年齢".as_bytes());
        assert!(!had_errors);
    }

    #[test]
    fn converts_to_shift_jis() {
        let (bytes, had_errors) = write_all_at_once("名前", encoding_rs::SHIFT_JIS);
        let (expected, _, _) = encoding_rs::SHIFT_JIS.encode("名前");
        assert_eq!(bytes, expected.as_ref());
        assert!(!had_errors);
    }

    #[test]
    fn handles_writes_split_mid_character() {
        // マルチバイト文字の途中で write が分断されても正しく変換できる
        let mut writer = EncodingWriter::new(Vec::new(), encoding_rs::SHIFT_JIS);
        let source = "名前,年齢".as_bytes();
        // 1 バイトずつ書き込む
        for chunk in source.chunks(1) {
            writer.write_all(chunk).unwrap();
        }
        writer.flush().unwrap();
        let (expected, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢");
        assert_eq!(writer.inner, expected.as_ref());
        assert!(!writer.had_errors());
    }

    #[test]
    fn reports_unmappable_character() {
        // 絵文字は Shift_JIS で表現できない
        let (_, had_errors) = write_all_at_once("a😀b", encoding_rs::SHIFT_JIS);
        assert!(had_errors);
    }
}
