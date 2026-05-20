// CSV マスキングの本体。
// 設定 (MaskOptions) を受け取り、Read からデータを取り Write に書き出す純粋関数を提供する。
// CLI / ファイル I/O は持たず、Read / Write トレイトを介してのみ外と接続する。

use std::io::{Read, Write};

use crate::column::ColumnRef;
use crate::error::{CsvOpsError, TransformError};
use crate::strategy::MaskStrategy;

/// mask_csv に渡すマスキング設定一式
pub struct MaskOptions<'a> {
    // マスク対象カラム (Name と Index の混在可)
    pub columns: &'a [ColumnRef],
    // 区切り文字 (csv crate の API に合わせて u8 で持つ)
    pub delimiter: u8,
    // マスク戦略
    pub strategy: &'a dyn MaskStrategy,
    // ヘッダ行の有無
    pub has_headers: bool,
}

/// 指定カラムを options.strategy でマスクして writer に書き出す
/// R: Read / W: Write のジェネリクスでファイル / バイト列 / メモリバッファ等を受ける
pub fn mask_csv<R: Read, W: Write>(
    reader: R,
    writer: W,
    options: &MaskOptions<'_>,
) -> Result<(), CsvOpsError> {
    // CSV リーダー / ライターを構築
    // has_headers を csv crate にも伝えてヘッダ行をデータと区別させる
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .from_reader(reader);
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(options.delimiter)
        .from_writer(writer);

    // ヘッダ取得 (ヘッダ無しなら None)
    // ColumnRef::Name の解決と Index の事前範囲チェックに使う
    let headers: Option<csv::StringRecord> = if options.has_headers {
        Some(rdr.headers()?.clone())
    } else {
        None
    };

    // ColumnRef をマスク対象 indices に解決
    // - Name: ヘッダ有りでヘッダ内位置を引く。ヘッダ無しなら NameWithoutHeaders
    // - Index: ヘッダ有り時はヘッダ長で範囲チェック。ヘッダ無し時は各データ行で後段チェック
    let mut indices: Vec<usize> = Vec::with_capacity(options.columns.len());
    for col in options.columns {
        match col {
            ColumnRef::Name(name) => match &headers {
                Some(h) => match h.iter().position(|c| c == name) {
                    Some(i) => indices.push(i),
                    None => {
                        return Err(TransformError::UnknownColumn {
                            name: name.clone(),
                            available: h.iter().map(|c| c.to_string()).collect(),
                        }
                        .into());
                    }
                },
                None => return Err(TransformError::NameWithoutHeaders(name.clone()).into()),
            },
            ColumnRef::Index(i) => {
                // let-chain で「ヘッダ有り かつ 範囲外」を 1 段の if にまとめる
                if let Some(h) = &headers
                    && *i >= h.len()
                {
                    return Err(TransformError::IndexOutOfRange {
                        index: *i,
                        columns: h.len(),
                    }
                    .into());
                }
                indices.push(*i);
            }
        }
    }

    // ヘッダ行はマスクしないので、あればそのまま出力
    if let Some(h) = &headers {
        wtr.write_record(h)?;
    }

    // 各データ行を処理
    // records() はストリーミング iterator で 1 行ずつ読み進む (全行をメモリに載せない)
    for result in rdr.records() {
        let record = result?;

        // ヘッダ無しは事前に範囲チェックできないので、ここでデータ行のカラム数に対し検証
        if headers.is_none() {
            for &i in &indices {
                if i >= record.len() {
                    return Err(TransformError::IndexOutOfRange {
                        index: i,
                        columns: record.len(),
                    }
                    .into());
                }
            }
        }

        // 各フィールドを走査し、対象列なら strategy を通してマスクする
        let masked: Vec<String> = record
            .iter()
            .enumerate()
            .map(|(i, field)| {
                if indices.contains(&i) {
                    options.strategy.mask(field)
                } else {
                    field.to_string()
                }
            })
            .collect();

        wtr.write_record(&masked)?;
    }

    // 内部バッファを最後まで吐き出す
    // 明示しないと Drop 時の自動 flush になりエラーを取りこぼす可能性がある
    wtr.flush()?;
    Ok(())
}
