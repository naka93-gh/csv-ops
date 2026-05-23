// 辞書ローダ (CSV / TOML 両対応) とベストマッチ判定
// 辞書ファイルのエンコーディングは UTF-8 / Shift_JIS を自動判定する

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{ConfigError, CsvOpsError, DictError, EncodingError};
use crate::io::detect_encoding;
use crate::text::algorithm::Algorithm;
use crate::text::normalize::NormalizeSet;

/// TOML 辞書がサポートするバージョン
const DICT_VERSION: u32 = 1;

/// 辞書の 1 エントリ
#[derive(Debug)]
struct DictEntry {
    canonical: String,
    aliases: Vec<String>,
}

/// ベストマッチの結果
#[derive(Debug)]
pub(crate) struct MatchResult {
    /// 最良エントリの canonical 名
    pub canonical: String,
    /// 類似度スコア (0.0-1.0)
    pub score: f64,
    /// 同点エントリが複数あったか
    pub tie: bool,
}

/// マッチ用に正規化済みの辞書
#[derive(Debug)]
pub(crate) struct Dictionary {
    /// canonical 一覧 (エントリ index で引く、出力用)
    canonicals: Vec<String>,
    /// マッチ候補: (正規化済み候補文字列, エントリ index)
    /// canonical + aliases をすべて正規化して展開したもの
    candidates: Vec<(String, usize)>,
}

impl Dictionary {
    /// 辞書ファイルをロードし、normalize で各候補を正規化して構築する
    /// 形式は拡張子で判定する (.toml は TOML、それ以外は CSV)
    pub fn load(path: &Path, delimiter: u8, normalize: &NormalizeSet) -> Result<Self, CsvOpsError> {
        let entries = if is_toml(path) {
            load_toml(path)?
        } else {
            load_csv(path, delimiter)?
        };
        validate(&entries, path)?;
        Ok(Self::build(entries, normalize))
    }

    /// エントリ群から正規化済みの候補リストを構築する
    fn build(entries: Vec<DictEntry>, normalize: &NormalizeSet) -> Self {
        let mut canonicals = Vec::with_capacity(entries.len());
        let mut candidates = Vec::new();
        for (idx, entry) in entries.into_iter().enumerate() {
            candidates.push((normalize.apply(&entry.canonical), idx));
            for alias in &entry.aliases {
                candidates.push((normalize.apply(alias), idx));
            }
            canonicals.push(entry.canonical);
        }
        Self {
            canonicals,
            candidates,
        }
    }

    /// 正規化済み入力文字列に最も近いエントリを返す
    /// 同点 (同じ最良スコアを別エントリが取る) のときは辞書記述順で先勝ち
    pub fn best_match(&self, normalized_input: &str, algorithm: Algorithm) -> MatchResult {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_entry = 0usize;
        let mut tie = false;
        for (candidate, entry_idx) in &self.candidates {
            let score = algorithm.similarity(normalized_input, candidate);
            if score > best_score {
                best_score = score;
                best_entry = *entry_idx;
                tie = false;
            } else if score == best_score && *entry_idx != best_entry {
                // 別エントリが同点。先勝ちは維持しつつ tie を立てる
                tie = true;
            }
        }
        MatchResult {
            canonical: self.canonicals[best_entry].clone(),
            score: best_score,
            tie,
        }
    }
}

/// 拡張子が toml かどうか (大文字小文字無視)
fn is_toml(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("toml"))
        .unwrap_or(false)
}

/// ファイルを読み、エンコーディングを自動判定してデコードする
fn read_decoded(path: &Path) -> Result<String, CsvOpsError> {
    let raw = std::fs::read(path)?;
    let (encoding, _) = detect_encoding(&raw);
    let (decoded, _, had_errors) = encoding.decode(&raw);
    if had_errors {
        return Err(EncodingError::DecodeFailure {
            encoding: encoding.name().to_string(),
        }
        .into());
    }
    Ok(decoded.into_owned())
}

/// CSV 辞書をロードする (1 列目 canonical、2 列目以降 aliases)
fn load_csv(path: &Path, delimiter: u8) -> Result<Vec<DictEntry>, CsvOpsError> {
    let text = read_decoded(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut entries = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let mut fields = record.iter();
        // 1 列目: canonical (空ならその行は無視)
        let canonical = match fields.next() {
            Some(c) if !c.is_empty() => c.to_string(),
            _ => continue,
        };
        // 2 列目以降: aliases (空欄は無視)
        let aliases = fields
            .filter(|a| !a.is_empty())
            .map(|a| a.to_string())
            .collect();
        entries.push(DictEntry { canonical, aliases });
    }
    Ok(entries)
}

/// TOML 辞書のスキーマ
#[derive(Debug, Deserialize)]
struct TomlDict {
    // version 未指定を検出するため Option で受ける
    version: Option<u32>,
    #[serde(default)]
    entries: Vec<TomlEntry>,
}

/// `[[entries]]` の 1 要素
#[derive(Debug, Deserialize)]
struct TomlEntry {
    canonical: String,
    #[serde(default)]
    aliases: Vec<String>,
}

/// TOML 辞書をロードする
fn load_toml(path: &Path) -> Result<Vec<DictEntry>, CsvOpsError> {
    let text = read_decoded(path)?;
    let dict: TomlDict = toml::from_str(&text).map_err(ConfigError::Parse)?;
    // version は必須
    match dict.version {
        None => return Err(ConfigError::VersionMissing.into()),
        Some(v) if v != DICT_VERSION => {
            return Err(ConfigError::UnsupportedVersion {
                found: v,
                supported: DICT_VERSION,
            }
            .into());
        }
        Some(_) => {}
    }
    let entries = dict
        .entries
        .into_iter()
        .map(|e| DictEntry {
            canonical: e.canonical,
            aliases: e.aliases,
        })
        .collect();
    Ok(entries)
}

/// 辞書の妥当性を検証する (空・canonical 重複・alias 重複をロード時に弾く)
fn validate(entries: &[DictEntry], path: &Path) -> Result<(), DictError> {
    if entries.is_empty() {
        return Err(DictError::Empty(path.to_path_buf()));
    }
    // canonical の重複
    let mut seen_canonical: HashMap<&str, ()> = HashMap::new();
    for entry in entries {
        if seen_canonical.insert(&entry.canonical, ()).is_some() {
            return Err(DictError::DuplicateCanonical(entry.canonical.clone()));
        }
    }
    // 同一 alias が別 canonical に割り当てられていないか
    let mut alias_owner: HashMap<&str, &str> = HashMap::new();
    for entry in entries {
        for alias in &entry.aliases {
            match alias_owner.insert(alias, &entry.canonical) {
                Some(prev) if prev != entry.canonical => {
                    return Err(DictError::DuplicateAlias {
                        alias: alias.clone(),
                        canonicals: vec![prev.to_string(), entry.canonical.clone()],
                    });
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
