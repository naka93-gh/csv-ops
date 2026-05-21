pub(crate) mod config;
pub mod stats;
pub(crate) mod transform;

use std::path::PathBuf;

use crate::column::ColumnRef;
use crate::error::CsvOpsError;
use crate::io::resolve_encoding;
use crate::pipeline::{PipelineOptions, run_pipeline};

use config::MaskConfig;
use stats::MaskStats;
use transform::MaskTransform;

/// マスク対象の列指定の供給元
/// Config ファイル指定と CLI 引数指定のどちらかに集約する
pub enum MaskSource {
    /// TOML 設定ファイルのパス
    Config(PathBuf),
    /// CLI 引数による直接指定
    Inline {
        columns: Vec<ColumnRef>,
        mask_char: char,
    },
}

/// mask::run に渡す設定一式
pub struct MaskRequest {
    /// 列指定 (Config ファイル or CLI 引数)
    pub source: MaskSource,
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング名 (utf-8 / shift_jis / euc-jp)
    pub input_encoding: String,
    /// 出力エンコーディング名
    pub output_encoding: String,
    /// 区切り文字
    pub delimiter: u8,
    /// ヘッダー行の有無
    pub has_headers: bool,
    /// dry-run (true なら出力ファイルへ書き込まず、統計のみ集計する)
    pub dry_run: bool,
}

/// mask サブコマンドのエントリポイント
/// 指定列を文字数を保ったままマスクする
pub fn run(request: MaskRequest) -> Result<MaskStats, CsvOpsError> {
    let MaskRequest {
        source,
        input,
        output,
        input_encoding,
        output_encoding,
        delimiter,
        has_headers,
        dry_run,
    } = request;

    // 列指定とマスク文字を解決する (Config 優先)
    let (columns, mask_char) = match source {
        MaskSource::Config(path) => {
            let text = std::fs::read_to_string(&path)?;
            let cfg = MaskConfig::from_toml(&text)?;
            (cfg.columns().to_vec(), cfg.mask_char())
        }
        MaskSource::Inline { columns, mask_char } => (columns, mask_char),
    };

    let mut transform = MaskTransform::new(columns, mask_char);
    let opts = PipelineOptions {
        input,
        output,
        input_encoding: resolve_encoding(&input_encoding)?,
        output_encoding: resolve_encoding(&output_encoding)?,
        // mask は内容のみ変換するので入出力で区切り文字は同一
        input_delimiter: delimiter,
        output_delimiter: delimiter,
        has_headers,
        dry_run,
    };

    let rows = run_pipeline(&mut transform, &opts)?;
    let mut stats = transform.stats;
    stats.rows_processed = rows;
    Ok(stats)
}
