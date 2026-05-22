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
    fn text(&self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
            LineEnding::Mixed => "mixed",
            LineEnding::None => "none",
        }
    }
}

/// info の解析結果
/// JSON 出力は serde 由来。delimiter / quote は表示用文字列で持つ。
#[derive(Debug, Serialize)]
pub struct InfoReport {
    /// ファイル名 (パスではなく名前のみ)
    pub file: String,
    /// ファイルサイズ (バイト)
    pub size_bytes: u64,
    /// 推定エンコーディング (utf-8 / shift_jis / euc-jp)
    pub encoding: String,
    /// UTF-8 BOM の有無
    pub bom: bool,
    /// 区切り文字 (表示用、タブは \t)
    pub delimiter: String,
    /// クォート文字
    pub quote: String,
    /// 行終端
    pub line_ending: LineEnding,
    /// データ行数 (ヘッダー除く)
    pub rows: u64,
    /// 列数
    pub columns: usize,
    /// ヘッダーの全カラム名
    pub headers: Vec<String>,
}

impl InfoReport {
    /// テキスト形式でフォーマットする
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("File:        {}\n", self.file));
        out.push_str(&format!(
            "Size:        {} ({} bytes)\n",
            human_size(self.size_bytes),
            group_digits(self.size_bytes)
        ));
        out.push_str(&format!("Encoding:    {}\n", self.encoding_text()));
        out.push_str(&format!("Delimiter:   {}\n", self.delimiter));
        out.push_str(&format!("Quote:       {}\n", self.quote));
        out.push_str(&format!("Line ending: {}\n", self.line_ending.text()));
        out.push_str(&format!(
            "Rows:        {} (excluding header)\n",
            group_digits(self.rows)
        ));
        out.push_str(&format!("Columns:     {}\n", self.columns));
        out.push_str(&format!("Headers:     {}", self.headers.join(", ")));
        out
    }

    /// JSON 形式でフォーマットする
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("InfoReport は常にシリアライズできる")
    }

    /// エンコーディングのテキスト表示 (UTF-8 は BOM 有無を含める)
    fn encoding_text(&self) -> String {
        match self.encoding.as_str() {
            "utf-8" => format!("UTF-8 ({})", if self.bom { "with BOM" } else { "no BOM" }),
            "shift_jis" => "Shift_JIS".to_string(),
            "euc-jp" => "EUC-JP".to_string(),
            other => other.to_string(),
        }
    }
}

/// バイト数を人間可読なサイズ文字列にする
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}

/// 数値を 3 桁ごとにカンマ区切りする
fn group_digits(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(b as char);
    }
    out
}

#[cfg(test)]
mod tests;
