// 行終端の検出と表現
// \r \n は ASCII で SJIS / EUC-JP / UTF-8 のどの多バイト列にも紛れないため、
// デコード前の生バイトのまま走査して安全

use serde::Serialize;

/// 行終端の種類
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    Lf,
    Crlf,
    /// CRLF と LF が混在
    Mixed,
    /// 改行が 1 つも無い
    None,
}

impl LineEnding {
    /// テキスト表示用の名前
    pub fn text(&self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
            LineEnding::Mixed => "mixed",
            LineEnding::None => "none",
        }
    }
}

/// バイト列を走査して行終端の種類を返す
pub fn analyze_line_ending(bytes: &[u8]) -> LineEnding {
    let mut crlf = false;
    let mut lone_lf = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            if i > 0 && bytes[i - 1] == b'\r' {
                crlf = true;
            } else {
                lone_lf = true;
            }
        }
    }
    match (crlf, lone_lf) {
        (true, true) => LineEnding::Mixed,
        (true, false) => LineEnding::Crlf,
        (false, true) => LineEnding::Lf,
        (false, false) => LineEnding::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_crlf() {
        assert_eq!(analyze_line_ending(b"a\r\nb\r\n"), LineEnding::Crlf);
    }

    #[test]
    fn detects_lf() {
        assert_eq!(analyze_line_ending(b"a\nb\n"), LineEnding::Lf);
    }

    #[test]
    fn detects_mixed() {
        assert_eq!(analyze_line_ending(b"a\r\nb\nc\r\n"), LineEnding::Mixed);
    }

    #[test]
    fn detects_none_when_no_newline() {
        assert_eq!(analyze_line_ending(b"abc"), LineEnding::None);
    }

    #[test]
    fn empty_input_is_none() {
        assert_eq!(analyze_line_ending(b""), LineEnding::None);
    }
}
