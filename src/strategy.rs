/// マスク戦略の trait
/// 値を受け取って何らかの変換 (塗り潰し / 固定文字列 / ハッシュ等) を返す
pub trait MaskStrategy {
    fn mask(&self, value: &str) -> String;
}

/// 文字埋め戦略
/// 入力と同じ文字数の ch で埋める
/// 例: CharFill { ch: '*' } に "abc" を渡すと "***"
pub struct CharFill {
    pub ch: char,
}

impl MaskStrategy for CharFill {
    fn mask(&self, value: &str) -> String {
        // chars().count() を使うのは、bytes 単位の len() だと日本語等のマルチバイトで桁数が狂うため
        self.ch.to_string().repeat(value.chars().count())
    }
}
