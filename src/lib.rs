//! A Rust library for implementing Forward Error Correction (FEC) using Raptor codes.
//!
//! Raptor codes are a class of FEC codes that are designed to be highly efficient in the presence of packet erasures.
//! This library provides functionality for encoding source blocks into encoding symbols and decoding source blocks from a set of encoding symbols.
//!
//! # Examples
//!
//! ```
//!
//! let source_data: Vec<u8> = vec![1,2,3,4,5,6,7,8,9,10,11,12];
//! let encoding_symbol_length = 3;
//! let source_block:Vec<Vec<u8>> = source_data.chunks(encoding_symbol_length)
//!                                     .map(|source_symbol| source_symbol.to_vec()).collect();
//! let nb_repair = 3;
//!
//! 
//! // Step 1 - Generate the encoding symbols (source symbols + repair symbols)
//! let encoding_symbols = raptor::encode_source_block(&source_block, nb_repair);
//!
//! // Step 2 - Re-construct the source data from the encoding symbols
//! 
//! let nb_source_symbols = source_block.len();
//! let source_block_length = source_data.len();
//! 
//! let mut received_symbols: Vec<Option<Vec<u8>>> = encoding_symbols.into_iter().map(|symbols| Some(symbols)).collect();
//! received_symbols[0] = None; // simulate encoding symbol ESI=0 lost
//! 
//! let reconstructed_data = raptor::decode_source_block(&received_symbols, nb_source_symbols, source_block_length).unwrap();
//! 
//! // Source data and decoded data should be identical
//! assert!(reconstructed_data == source_data)
//!

mod common;
mod decoder;
mod encoder;
mod encodingsymbols;
mod raptor;
mod sparse_matrix;
mod tables;

pub use decoder::decode_source_block;
pub use encoder::encode_source_block;
pub use encoder::SourceBlockEncoder;

#[cfg(test)]
mod tests {
    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::builder().is_test(true).try_init().ok();
    }
}
