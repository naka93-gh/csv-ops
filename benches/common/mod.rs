// ベンチ用の決定論的データジェネレータ
// 各 bench から `mod common;` で読み込む

#![allow(dead_code)]

use std::fs;
use std::path::Path;

use rand::{RngExt, SeedableRng, rngs::SmallRng};

/// 小ケース (1K 行)
pub const SMALL_ROWS: usize = 1_000;
/// 中ケース (100K 行)
pub const MEDIUM_ROWS: usize = 100_000;
/// similarity の中ケース (辞書 × 行数で爆発するので縮小)
pub const SIMILARITY_MEDIUM_ROWS: usize = 10_000;

/// 既定 seed (再現性確保のため固定)
pub const SEED: u64 = 0xCAFE_F00D;

/// 5 列の汎用 CSV を生成する (id, name, email, phone, note)
/// mask / replace の入力として使う
pub fn gen_csv(rows: usize, seed: u64) -> String {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut out = String::from("id,name,email,phone,note\n");
    for i in 0..rows {
        let name = random_word(&mut rng, 6);
        let email = format!("{}@example.com", random_word(&mut rng, 8));
        let phone = format!(
            "{:03}-{:04}-{:04}",
            rng.random_range(0..1000),
            rng.random_range(0..10000),
            rng.random_range(0..10000),
        );
        // note は "old_<4 文字>_<数字>" の形式。replace のベンチでマッチ対象になる
        let note = format!(
            "old_{}_{}",
            random_word(&mut rng, 4),
            rng.random_range(0..100)
        );
        out.push_str(&format!("{},{},{},{},{}\n", i, name, email, phone, note));
    }
    out
}

/// similarity 入力 CSV を生成する (id, region)
/// 辞書の canonical / alias / ノイズを混ぜた region 列を持つ
pub fn gen_similarity_input_csv(rows: usize, dict_entries: usize, seed: u64) -> String {
    let mut rng = SmallRng::seed_from_u64(seed);
    let entries = synthetic_entries(dict_entries);
    let mut out = String::from("id,region\n");
    for i in 0..rows {
        let entry = &entries[rng.random_range(0..entries.len())];
        // 50% canonical そのまま, 30% alias の 1 つ, 20% noise (末尾に "X")
        let r: u32 = rng.random_range(0..100);
        let value = if r < 50 {
            entry.canonical.clone()
        } else if r < 80 && !entry.aliases.is_empty() {
            entry.aliases[rng.random_range(0..entry.aliases.len())].clone()
        } else {
            format!("{}X", entry.canonical)
        };
        out.push_str(&format!("{},{}\n", i, value));
    }
    out
}

/// similarity 用の CSV 辞書を生成する (canonical, alias1, alias2, alias3)
pub fn gen_dict_csv(entries: usize) -> String {
    let entries = synthetic_entries(entries);
    let mut out = String::from("canonical,alias1,alias2,alias3\n");
    for e in &entries {
        // 3 個未満の alias は空欄で埋める
        let a1 = e.aliases.first().cloned().unwrap_or_default();
        let a2 = e.aliases.get(1).cloned().unwrap_or_default();
        let a3 = e.aliases.get(2).cloned().unwrap_or_default();
        out.push_str(&format!("{},{},{},{}\n", e.canonical, a1, a2, a3));
    }
    out
}

/// 10 ルールの単純置換 TOML を生成する
/// "old_a"〜"old_j" → "new_a"〜"new_j"。gen_csv の note 列に対するマッチをある程度想定する
pub fn gen_replace_10_rules_toml() -> String {
    let mut out = String::from("version = 1\n");
    for c in b'a'..=b'j' {
        let ch = c as char;
        out.push_str(&format!(
            "\n[[rules]]\nfrom = \"old_{}\"\nto = \"new_{}\"\n",
            ch, ch
        ));
    }
    out
}

struct Entry {
    canonical: String,
    aliases: Vec<String>,
}

/// "Region-000" / "Region-001" のような決定論的なエントリを作る
/// alias は半角小文字・スペース挿入・全大文字の 3 種
fn synthetic_entries(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|i| Entry {
            canonical: format!("Region-{:03}", i),
            aliases: vec![
                format!("region{:03}", i),
                format!("Region {:03}", i),
                format!("REGION-{:03}", i),
            ],
        })
        .collect()
}

/// 指定長のランダム英数字文字列
fn random_word(rng: &mut SmallRng, len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}

/// content を path に書き出す
pub fn write_file(path: &Path, content: &str) {
    fs::write(path, content).expect("ベンチ用ファイル書き込みに失敗");
}
