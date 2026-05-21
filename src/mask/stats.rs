use serde::Serialize;

/// mask 実行の統計
#[derive(Debug, Default, Serialize)]
pub struct MaskStats {
    /// 処理した行数 (ヘッダー除く)
    pub rows_processed: u64,
    /// 1 セル以上マスクした行数
    pub rows_masked: u64,
    /// マスクしたセル総数 (空セルは対象外)
    pub cells_masked: u64,
}

impl MaskStats {
    /// テキスト形式でフォーマットする
    pub fn to_text(&self) -> String {
        format!(
            "処理行数:     {}\nマスク行数:   {}\nマスクセル数: {}",
            self.rows_processed, self.rows_masked, self.cells_masked
        )
    }

    /// JSON 形式でフォーマットする
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("MaskStats は常にシリアライズできる")
    }
}
