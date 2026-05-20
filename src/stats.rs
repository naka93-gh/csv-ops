// サブコマンド実行の統計

/// 処理統計
#[derive(Debug, Default)]
pub struct Stats {
    /// 処理した行数 (ヘッダ除く)
    pub rows_processed: u64,
    /// 置換が入った行数
    pub rows_modified: u64,
}
