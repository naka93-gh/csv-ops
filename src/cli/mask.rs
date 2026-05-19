use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use csv_ops::{CharFill, MaskConfig, MaskOptions, Target, mask_csv, resolve_encoding};

/// `csv-ops mask` の引数
#[derive(Args, Debug)]
pub(crate) struct MaskArgs {
    /// 設定ファイル (TOML)
    #[arg(short = 'c', long, value_name = "FILE", default_value = "config.toml")]
    pub config: PathBuf,
}

/// mask サブコマンドのエントリポイント
pub(crate) fn run(args: MaskArgs) -> Result<ExitCode, Box<dyn Error>> {
    let toml_str = std::fs::read_to_string(&args.config).map_err(|e| {
        format!(
            "設定ファイルを開けません ({}): {}",
            args.config.display(),
            e
        )
    })?;
    let config: MaskConfig = toml::from_str(&toml_str)?;

    for target in &config.targets {
        process_target(target)?;
    }

    println!("Masking complete.");
    Ok(ExitCode::SUCCESS)
}

/// 1 ファイル単位のマスキング処理
fn process_target(target: &Target) -> Result<(), Box<dyn Error>> {
    let delimiter = target.delimiter.as_bytes().first().copied().unwrap_or(b',');
    let input_path = Path::new(&target.filepath);
    let output_path = masked_path(input_path);

    println!("Processing: {}", input_path.display());

    // 入力エンコーディングを解決し、DecodeReaderBytes で UTF-8 にデコードしながら読むラッパを作る
    let encoding = resolve_encoding(&target.encoding)?;
    let file = File::open(input_path)?;
    let decoded = encoding_rs_io::DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .build(file);
    let reader = BufReader::new(decoded);

    // マスキング結果は一旦 UTF-8 で Vec<u8> に書き出し、後で入力と同じエンコーディングに変換する
    // TODO: EncodingWriter (Write ラッパ) を実装してストリーミング化する
    let mut utf8_buf = Vec::new();
    let strategy = CharFill { ch: '*' };
    let options = MaskOptions {
        columns: &target.columns,
        delimiter,
        strategy: &strategy,
        has_headers: target.has_headers,
    };
    mask_csv(reader, &mut utf8_buf, &options)
        .map_err(|e| format!("{} の処理中にエラー: {}", input_path.display(), e))?;

    // UTF-8 のマスク結果を入力と同じエンコーディングに変換して書き出す
    let utf8_str =
        std::str::from_utf8(&utf8_buf).map_err(|e| format!("内部 UTF-8 表現に異常: {}", e))?;
    let (encoded, _, had_errors) = encoding.encode(utf8_str);
    if had_errors {
        return Err(format!(
            "{} に {} でエンコードできない文字があります",
            output_path.display(),
            encoding.name()
        )
        .into());
    }
    let mut writer = BufWriter::new(File::create(&output_path)?);
    writer.write_all(&encoded)?;
    writer.flush()?;
    Ok(())
}

/// マスク済みファイルの出力先を生成する
/// `path/to/foo.csv` → `path/to/foo_masked.csv`
fn masked_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("csv");
    let parent = input.parent().unwrap_or(Path::new(""));
    parent.join(format!("{}_masked.{}", stem, ext))
}
