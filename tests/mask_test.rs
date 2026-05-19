// mask サブコマンドの統合テスト。
// mask 本体 / MaskStrategy 戦略 / エンコーディング解決を 1 ファイルに集約。
// 戦略・エンコーディングはサブモジュールで分けて見通しを保つ。

use std::io::Cursor;

use csv_ops::{
    CharFill, ColumnSpec, CsvOpsError, EncodingError, MaskOptions, MaskStrategy, TransformError,
    mask_csv, resolve_encoding,
};
use encoding_rs_io::DecodeReaderBytesBuilder;

// =========================================================================
// mask 本体テスト
// =========================================================================

/// マスク対象カラムだけ置換される
#[test]
fn masks_target_columns_only() {
    let input = "header1,header2,header3\nfoo,bar,baz\nhello,world,!\n";
    let columns = [ColumnSpec::Name("header2".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap();
    let result = String::from_utf8(output).unwrap();
    assert_eq!(
        result,
        "header1,header2,header3\nfoo,***,baz\nhello,*****,!\n"
    );
}

/// 存在しないカラム指定はエラー
#[test]
fn returns_error_for_unknown_column() {
    let input = "a,b\n1,2\n";
    let columns = [ColumnSpec::Name("nope".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::UnknownColumn { ref name, .. }) if name == "nope"
    ));
}

/// 存在しないカラム指定のメッセージに「利用可能カラム一覧」が含まれる
#[test]
fn unknown_column_error_lists_available_columns() {
    let input = "header1,header2,header3\n1,2,3\n";
    let columns = [ColumnSpec::Name("nope".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("nope"), "msg = {}", msg);
    assert!(msg.contains("header1"), "msg = {}", msg);
    assert!(msg.contains("header2"), "msg = {}", msg);
    assert!(msg.contains("header3"), "msg = {}", msg);
}

/// CSV パースエラーに行番号が含まれる
/// 2 行目でカラム数が不整合だと csv::Error が発生し、position 経由で行番号が取れる
#[test]
fn csv_error_carries_line_number() {
    let input = "a,b,c\n1,2\n";
    let columns = [ColumnSpec::Name("a".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap_err();
    match err {
        CsvOpsError::Csv { line, .. } => {
            assert!(line.is_some(), "行番号が取れていない: {:?}", line);
        }
        other => panic!("Csv エラーを期待したが別の variant: {:?}", other),
    }
}

/// タブ区切りも扱える
#[test]
fn handles_tab_delimiter() {
    let input = "a\tb\n1\t2\n";
    let columns = [ColumnSpec::Name("b".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b'\t',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "a\tb\n1\t*\n");
}

/// ヘッダ有りで列番号 (0-indexed) 指定が動く
#[test]
fn masks_by_index_with_headers() {
    let input = "a,b,c\nfoo,bar,baz\n";
    let columns = [ColumnSpec::Index(1)];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "a,b,c\nfoo,***,baz\n");
}

/// ヘッダ無し設定で列番号指定が動く (ヘッダ行も出力されない)
#[test]
fn masks_by_index_without_headers() {
    let input = "foo,bar,baz\nhello,world,!\n";
    let columns = [ColumnSpec::Index(0)];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: false,
        },
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "***,bar,baz\n*****,world,!\n"
    );
}

/// ヘッダ無し設定で名前指定するとエラー
#[test]
fn name_spec_errors_without_headers() {
    let input = "1,2,3\n";
    let columns = [ColumnSpec::Name("a".to_string())];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: false,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::NameWithoutHeaders(ref n)) if n == "a"
    ));
}

/// 列番号が範囲外だとエラー (ヘッダ有り)
#[test]
fn out_of_range_index_errors_with_headers() {
    let input = "a,b\n1,2\n";
    let columns = [ColumnSpec::Index(5)];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: true,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2,
        })
    ));
}

/// 列番号が範囲外だとエラー (ヘッダ無し、データ行で検知)
#[test]
fn out_of_range_index_errors_without_headers() {
    let input = "1,2\n";
    let columns = [ColumnSpec::Index(5)];
    let strategy = CharFill { ch: '*' };
    let mut output = Vec::new();
    let err = mask_csv(
        input.as_bytes(),
        &mut output,
        &MaskOptions {
            columns: &columns,
            delimiter: b',',
            strategy: &strategy,
            has_headers: false,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        CsvOpsError::Transform(TransformError::IndexOutOfRange {
            index: 5,
            columns: 2,
        })
    ));
}

