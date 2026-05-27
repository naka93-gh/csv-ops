pub mod report;

use std::path::PathBuf;

use crate::error::CsvOpsError;
use crate::io::{analyze_line_ending, detect_encoding};

use report::InfoReport;

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

/// 先頭 10 行で各候補区切り文字の列数を試算し、最も安定する候補を返す
/// 各候補について「列数の最頻値」と「最頻値が現れた行数」を取り、
/// `(最頻列数の出現行数 × 1000 + 最頻列数)` をスコアとして最大の候補を選ぶ。
/// 列数が 1 にしかならない候補は区切り文字として機能していないので除外。
/// すべての候補が機能しなければデフォルトでカンマ。
/// 同点は候補の並び順 (カンマ → タブ → パイプ → セミコロン) で先勝ち。
fn detect_delimiter(text: &str) -> u8 {
    const CANDIDATES: [u8; 4] = [b',', b'\t', b'|', b';'];
    const SAMPLE_LINES: usize = 10;
    let lines: Vec<&str> = text.lines().take(SAMPLE_LINES).collect();
    if lines.is_empty() {
        return b',';
    }
    let mut best = b',';
    let mut best_score: i64 = -1;
    for &d in &CANDIDATES {
        // 各行の列数 = 区切り文字の出現回数 + 1 (簡易にクォート内も含めてカウント)
        let cols: Vec<usize> = lines
            .iter()
            .map(|line| line.bytes().filter(|&b| b == d).count() + 1)
            .collect();
        let (mode, mode_count) = most_common(&cols);
        // 区切り文字として機能していない (1 列にしかならない) ものはスキップ
        if mode <= 1 {
            continue;
        }
        // 安定した列数を持つ行が多いほど良く、同点なら列数が多い候補を優先
        let score = (mode_count as i64) * 1000 + mode as i64;
        if score > best_score {
            best_score = score;
            best = d;
        }
    }
    best
}

/// 値の列の中で最も頻出する値と、その出現回数を返す
/// 同点は最初に現れた値が勝つ。空入力なら (1, 0) を返す
fn most_common(values: &[usize]) -> (usize, usize) {
    let mut counts: Vec<(usize, usize)> = Vec::new();
    for &v in values {
        if let Some(c) = counts.iter_mut().find(|(k, _)| *k == v) {
            c.1 += 1;
        } else {
            counts.push((v, 1));
        }
    }
    // 同点は最初に挿入された (= 最初に現れた) ものを残すため、安定な max を取る
    counts
        .into_iter()
        .reduce(|a, b| if b.1 > a.1 { b } else { a })
        .unwrap_or((1, 0))
}

#[cfg(test)]
mod tests;
