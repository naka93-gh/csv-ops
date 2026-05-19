// RecordTransform trait。
// 各サブコマンドの行ごとの変換ロジックを抽象化する。
// 最小実装で開始し、必要時に拡張する (pub(crate) なので破壊的変更は外部影響なし)。
// Phase 0 では定義のみ、Phase 1 (replace) で実装・使用を開始する。

#![allow(dead_code)]

use crate::error::TransformError;

/// CSV の 1 レコードを変換する trait。
/// 実装は record を in-place で書き換え、結果として行の状態を返す。
pub(crate) trait RecordTransform {
    fn apply(&self, record: &mut csv::StringRecord) -> Result<TransformOutcome, TransformError>;
}

/// 変換結果の状態。
/// 統計集計 (Modified 行数 / Skipped 行数) と pipeline 制御で使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransformOutcome {
    /// 何らかの変更が入った。
    Modified,
    /// 変換対象だが結果として変わらなかった。
    Unchanged,
    /// この行はスキップする (出力にも書かない用途を想定、現状未使用)。
    Skipped,
}
