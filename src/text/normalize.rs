// 文字列正規化。表記揺れを吸収して similarity のマッチ精度を上げる
// 正規化ステップは指定順に関わらず固定順 (ORDER) で適用する

use unicode_normalization::UnicodeNormalization;

use crate::error::ConfigError;

/// 正規化ステップ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizeStep {
    /// NFKC 正規化 (Unicode 互換分解 + 合成)
    Nfkc,
    /// 半角カナ → 全角カナ
    Fullwidth,
    /// 長音記号類を統一
    Prolonged,
    /// ひらがな → カタカナ
    Kana,
    /// 大文字小文字統一 (小文字へ)
    Casefold,
    /// 空白除去
    Whitespace,
}

/// 適用順。リスト指定順は無視し、この順で適用する
/// (互換分解を先、空白除去を最後にする)
const ORDER: [NormalizeStep; 6] = [
    NormalizeStep::Nfkc,
    NormalizeStep::Fullwidth,
    NormalizeStep::Prolonged,
    NormalizeStep::Kana,
    NormalizeStep::Casefold,
    NormalizeStep::Whitespace,
];

/// 適用する正規化ステップの集合 (ORDER 順に保持)
#[derive(Debug, Clone)]
pub struct NormalizeSet {
    steps: Vec<NormalizeStep>,
}

impl NormalizeSet {
    /// デフォルト構成 (nfkc, casefold, whitespace)
    pub fn default_set() -> Self {
        Self {
            steps: vec![
                NormalizeStep::Nfkc,
                NormalizeStep::Casefold,
                NormalizeStep::Whitespace,
            ],
        }
    }

    /// 名前トークンのリストから構築する。未知トークンはエラー
    pub fn from_names<S: AsRef<str>>(names: &[S]) -> Result<Self, ConfigError> {
        let mut selected = Vec::new();
        for name in names {
            let step = parse_step(name.as_ref())?;
            if !selected.contains(&step) {
                selected.push(step);
            }
        }
        // ORDER 順に並べ直す
        let steps = ORDER
            .iter()
            .copied()
            .filter(|s| selected.contains(s))
            .collect();
        Ok(Self { steps })
    }

    /// 文字列に正規化を適用する
    pub fn apply(&self, s: &str) -> String {
        let mut out = s.to_string();
        for step in &self.steps {
            out = apply_step(*step, &out);
        }
        out
    }
}

/// 正規化オプション名を NormalizeStep に変換する
fn parse_step(name: &str) -> Result<NormalizeStep, ConfigError> {
    match name {
        "nfkc" => Ok(NormalizeStep::Nfkc),
        "fullwidth" => Ok(NormalizeStep::Fullwidth),
        "prolonged" => Ok(NormalizeStep::Prolonged),
        "kana" => Ok(NormalizeStep::Kana),
        "casefold" => Ok(NormalizeStep::Casefold),
        "whitespace" => Ok(NormalizeStep::Whitespace),
        other => Err(ConfigError::Validation(format!(
            "未知の正規化オプション: {} (nfkc / fullwidth / prolonged / kana / casefold / whitespace)",
            other
        ))),
    }
}

/// 1 ステップを適用する
fn apply_step(step: NormalizeStep, s: &str) -> String {
    match step {
        NormalizeStep::Nfkc => s.nfkc().collect(),
        NormalizeStep::Casefold => s.to_lowercase(),
        NormalizeStep::Whitespace => s.chars().filter(|c| !c.is_whitespace()).collect(),
        NormalizeStep::Kana => s.chars().map(hira_to_kata).collect(),
        NormalizeStep::Prolonged => s.chars().map(unify_prolonged).collect(),
        NormalizeStep::Fullwidth => halfwidth_kana_to_fullwidth(s),
    }
}

/// ひらがなをカタカナへ寄せる (U+3041..=U+3096)
fn hira_to_kata(c: char) -> char {
    match c {
        'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        _ => c,
    }
}

