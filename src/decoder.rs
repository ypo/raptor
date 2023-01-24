use crate::{encodingsymbols::EncodingSymbol, raptor};

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
