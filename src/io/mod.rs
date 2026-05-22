// 入出力共通モジュール
pub mod decoding_reader;
pub mod encoding;
pub mod encoding_writer;

pub use encoding::{detect_encoding, resolve_encoding};