/// 長音記号類を U+30FC 'ー' に統一する
/// 全角長音・半角長音・長いダッシュ類・全角ハイフンマイナスを対象にする
fn unify_prolonged(c: char) -> char {
    match c {
        '\u{30FC}' // ー カタカナ・ひらがな長音
        | '\u{FF70}' // ｰ 半角長音
        | '\u{2014}' // — em dash
        | '\u{2015}' // ― horizontal bar
        | '\u{FF0D}' => 'ー', // － 全角ハイフンマイナス
        _ => c,
    }
}

/// 半角カナを全角カナへ変換する。後続の濁点・半濁点は合成する
fn halfwidth_kana_to_fullwidth(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let Some(base) = hw_base(c) else {
            out.push(c);
            continue;
        };
        // 次が濁点 (ﾞ) / 半濁点 (ﾟ) なら合成を試みる
        match chars.peek() {
            Some('\u{FF9E}') => {
                if let Some(voiced) = voiced(base) {
                    out.push(voiced);
                    chars.next();
                    continue;
                }
            }
            Some('\u{FF9F}') => {
                if let Some(semi) = semi_voiced(base) {
                    out.push(semi);
                    chars.next();
                    continue;
                }
            }
            _ => {}
        }
        out.push(base);
    }
    out
}

/// 半角カナ 1 文字を全角の基底文字へ変換する (濁点合成前)
fn hw_base(c: char) -> Option<char> {
    let mapped = match c {
        '\u{FF61}' => '。',
        '\u{FF62}' => '「',
        '\u{FF63}' => '」',
        '\u{FF64}' => '、',
        '\u{FF65}' => '・',
        '\u{FF66}' => 'ヲ',
        '\u{FF67}' => 'ァ',
        '\u{FF68}' => 'ィ',
        '\u{FF69}' => 'ゥ',
        '\u{FF6A}' => 'ェ',
        '\u{FF6B}' => 'ォ',
        '\u{FF6C}' => 'ャ',
        '\u{FF6D}' => 'ュ',
        '\u{FF6E}' => 'ョ',
        '\u{FF6F}' => 'ッ',
        '\u{FF70}' => 'ー',
        '\u{FF71}' => 'ア',
        '\u{FF72}' => 'イ',
        '\u{FF73}' => 'ウ',
        '\u{FF74}' => 'エ',
        '\u{FF75}' => 'オ',
        '\u{FF76}' => 'カ',
        '\u{FF77}' => 'キ',
        '\u{FF78}' => 'ク',
        '\u{FF79}' => 'ケ',
        '\u{FF7A}' => 'コ',
        '\u{FF7B}' => 'サ',
        '\u{FF7C}' => 'シ',
        '\u{FF7D}' => 'ス',
        '\u{FF7E}' => 'セ',
        '\u{FF7F}' => 'ソ',
        '\u{FF80}' => 'タ',
        '\u{FF81}' => 'チ',
        '\u{FF82}' => 'ツ',
        '\u{FF83}' => 'テ',
        '\u{FF84}' => 'ト',
        '\u{FF85}' => 'ナ',
        '\u{FF86}' => 'ニ',
        '\u{FF87}' => 'ヌ',
        '\u{FF88}' => 'ネ',
        '\u{FF89}' => 'ノ',
        '\u{FF8A}' => 'ハ',
        '\u{FF8B}' => 'ヒ',
        '\u{FF8C}' => 'フ',
        '\u{FF8D}' => 'ヘ',
        '\u{FF8E}' => 'ホ',
        '\u{FF8F}' => 'マ',
        '\u{FF90}' => 'ミ',
        '\u{FF91}' => 'ム',
        '\u{FF92}' => 'メ',
        '\u{FF93}' => 'モ',
        '\u{FF94}' => 'ヤ',
        '\u{FF95}' => 'ユ',
        '\u{FF96}' => 'ヨ',
        '\u{FF97}' => 'ラ',
        '\u{FF98}' => 'リ',
        '\u{FF99}' => 'ル',
        '\u{FF9A}' => 'レ',
        '\u{FF9B}' => 'ロ',
        '\u{FF9C}' => 'ワ',
        '\u{FF9D}' => 'ン',
        '\u{FF9E}' => '゛',
        '\u{FF9F}' => '゜',
        _ => return None,
    };
    Some(mapped)
}

