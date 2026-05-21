// 入力エンコーディングを UTF-8 へ厳密にデコードする Read ラッパ

use std::io::{self, Read};

use encoding_rs::{Decoder, DecoderResult, Encoding};

/// 指定エンコーディングのバイト列を読み取り、UTF-8 へデコードしながら供給する Read ラッパー
/// 不正なバイト列は置換文字で誤魔化さず io エラーで停止するため、入力エンコーディングの取り違えを黙って通さない
/// EncodingWriter (出力側) と対になる
pub struct DecodingReader<R: Read> {
    inner: R,
    decoder: Decoder,
    encoding_name: &'static str,
    /// inner から読み取った生バイト
    in_buf: Box<[u8]>,
    /// in_buf に読み込んだ有効バイト数
    in_filled: usize,
    /// in_buf の消費位置
    in_pos: usize,
    /// デコード済み UTF-8
    out_buf: Vec<u8>,
    /// out_buf の供給位置
    out_pos: usize,
    /// inner が EOF に達したか
    inner_eof: bool,
    /// デコード完了
    done: bool,
}

impl<R: Read> DecodingReader<R> {
    pub fn new(inner: R, encoding: &'static Encoding) -> Self {
        Self {
            inner,
            // BOM があれば消費する標準のデコーダを使う
            decoder: encoding.new_decoder(),
            encoding_name: encoding.name(),
            in_buf: vec![0u8; 8192].into_boxed_slice(),
            in_filled: 0,
            in_pos: 0,
            out_buf: Vec::new(),
            out_pos: 0,
            inner_eof: false,
            done: false,
        }
    }
}

impl<R: Read> Read for DecodingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            // デコード済みバッファに残りがあれば供給する
            if self.out_pos < self.out_buf.len() {
                let n = (self.out_buf.len() - self.out_pos).min(buf.len());
                buf[..n].copy_from_slice(&self.out_buf[self.out_pos..self.out_pos + n]);
                self.out_pos += n;
                return Ok(n);
            }
            if self.done {
                return Ok(0);
            }

            // 入力バイトが尽きていれば inner から補充する
            if self.in_pos >= self.in_filled && !self.inner_eof {
                self.in_filled = self.inner.read(&mut self.in_buf)?;
                self.in_pos = 0;
                if self.in_filled == 0 {
                    self.inner_eof = true;
                }
            }

            // 残りの入力を UTF-8 へデコードする
            let last = self.inner_eof;
            let src = &self.in_buf[self.in_pos..self.in_filled];
            let cap = self
                .decoder
                .max_utf8_buffer_length_without_replacement(src.len())
                .unwrap_or(src.len() * 4 + 16);
            self.out_buf.clear();
            self.out_buf.resize(cap.max(16), 0);
            let (result, read, written) =
                self.decoder
                    .decode_to_utf8_without_replacement(src, &mut self.out_buf, last);
            self.in_pos += read;
            self.out_buf.truncate(written);
            self.out_pos = 0;

            match result {
                // 不正なバイト列。入力エンコーディングの取り違え等
                DecoderResult::Malformed(_, _) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "{} としてデコードできないバイト列が含まれています",
                            self.encoding_name
                        ),
                    ));
                }
                // 入力を消費しきった。last なら完了、そうでなければ次ループで補充
                DecoderResult::InputEmpty => {
                    if last {
                        self.done = true;
                    }
                }
                // 出力バッファ不足。max_utf8_buffer_length で確保済みのため通常起きない
                DecoderResult::OutputFull => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// reader を最後まで読み切って文字列にする
    fn read_all<R: Read>(mut reader: DecodingReader<R>) -> io::Result<String> {
        let mut out = Vec::new();
        reader.read_to_end(&mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn decodes_utf8() {
        let reader = DecodingReader::new("名前,年齢".as_bytes(), encoding_rs::UTF_8);
        assert_eq!(read_all(reader).unwrap(), "名前,年齢");
    }

    #[test]
    fn decodes_shift_jis() {
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢\n田中,30");
        let reader = DecodingReader::new(&bytes[..], encoding_rs::SHIFT_JIS);
        assert_eq!(read_all(reader).unwrap(), "名前,年齢\n田中,30");
    }

    #[test]
    fn rejects_malformed_bytes() {
        // SJIS バイト列を UTF-8 として読むとデコード失敗する
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前");
        let reader = DecodingReader::new(&bytes[..], encoding_rs::UTF_8);
        let err = read_all(reader).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn handles_tiny_read_buffer() {
        // 1 バイトずつ読んでもマルチバイト文字を正しく復元できる
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("名前,年齢");
        let mut reader = DecodingReader::new(&bytes[..], encoding_rs::SHIFT_JIS);
        let mut out = Vec::new();
        let mut one = [0u8; 1];
        loop {
            let n = reader.read(&mut one).unwrap();
            if n == 0 {
                break;
            }
            out.extend_from_slice(&one[..n]);
        }
        assert_eq!(String::from_utf8(out).unwrap(), "名前,年齢");
    }
}
