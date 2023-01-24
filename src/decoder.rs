use crate::{encodingsymbols::EncodingSymbol, raptor};

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
