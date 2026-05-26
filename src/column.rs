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

    /// カンマ区切りの CLI 文字列を ColumnRef のリストへ分解する
    /// 各要素は trim してから parse する。空要素 ("a,,b") は除外せず、空列名として残す
    pub fn parse_csv_list(s: &str) -> Vec<ColumnRef> {
        s.split(',').map(|x| ColumnRef::parse(x.trim())).collect()
    }
}

/// ColumnRef のリストを、ヘッダーと照合して列インデックスのリストに解決する
pub fn resolve_indices(
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

/// 列インデックス列から O(1) lookup 用の bool ビットマップを作る
/// サイズは max+1。空入力なら空ベクタを返す
pub fn build_index_mask(list: &[usize]) -> Vec<bool> {
    match list.iter().max() {
        Some(&max) => {
            let mut m = vec![false; max + 1];
            for &i in list {
                m[i] = true;
            }
            m
        }
        None => Vec::new(),
    }
}

/// 列番号が len の範囲内かを行単位で検証する
/// ヘッダー無し + 列番号指定では init で範囲チェックできないため、各行で呼ぶ
pub fn ensure_in_range(
    indices: impl IntoIterator<Item = usize>,
    len: usize,
) -> Result<(), TransformError> {
    for i in indices {
        if i >= len {
            return Err(TransformError::IndexOutOfRange {
                index: i,
                columns: len,
            });
        }
    }
    Ok(())
}

/// 追加する出力列名が既存ヘッダーまたは他の追加列と衝突しないか検査する
/// ヘッダー無し時は no-op (列名衝突は発生し得ない)
pub fn check_output_conflicts<'a, I>(
    headers: Option<&StringRecord>,
    new_cols: I,
) -> Result<(), TransformError>
where
    I: IntoIterator<Item = &'a str>,
{
    let Some(h) = headers else {
        return Ok(());
    };
    let mut seen: std::collections::HashSet<String> = h.iter().map(|s| s.to_string()).collect();
    for name in new_cols {
        if !seen.insert(name.to_string()) {
            return Err(TransformError::OutputColumnConflict {
                name: name.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_list_mixed_name_and_index() {
        let v = ColumnRef::parse_csv_list("name, 2 ,id");
        assert_eq!(v.len(), 3);
        match &v[0] {
            ColumnRef::Name(s) => assert_eq!(s, "name"),
            other => panic!("想定外: {:?}", other),
        }
        match &v[1] {
            ColumnRef::Index(i) => assert_eq!(*i, 2),
            other => panic!("想定外: {:?}", other),
        }
        match &v[2] {
            ColumnRef::Name(s) => assert_eq!(s, "id"),
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn parse_csv_list_keeps_empty_elements() {
        // split(',') の挙動を維持: 連続カンマは空要素として残る
        let v = ColumnRef::parse_csv_list("a,,b");
        assert_eq!(v.len(), 3);
        match &v[1] {
            ColumnRef::Name(s) => assert_eq!(s, ""),
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn parse_csv_list_single() {
        let v = ColumnRef::parse_csv_list("col1");
        assert_eq!(v.len(), 1);
        match &v[0] {
            ColumnRef::Name(s) => assert_eq!(s, "col1"),
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn build_index_mask_empty() {
        let m = build_index_mask(&[]);
        assert!(m.is_empty());
    }

    #[test]
    fn build_index_mask_single() {
        let m = build_index_mask(&[3]);
        assert_eq!(m, vec![false, false, false, true]);
    }

    #[test]
    fn build_index_mask_multiple_contiguous() {
        let m = build_index_mask(&[0, 1, 2]);
        assert_eq!(m, vec![true, true, true]);
    }

    #[test]
    fn build_index_mask_sparse() {
        let m = build_index_mask(&[0, 4, 2]);
        assert_eq!(m, vec![true, false, true, false, true]);
    }

    #[test]
    fn ensure_in_range_ok() {
        assert!(ensure_in_range([0, 1, 2], 3).is_ok());
    }

    #[test]
    fn ensure_in_range_out() {
        let err = ensure_in_range([0, 3], 3).unwrap_err();
        match err {
            TransformError::IndexOutOfRange { index, columns } => {
                assert_eq!(index, 3);
                assert_eq!(columns, 3);
            }
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn check_output_conflicts_no_headers_is_noop() {
        let r = check_output_conflicts(None, ["a", "a"]);
        assert!(r.is_ok());
    }

    #[test]
    fn check_output_conflicts_against_existing() {
        let h = StringRecord::from(vec!["id", "name"]);
        let err = check_output_conflicts(Some(&h), ["name"]).unwrap_err();
        match err {
            TransformError::OutputColumnConflict { name } => assert_eq!(name, "name"),
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn check_output_conflicts_among_new() {
        let h = StringRecord::from(vec!["id"]);
        let err = check_output_conflicts(Some(&h), ["flag", "flag"]).unwrap_err();
        match err {
            TransformError::OutputColumnConflict { name } => assert_eq!(name, "flag"),
            other => panic!("想定外: {:?}", other),
        }
    }

    #[test]
    fn check_output_conflicts_ok() {
        let h = StringRecord::from(vec!["id", "name"]);
        assert!(check_output_conflicts(Some(&h), ["flag", "score"]).is_ok());
    }
}
