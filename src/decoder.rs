use crate::{encodingsymbols::EncodingSymbol, raptor};

///
/// A struct that represents a source block decoder that uses Raptor codes.
pub struct SourceBlockDecoder {
    raptor: raptor::Raptor,
}

impl SourceBlockDecoder {
    /// Create a new decoder
    ///
    /// # Arguments
    ///
    /// * `nb_source_symbols` - Number of source symbols in the block
    ///
    /// # Returns
    ///
    /// * A new `SourceBlockDecoder` instance
    pub fn new(nb_source_symbols: usize) -> Self {
        SourceBlockDecoder {
            raptor: raptor::Raptor::new(nb_source_symbols as u32),
        }
    }

    /// Push an encoding symbol to the decoder
    ///
    /// # Arguments
    ///
    /// * `encoding_symbol` - A slice of u8 numbers representing the encoding symbol data
    /// * `esi` - Encoding symbol identifier (ESI)
    pub fn push_encoding_symbol(&mut self, encoding_symbol: &[u8], esi: u32) {
        let encoding_symbol = EncodingSymbol::new(encoding_symbol, esi);
        self.raptor.add_encoding_symbol(&encoding_symbol);
    }

    /// Return true when the block can be fully decoded
    pub fn fully_specified(&self) -> bool {
        self.raptor.fully_specified()
    }

    /// Decode the source block
    ///
    ///
    /// # Parameters
    ///
    /// * `source_block_length`: The size of the source block in bytes.
    /// * `encoding_symbol_size`: Size of an encoding symbol
    ///
    /// # Returns
    ///
    /// * `None` if the source block cannot be decoded
    /// * `Some(Vec<u8>)` if the block is decoded. The vector contains the decoded source block data
    pub fn decode(&mut self, source_block_length: usize) -> Option<Vec<u8>> {
        self.raptor.decode(source_block_length)
    }
}

/// Decodes a source block from a given set of available encoding symbols.
///
/// # Parameters
///
/// * `encoding_symbols`: A list of available encoding symbols. Missing encoding symbols should be represented as `None`.
/// * `nb_source_symbols`: The number of source symbols in the block (k).
/// * `source_block_length`: The size of the source block in bytes.
///
/// # Returns
///
/// A vector of bytes representing the decoded source block, or `None` if the source block cannot be decoded.
/// The function uses the available encoding symbols to reconstruct the original source block.
///
pub fn decode_source_block(
    encoding_symbols: &[Option<Vec<u8>>],
    nb_source_symbols: usize,
    source_block_length: usize,
) -> Option<Vec<u8>> {
    let encoding_symbols = EncodingSymbol::from_option_block(encoding_symbols);
    let mut raptor = raptor::Raptor::new(nb_source_symbols as u32);
    raptor.add_encoding_symbols(&encoding_symbols);
    raptor.decode(source_block_length)
}
