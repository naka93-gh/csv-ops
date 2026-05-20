use serde::Deserialize;

use crate::column::ColumnRef;

/// 設定ファイル全体に対応する型
/// #[derive(Deserialize)] で toml クレートが TOML → 構造体への変換を自動生成する
#[derive(Debug, Deserialize)]
pub struct MaskConfig {
    pub targets: Vec<Target>,
}

/// マスク対象 1 ファイル分の設定
/// delimiter / has_headers / encoding は省略可能で、未指定時はデフォルト値が入る
#[derive(Debug, Deserialize)]
pub struct Target {
    pub filepath: String,
    pub columns: Vec<ColumnRef>,
    // デリミター
    #[serde(default = "default_delimiter")]
    pub delimiter: String,
    // ヘッダ行があるか
    #[serde(default = "default_has_headers")]
    pub has_headers: bool,
    // 入力ファイルのエンコーディング
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

/// delimiter のデフォルト値
/// #[serde(default = "...")] から参照されるので関数として定義する必要がある
fn default_delimiter() -> String {
    ",".to_string()
}

/// has_headers のデフォルト値
fn default_has_headers() -> bool {
    true
}

/// encoding のデフォルト値
fn default_encoding() -> String {
    "utf-8".to_string()
}
