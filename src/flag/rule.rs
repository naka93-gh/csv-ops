// compile 済み flag ルールの内部表現

/// compile 済みの flag ルール
/// 対象列は run 側でヘッダ照合により列インデックスへ解決済み
/// ルールの識別子には out_col をそのまま使う (flag では out_col が必須かつ一意のため)
pub(crate) struct CompiledFlagRule {
    /// マッチ判定に使う正規表現
    pub pattern: regex::Regex,
    /// 判定対象の列インデックス (解決済み)
    /// 複数列を指定でき、どれか 1 つでもマッチすれば true 扱いになる
    pub columns: Vec<usize>,
    /// 追加する列の名前 (ヘッダ無し CSV では使われない)
    pub out_col: String,
    /// マッチ時に書き込む値
    pub true_value: String,
    /// 非マッチ時に書き込む値
    pub false_value: String,
}
