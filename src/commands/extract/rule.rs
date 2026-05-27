// compile 済み extract ルールの内部表現

/// compile 済みの extract ルール
/// 対象列は run 側でヘッダ照合により列インデックスへ解決済み
/// ルールの識別子には out_col をそのまま使う (extract では out_col が必須かつ一意のため)
pub struct CompiledExtractRule {
    /// 抽出に使う正規表現
    pub pattern: regex::Regex,
    /// 抽出対象の列インデックス (解決済み、extract は 1 列固定)
    pub column: usize,
    /// 追加する列の名前 (ヘッダ無し CSV では使われない)
    pub out_col: String,
    /// 複数マッチを連結するときの区切り文字
    pub separator: String,
}
