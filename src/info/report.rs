use serde::Serialize;

use crate::StatsReport;
use crate::io::LineEnding;

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

impl StatsReport for InfoReport {
    fn to_text(&self) -> String {
        let lines = vec![
            format!("File:        {}", self.file),
            format!(
                "Size:        {} ({} bytes)",
                human_size(self.size_bytes),
                group_digits(self.size_bytes)
            ),
            format!("Encoding:    {}", self.encoding_text()),
            format!("Delimiter:   {}", self.delimiter),
            format!("Quote:       {}", self.quote),
            format!("Line ending: {}", self.line_ending.text()),
            format!(
                "Rows:        {} (excluding header)",
                group_digits(self.rows)
            ),
            format!("Columns:     {}", self.columns),
            format!("Headers:     {}", self.headers.join(", ")),
        ];
        lines.join("\n")
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