// =========================================================================
// MaskStrategy / CharFill のテスト
// =========================================================================

mod strategy_tests {
    use super::*;

    /// ASCII 文字列を同じ長さの mask_char で埋める
    #[test]
    fn char_fill_replaces_ascii_with_same_length() {
        let s = CharFill { ch: '*' };
        assert_eq!(s.mask("abc"), "***");
        assert_eq!(s.mask("hello"), "*****");
    }

    /// 空文字列は空文字列のまま
    #[test]
    fn char_fill_keeps_empty_string_empty() {
        let s = CharFill { ch: '*' };
        assert_eq!(s.mask(""), "");
    }

    /// 日本語などのマルチバイト文字も「文字数」単位で塗り潰す (バイト長ではない)
    #[test]
    fn char_fill_counts_characters_not_bytes() {
        let s = CharFill { ch: '*' };
        // "あいう" は UTF-8 で 9 バイトだが 3 文字
        assert_eq!(s.mask("あいう"), "***");
        // "a日b" は 5 バイトだが 3 文字
        assert_eq!(s.mask("a日b"), "***");
    }

    /// 絵文字 (サロゲートペア相当) も Unicode scalar 単位で 1 文字扱い
    #[test]
    fn char_fill_handles_emoji_as_scalar() {
        let s = CharFill { ch: '*' };
        // "🦀" は 1 文字 (U+1F980)
        assert_eq!(s.mask("🦀"), "*");
    }

    /// 塗り潰し文字を切り替えられる
    #[test]
    fn char_fill_uses_configured_char() {
        let s = CharFill { ch: 'X' };
        assert_eq!(s.mask("abc"), "XXX");
    }
}

// =========================================================================
// エンコーディング解決のテスト
// =========================================================================

mod encoding_tests {
    use super::*;

    /// utf-8 が解決できる
    #[test]
    fn resolves_utf8() {
        let enc = resolve_encoding("utf-8").unwrap();
        assert_eq!(enc.name(), "UTF-8");
    }

    /// shift_jis が解決できる
    #[test]
    fn resolves_shift_jis() {
        let enc = resolve_encoding("shift_jis").unwrap();
        assert_eq!(enc.name(), "Shift_JIS");
    }

    /// euc-jp が解決できる
    #[test]
    fn resolves_euc_jp() {
        let enc = resolve_encoding("euc-jp").unwrap();
        assert_eq!(enc.name(), "EUC-JP");
    }

    /// サポート外の名前はエラー
    #[test]
    fn rejects_unknown_encoding() {
        let err = resolve_encoding("latin1").unwrap_err();
        assert!(matches!(err, EncodingError::Unsupported(ref n) if n == "latin1"));
    }

    /// 表記揺れ (大文字 / ハイフン違い) は受け付けない (ホワイトリスト方式)
    #[test]
    fn rejects_case_variations() {
        assert!(resolve_encoding("UTF-8").is_err());
        assert!(resolve_encoding("sjis").is_err());
        assert!(resolve_encoding("cp932").is_err());
    }

    /// Shift_JIS で書かれた CSV を DecodeReaderBytes 経由で読み、UTF-8 として出力できる
    #[test]
    fn reads_shift_jis_csv_through_decoder() {
        // Shift_JIS で "あ,b\n1,2\n" を表現したバイト列
        let input: &[u8] = &[
            0x82, 0xA0, // "あ"
            0x2C, // ","
            0x62, // "b"
            0x0A, // "\n"
            0x31, // "1"
            0x2C, // ","
            0x32, // "2"
            0x0A, // "\n"
        ];
        let encoding = resolve_encoding("shift_jis").unwrap();
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(Cursor::new(input));

        let columns = [ColumnSpec::Name("b".to_string())];
        let strategy = CharFill { ch: '*' };
        let mut output = Vec::new();
        mask_csv(
            decoded,
            &mut output,
            &MaskOptions {
                columns: &columns,
                delimiter: b',',
                strategy: &strategy,
                has_headers: true,
            },
        )
        .unwrap();
        // 出力は UTF-8 で "あ,b\n1,*\n"
        assert_eq!(String::from_utf8(output).unwrap(), "あ,b\n1,*\n");
    }
}