/// 濁点付きの全角カナを返す
fn voiced(c: char) -> Option<char> {
    let v = match c {
        'カ' => 'ガ',
        'キ' => 'ギ',
        'ク' => 'グ',
        'ケ' => 'ゲ',
        'コ' => 'ゴ',
        'サ' => 'ザ',
        'シ' => 'ジ',
        'ス' => 'ズ',
        'セ' => 'ゼ',
        'ソ' => 'ゾ',
        'タ' => 'ダ',
        'チ' => 'ヂ',
        'ツ' => 'ヅ',
        'テ' => 'デ',
        'ト' => 'ド',
        'ハ' => 'バ',
        'ヒ' => 'ビ',
        'フ' => 'ブ',
        'ヘ' => 'ベ',
        'ホ' => 'ボ',
        'ウ' => 'ヴ',
        _ => return None,
    };
    Some(v)
}

/// 半濁点付きの全角カナを返す
fn semi_voiced(c: char) -> Option<char> {
    let v = match c {
        'ハ' => 'パ',
        'ヒ' => 'ピ',
        'フ' => 'プ',
        'ヘ' => 'ペ',
        'ホ' => 'ポ',
        _ => return None,
    };
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_names_rejects_unknown_option() {
        assert!(NormalizeSet::from_names(&["nfkc", "bogus"]).is_err());
    }

    #[test]
    fn default_set_normalizes_width_case_space() {
        let set = NormalizeSet::default_set();
        // 全角英数 → 半角 (nfkc)、大文字 → 小文字 (casefold)、空白除去 (whitespace)
        assert_eq!(set.apply("Ａ Ｂ Ｃ"), "abc");
    }

    #[test]
    fn nfkc_folds_fullwidth_digits() {
        let set = NormalizeSet::from_names(&["nfkc"]).unwrap();
        assert_eq!(set.apply("１２３"), "123");
    }

    #[test]
    fn kana_maps_hiragana_to_katakana() {
        let set = NormalizeSet::from_names(&["kana"]).unwrap();
        assert_eq!(set.apply("とうきょう"), "トウキョウ");
    }

    #[test]
    fn fullwidth_maps_halfwidth_kana() {
        let set = NormalizeSet::from_names(&["fullwidth"]).unwrap();
        assert_eq!(set.apply("ｶﾀｶﾅ"), "カタカナ");
    }

    #[test]
    fn fullwidth_combines_voiced_mark() {
        let set = NormalizeSet::from_names(&["fullwidth"]).unwrap();
        assert_eq!(set.apply("ｶﾞｷﾞ"), "ガギ");
        assert_eq!(set.apply("ﾊﾟ"), "パ");
    }

    #[test]
    fn whitespace_removes_all_spaces() {
        let set = NormalizeSet::from_names(&["whitespace"]).unwrap();
        // 半角・全角空白とも除去
        assert_eq!(set.apply("東京 都\u{3000}庁"), "東京都庁");
    }

    #[test]
    fn prolonged_unifies_dashes() {
        let set = NormalizeSet::from_names(&["prolonged"]).unwrap();
        assert_eq!(set.apply("コーヒー"), "コーヒー");
        assert_eq!(set.apply("コ\u{FF0D}ヒ\u{2014}"), "コーヒー");
    }

    #[test]
    fn application_order_is_fixed_regardless_of_input_order() {
        let a = NormalizeSet::from_names(&["whitespace", "nfkc", "casefold"]).unwrap();
        let b = NormalizeSet::from_names(&["nfkc", "casefold", "whitespace"]).unwrap();
        assert_eq!(a.apply("Ｔ Ｏ Ｋ"), b.apply("Ｔ Ｏ Ｋ"));
    }
}
