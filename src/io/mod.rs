// 入出力共通モジュール
pub mod decoding_reader;
pub mod encoding;
pub mod encoding_writer;
pub mod line_ending;

pub use encoding::{detect_encoding, resolve_encoding, resolve_input_encoding};
pub use line_ending::{LineEnding, analyze_line_ending};
