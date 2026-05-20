use csv::StringRecord;
use serde::Deserialize;

use crate::error::TransformError;

/// 列の参照方法
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ColumnRef {
    Name(String),
    Index(usize),
}

impl ColumnRef {
    /// CLI 引数の 1 要素から ColumnRef を作る
    /// 数字のみなら列番号 (Index)、それ以外は列名 (Name) として扱う
    pub fn parse(s: &str) -> Self {
        match s.parse::<usize>() {
            Ok(n) => ColumnRef::Index(n),
            Err(_) => ColumnRef::Name(s.to_string()),
        }
    }
}

/// ColumnRef のリストを、ヘッダーと照合して列インデックスのリストに解決する
pub(crate) fn resolve_indices(
    columns: &[ColumnRef],
    headers: Option<&StringRecord>,
) -> Result<Vec<usize>, TransformError> {
    let mut indices = Vec::with_capacity(columns.len());
    for col in columns {
        match col {
            // 列名
            ColumnRef::Name(name) => match headers {
                // ヘッダー文字列から一致する位置を引く
                Some(h) => match h.iter().position(|c| c == name) {
                    Some(i) => indices.push(i),
                    None => {
                        return Err(TransformError::UnknownColumn {
                            name: name.clone(),
                            available: h.iter().map(|c| c.to_string()).collect(),
                        });
                    }
                },
                // ヘッダーがなければ列名を照合できないのでエラー
                None => return Err(TransformError::NameWithoutHeaders(name.clone())),
            },

            // 列番号
            ColumnRef::Index(i) => {
                // ヘッダー有りなら範囲チェック、ヘッダー無しは呼び出し側で各行チェックする
                if let Some(h) = headers
                    && *i >= h.len()
                {
                    return Err(TransformError::IndexOutOfRange {
                        index: *i,
                        columns: h.len(),
                    });
                }
                indices.push(*i);
            }
        }
    }
    Ok(indices)
}
