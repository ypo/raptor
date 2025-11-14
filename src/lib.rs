//! [![Rust](https://github.com/ypo/raptor/actions/workflows/rust.yml/badge.svg)](https://github.com/ypo/raptor/actions/workflows/rust.yml)
//! [![codecov](https://codecov.io/gh/ypo/raptor/branch/main/graph/badge.svg?token=P4KE639YU8)](https://codecov.io/gh/ypo/raptor)
//! [![Crates.io](https://img.shields.io/crates/v/raptor-code)](https://crates.io/crates/raptor-code/)
//!
//! # Raptor Code
//!
//! A Rust library for implementing Forward Error Correction (FEC) using Raptor
//! codes.
//!
//! Raptor codes are a class of FEC codes that are designed to be highly
//! efficient in the presence of packet erasures. This library provides
//! functionality for encoding source blocks into encoding symbols and decoding
//! source blocks from a set of encoding symbols.
//!
//! This library implements on the fly Gaussian Elimination to spread  decoding
//! complexity during packets reception.
//!
//! # Example : Source Block Encoder/Decoder
//!
//! Encode and decode a source block using `raptor_code::encode_source_block`
//! and `raptor_code::decode_source_block`
//!
//!
//! ```
//! let source_block_data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
//! let max_source_symbols = 4;
//! let nb_repair = 3;
//! let source_block_length = source_block_data.len();
//!
//! // Step 1 - Generate the encoding symbols (source symbols + repair symbols)
//! let (encoding_symbols, nb_source_symbols) =
//!     raptor_code::encode_source_block(&source_block_data, max_source_symbols, nb_repair).unwrap();
//!
//! // Step 2 - Re-construct the source data from the encoding symbols
//! let mut received_symbols: Vec<Option<Vec<u8>>> = encoding_symbols
//!     .into_iter()
//!     .map(|symbols| Some(symbols))
//!     .collect();
//! // simulate encoding symbol lost
//! received_symbols[0] = None;
//!
//! let reconstructed_data = raptor_code::decode_source_block(
//!     &received_symbols,
//!     nb_source_symbols as usize,
//!     source_block_length,
//! )
//! .unwrap();
//!
//! // Source data and decoded data should be identical
//! assert!(reconstructed_data == source_block_data)
//! ```
//!
//! # Example : On the fly encoder
//!
//! ```
//! let source_data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
//! let max_source_symbols = 4;
//! let nb_repair = 3;
//!
//! let mut encoder = raptor_code::SourceBlockEncoder::new(&source_data, max_source_symbols).unwrap();
//! let n = encoder.nb_source_symbols() + nb_repair;
//!
//! for esi in 0..n as u32 {
//!     let encoding_symbol = encoder.fountain(esi);
//!     // TODO transfer symbol over Network
//!     // network_push_pkt(encoding_symbol);
//! }
//! ```
//! # Example : On the fly decoder
//!
//! ```
//! let encoding_symbol_length = 1024;
//! let nb_source_symbols = 4; // Number of source symbols in the source block
//! let source_block_length = encoding_symbol_length  * nb_source_symbols; // Total size size of the block;
//! let mut n = 0u32;
//! let mut decoder = raptor_code::SourceBlockDecoder::new(nb_source_symbols);
//!
//! while decoder.fully_specified() == false {
//!     //TODO replace the following line with pkt received from network
//!     let (encoding_symbol, esi) = (vec![0; encoding_symbol_length],n);
//!     decoder.push_encoding_symbol(&encoding_symbol, esi);
//!     n += 1;
//! }
//!
//! let source_block = decoder.decode(source_block_length as usize);
//! ```
//!
//! # Credit
//!
//! RFC 5053 <https://www.rfc-editor.org/rfc/rfc5053.html>  
//!
//! On the fly Gaussian Elimination for LT codes, Valerio Bioglio, Marco
//! Grangetto, 2009
//!
//! Reuse ideas and concepts of [gofountain](https://github.com/google/gofountain)
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

extern crate alloc;

#[cfg(any(test))]
extern crate std;

mod common;
mod decoder;
mod encoder;
mod encodingsymbols;
mod partition;
mod raptor;
mod sparse_matrix;
mod tables;

pub use decoder::{decode_source_block, SourceBlockDecoder};
pub use encoder::{encode_source_block, SourceBlockEncoder};

#[cfg(test)]
mod tests {
    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::builder().is_test(true).try_init().ok();
    }
}
