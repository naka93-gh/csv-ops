// 各サブコマンドの統計／メタ情報レポートを text / json で書き出す共通 trait
// CLI ハンドラはこの trait 経由でフォーマットを切り替える

/// 統計レポート系の型が実装する整形 trait
/// `to_text` は末尾改行を含めない。改行付与は呼び出し側 (CLI ハンドラ) の責務とする
pub trait StatsReport {
    /// テキスト形式で整形する (末尾改行なし)
    fn to_text(&self) -> String;
    /// JSON 形式で整形する
    fn to_json(&self) -> String;
}
