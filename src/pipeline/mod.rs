// CSV 変換サブコマンドの共通パイプライン
//
// 入力 → デコード(stream) → csv 読込 → RecordTransform → csv 書出 →
// エンコード(stream) → 一時ファイル → rename という流れを 1 箇所に集約する。
// 各サブコマンドは RecordTransform を実装し、run_pipeline に渡すだけでよい。

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use csv::StringRecord;
use encoding_rs::Encoding;

use crate::error::{CsvOpsError, EncodingError};
use crate::io::decoding_reader::DecodingReader;
use crate::io::encoding_writer::EncodingWriter;

/// CSV を 1 行ずつ変換する処理の trait
/// run_pipeline がヘッダー → データ行の順に呼び出す。
/// 統計は実装側が内部に保持し、run_pipeline 呼び出し後に取り出す。
pub(crate) trait RecordTransform {
    /// 初期化フック (各実行で 1 度だけ呼ばれる)
    /// headers はヘッダー行 (ヘッダー無し設定なら None)。
    /// 列名解決や出力列の衝突検査をここで行う。
    /// 戻り値 Some(record) は出力ヘッダーとして書き出される (None なら書き出さない)。
    fn init(&mut self, headers: Option<&StringRecord>)
    -> Result<Option<StringRecord>, CsvOpsError>;

    /// データ行を in-place で変換する。row は 1-indexed。
    fn on_record(&mut self, record: &mut StringRecord, row: u64) -> Result<(), CsvOpsError>;
}

/// run_pipeline に渡す入出力設定
pub(crate) struct PipelineOptions {
    /// 入力ファイルパス
    pub input: PathBuf,
    /// 出力ファイルパス
    pub output: PathBuf,
    /// 入力エンコーディング
    pub input_encoding: &'static Encoding,
    /// 出力エンコーディング
    pub output_encoding: &'static Encoding,
    /// 入力区切り文字
    pub input_delimiter: u8,
    /// 出力区切り文字
    pub output_delimiter: u8,
    /// ヘッダーー行の有無
    pub has_headers: bool,
    /// true なら出力ファイルへ書き込まず、変換と統計集計だけ行う
    pub dry_run: bool,
}

/// パイプラインを実行し、処理した行数 (ヘッダー除く) を返す
pub(crate) fn run_pipeline<T: RecordTransform>(
    transform: &mut T,
    opts: &PipelineOptions,
) -> Result<u64, CsvOpsError> {
    // 入力の行終端を検出し、出力でも同じものを使う
    let crlf = detect_crlf(&opts.input)?;

    // 入力をストリーミングでデコードしながら読む (不正バイトは DecodingReader が弾く)
    let file = File::open(&opts.input)?;
    let decoded = DecodingReader::new(file, opts.input_encoding);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(opts.input_delimiter)
        .has_headers(opts.has_headers)
        .flexible(true)
        .from_reader(BufReader::new(decoded));

    // ヘッダーを取得し、transform を初期化する
    // init は出力ファイルを作る前に呼ぶので、列解決エラー時に空ファイルを残さない
    let headers: Option<StringRecord> = if opts.has_headers {
        Some(rdr.headers()?.clone())
    } else {
        None
    };
    let out_header = transform.init(headers.as_ref())?;

    // dry-run は出力先を持たず、変換と統計集計だけ行う
    if opts.dry_run {
        let mut row = 0u64;
        for result in rdr.records() {
            row += 1;
            let mut record = result?;
            transform.on_record(&mut record, row)?;
        }
        return Ok(row);
    }

    // 一時ファイルへ書き、成功時のみ rename する。
    // これにより出力はアトミックになり、エンコードエラー等で中断しても
    // 不完全なファイルが出力先に残らない。
    let temp = temp_path_for(&opts.output);
    let rows = write_to_temp(transform, opts, &mut rdr, out_header.as_ref(), crlf, &temp)
        .inspect_err(|_| {
            let _ = fs::remove_file(&temp);
        })?;
    fs::rename(&temp, &opts.output)?;
    Ok(rows)
}

