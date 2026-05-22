pub mod report;

use std::path::PathBuf;

use crate::error::CsvOpsError;
use crate::io::detect_encoding;

use report::{InfoReport, LineEnding};

/// info::run に渡す設定一式
pub struct InfoRequest {
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 区切り文字。None ならヘッダー行から自動判定する
    pub delimiter: Option<u8>,
    /// クォート文字
    pub quote: u8,
}

/// info サブコマンドのエントリポイント
/// CSV を解析してエンコーディング・区切り文字・行数などの情報を返す
pub fn run(request: InfoRequest) -> Result<InfoReport, CsvOpsError> {
    // 行数カウントと各種判定のため全読込する (info は診断用途で全体走査が前提)
    let raw = std::fs::read(&request.input)?;
    let size_bytes = raw.len() as u64;

    // エンコーディング推定 (BOM 検出 + UTF-8 試行 → SJIS fallback)
    let (encoding, bom) = detect_encoding(&raw);

    // 行終端は生バイトで解析する (\r \n は ASCII で多バイト列に紛れない)
    let line_ending = analyze_line_ending(&raw);

    // 推定したエンコーディングでデコード (表示目的なので不正バイトは lossy 許容)
    let (decoded, _, _) = encoding.decode(&raw);

    // 区切り文字: 指定があればそれ、なければヘッダー行から推定
    let delimiter = match request.delimiter {
        Some(d) => d,
        None => detect_delimiter(&decoded),
    };

    // csv で読み、ヘッダーと行数・列数を得る
    // フィールド数がそろわない行も止めずに数える (flexible)
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .quote(request.quote)
        .has_headers(true)
        .flexible(true)
        .from_reader(decoded.as_bytes());

    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    let columns = headers.len();

    let mut rows: u64 = 0;
    for result in rdr.records() {
        result?;
        rows += 1;
    }

    // パスからファイル名のみ取り出す (取れなければパス全体)
    let file = request
        .input
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| request.input.display().to_string());

    Ok(InfoReport {
        file,
        size_bytes,
        encoding: canonical_name(encoding),
        bom,
        delimiter: delimiter_display(delimiter),
        quote: (request.quote as char).to_string(),
        line_ending,
        rows,
        columns,
        headers,
    })
}

/// エンコーディングを設定文字列の語彙 (utf-8 / shift_jis / euc-jp) に直す
fn canonical_name(enc: &'static encoding_rs::Encoding) -> String {
    match enc.name() {
        "UTF-8" => "utf-8",
        "Shift_JIS" => "shift_jis",
        "EUC-JP" => "euc-jp",
        other => other,
    }
    .to_string()
}

/// 区切り文字の表示用文字列。タブは \t と表記する
fn delimiter_display(d: u8) -> String {
    match d {
        b'\t' => "\\t".to_string(),
        _ => (d as char).to_string(),
    }
}

/// ヘッダー行 (最初の行) で候補区切り文字の出現数を数え、最多のものを返す
/// 候補は comma / tab / pipe / semicolon。すべて 0 ならカンマ
/// 同数のときは候補の並び順 (カンマ優先) で先勝ち
fn detect_delimiter(text: &str) -> u8 {
    let first_line = text.lines().next().unwrap_or("");
    const CANDIDATES: [u8; 4] = [b',', b'\t', b'|', b';'];
    let mut best = b',';
    let mut best_count = 0usize;
    for &d in &CANDIDATES {
        let count = first_line.bytes().filter(|&b| b == d).count();
        if count > best_count {
            best_count = count;
            best = d;
        }
    }
    best
}

/// 生バイトを走査して行終端の種類を返す
fn analyze_line_ending(bytes: &[u8]) -> LineEnding {
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
mod tests;
