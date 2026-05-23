// compile 済み similarity ルールの内部表現

use crate::text::algorithm::Algorithm;
use crate::text::normalize::NormalizeSet;

use super::dict::Dictionary;

/// compile 済みの similarity ルール
/// 対象列はヘッダ照合で列インデックスへ解決済み、辞書はロード + 正規化済み
#[derive(Debug)]
pub(crate) struct CompiledSimilarityRule {
    /// マッチ対象の列インデックス (similarity は 1 列固定)
    pub column: usize,
    /// 正規化 + ロード済みの辞書
    pub dict: Dictionary,
    /// マッチ名を出力する列名
    pub out_col: String,
    /// スコアを出力する列名
    pub score_col: String,
    /// この値以上をマッチとみなすしきい値 (0.0-1.0)
    pub threshold: f64,
    /// 入力セルに適用する正規化 (辞書側と同じものを使う)
    pub normalize: NormalizeSet,
    /// 類似度アルゴリズム
    pub algorithm: Algorithm,
}
