// 各サブコマンドの統計／メタ情報レポートを text / json で書き出す共通 trait
// CLI ハンドラはこの trait 経由でフォーマットを切り替える

use serde::Serialize;

/// 統計レポート系の型が実装する整形 trait
/// `to_text` は末尾改行を含めない。改行付与は呼び出し側 (CLI ハンドラ) の責務とする
pub trait StatsReport: Serialize {
    /// テキスト形式で整形する (末尾改行なし)
    fn to_text(&self) -> String;

    /// JSON 形式で整形する (Serialize 派生の pretty 出力)
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("StatsReport は常にシリアライズできる")
    }
}