/// 一時ファイルへ変換結果を書き出し、処理行数を返す
fn write_to_temp<T: RecordTransform, R: Read>(
    transform: &mut T,
    opts: &PipelineOptions,
    rdr: &mut csv::Reader<R>,
    out_header: Option<&StringRecord>,
    crlf: bool,
    temp: &Path,
) -> Result<u64, CsvOpsError> {
    let file = File::create(temp)?;
    let enc_writer = EncodingWriter::new(BufWriter::new(file), opts.output_encoding);
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(opts.output_delimiter)
        .flexible(true)
        .terminator(if crlf {
            csv::Terminator::CRLF
        } else {
            csv::Terminator::Any(b'\n')
        })
        .from_writer(enc_writer);

    // ヘッダー行 (transform が返したもの) を先に書く
    if let Some(h) = out_header {
        wtr.write_record(h)?;
    }

    // 各データ行を変換して書き出す
    let mut row = 0u64;
    for result in rdr.records() {
        row += 1;
        let mut record = result?;
        transform.on_record(&mut record, row)?;
        wtr.write_record(&record)?;
    }
    wtr.flush()?;

    // csv::Writer を分解して EncodingWriter を取り出し、エンコードエラーを確認する。
    // エラーがあれば呼び出し側で一時ファイルが削除されるため、不正な出力は残らない。
    let enc_writer = wtr.into_inner().map_err(|e| e.into_error())?;
    if enc_writer.had_errors() {
        return Err(EncodingError::EncodeFailure {
            encoding: opts.output_encoding.name().to_string(),
        }
        .into());
    }
    Ok(row)
}

/// 出力パスに対応する一時ファイルパスを作る
/// 出力先と同じディレクトリに置くことで rename が同一ファイルシステム内で完結する
fn temp_path_for(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_owned();
    name.push(format!(".csv-ops-{}.tmp", std::process::id()));
    PathBuf::from(name)
}

/// 入力ファイル先頭を読み、最初の改行が CRLF かどうかを返す
/// \r \n は ASCII 制御文字で SJIS / EUC-JP / UTF-8 のどの多バイト列にも紛れないため、
/// デコード前の生バイトのまま走査して安全
fn detect_crlf(path: &Path) -> Result<bool, CsvOpsError> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 65536];
    let n = file.read(&mut buf)?;
    Ok(first_newline_is_crlf(&buf[..n]))
}

/// バイト列の最初の改行が CRLF かどうか
fn first_newline_is_crlf(bytes: &[u8]) -> bool {
    match bytes.iter().position(|&b| b == b'\n') {
        Some(i) => i > 0 && bytes[i - 1] == b'\r',
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_detected() {
        assert!(first_newline_is_crlf(b"a,b\r\n1,2\r\n"));
    }

    #[test]
    fn lf_only_is_not_crlf() {
        assert!(!first_newline_is_crlf(b"a,b\n1,2\n"));
    }

    #[test]
    fn no_newline_is_not_crlf() {
        assert!(!first_newline_is_crlf(b"a,b"));
    }

    #[test]
    fn temp_path_is_sibling_of_output() {
        let temp = temp_path_for(Path::new("/tmp/out.csv"));
        assert_eq!(temp.parent(), Path::new("/tmp/out.csv").parent());
        assert_ne!(temp, PathBuf::from("/tmp/out.csv"));
    }

    /// 既存フィールドを素通しするだけの transform (テスト用)
    struct Identity;
    impl RecordTransform for Identity {
        fn init(
            &mut self,
            headers: Option<&StringRecord>,
        ) -> Result<Option<StringRecord>, CsvOpsError> {
            Ok(headers.cloned())
        }
        fn on_record(&mut self, _record: &mut StringRecord, _row: u64) -> Result<(), CsvOpsError> {
            Ok(())
        }
    }

    #[test]
    fn pipeline_round_trips_and_writes_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.csv");
        let output = dir.path().join("out.csv");
        std::fs::write(&input, "名前,年齢\n田中,30\n").unwrap();

        let opts = PipelineOptions {
            input,
            output: output.clone(),
            input_encoding: encoding_rs::UTF_8,
            output_encoding: encoding_rs::UTF_8,
            input_delimiter: b',',
            output_delimiter: b',',
            has_headers: true,
            dry_run: false,
        };
        let rows = run_pipeline(&mut Identity, &opts).unwrap();

        assert_eq!(rows, 1);
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "名前,年齢\n田中,30\n"
        );
        // 一時ファイルが残っていないこと
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn pipeline_dry_run_writes_no_output() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.csv");
        let output = dir.path().join("out.csv");
        std::fs::write(&input, "a,b\n1,2\n").unwrap();

        let opts = PipelineOptions {
            input,
            output: output.clone(),
            input_encoding: encoding_rs::UTF_8,
            output_encoding: encoding_rs::UTF_8,
            input_delimiter: b',',
            output_delimiter: b',',
            has_headers: true,
            dry_run: true,
        };
        let rows = run_pipeline(&mut Identity, &opts).unwrap();

        assert_eq!(rows, 1);
        assert!(!output.exists());
    }
}
