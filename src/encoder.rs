use crate::common;
use crate::encodingsymbols::EncodingSymbol;
use crate::raptor;

/// A struct that represents a source block encoder that uses Raptor codes.
pub struct SourceBlockEncoder {
    intermediate: Vec<Vec<u8>>,
    k: u32,
    l: u32,
    l_prime: u32,
}

impl SourceBlockEncoder {
    /// Create a source block encoder, passing the list of source symbols
    ///
    /// # Parameters
    ///
    /// * `source_block`: A slice of vectors containing the source symbols.
    ///
    /// # Returns
    ///
    /// A new `SourceBlockEncoder` instance.
    pub fn new(source_block: &[Vec<u8>]) -> Self {
        let k = source_block.len() as u32;
        let mut decoder = raptor::Raptor::new(k);
        let source_block = EncodingSymbol::from_block(source_block);
        decoder.add_encoding_symbols(&source_block);
        decoder.reduce();

        SourceBlockEncoder {
            intermediate: decoder.intermediate_symbols().to_vec(),
            k: k,
            l: decoder.get_l(),
            l_prime: decoder.get_l_prime(),
        }
    }

    /// Generates an encoding symbol with the specified Encoding Symbol Identifier (ESI).
    ///
    /// This method generates a encoding symbol using the Raptor code and the intermediate symbols generated during the initialization of the encoder.
    ///
    /// # Parameters
    ///
    /// * `esi`: The Encoding Symbol Identifier (ESI) of the desired encoding symbol.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Vec<u8>` : The generated encoding symbol
    pub fn fountain(&mut self, esi: u32) -> Vec<u8> {
        let mut block = Vec::new();
        let indices = common::find_lt_indices(self.k, esi, self.l, self.l_prime);
        for indice in indices {
            if indice < self.intermediate.len() as u32 {
                common::xor(&mut block, &self.intermediate[indice as usize]);
            }
        }

        block
    }
}

///
/// Encodes a source block into encoding symbols using Raptor codes.
///
/// # Parameters
///
/// * `source_block`: A slice of vectors containing the source symbols.
/// * `nb_repair`: The number of repair symbols to be generated.
///
/// # Returns
///
/// A vector of vectors of bytes representing the encoding symbols (source symbols + repair symbol).
/// The function uses Raptor codes to generate the specified number of repair symbols from the source block.
///
pub fn encode_source_block(source_block: &[Vec<u8>], nb_repair: usize) -> Vec<Vec<u8>> {
    let mut encoder = SourceBlockEncoder::new(&source_block);
    let mut output: Vec<Vec<u8>> = Vec::new();
    let n = source_block.len() + nb_repair;
    for esi in 0..n as u32 {
        output.push(encoder.fountain(esi));
    }
    output
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_source_block_encoder() {
        crate::tests::init();

        let blocks = vec![
            vec![1, 2, 7, 4],
            vec![0, 2, 54, 4],
            vec![1, 1, 10, 200],
            vec![1, 21, 3, 80],
        ];

        let nb_repair = 3;
        let encoded_block = super::encode_source_block(&blocks, nb_repair);
        let mut encoded_block: Vec<Option<Vec<u8>>> = encoded_block
            .into_iter()
            .map(|symbols| Some(symbols))
            .collect();

        let mut count: usize = 0;
        let mut expected_output: Vec<u8> = Vec::new();
        for b in &blocks {
            count += b.len();
            expected_output.extend(b)
        }

        // Simulate loss
        encoded_block[0] = None;
        encoded_block[1] = None;

        let output =
            crate::decoder::decode_source_block(&encoded_block, blocks.len(), count).unwrap();
        log::debug!("{:?} / {:?}", output, expected_output);
        assert!(output.len() == count);
        assert!(output == expected_output);
    }
}
