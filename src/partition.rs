use crate::encodingsymbols::EncodingSymbol;

/// Partitions a block into semi-equal pieces of symbols.
pub struct Partition {
    pub long_size: usize,
    pub nb_long: usize,
    pub small_size: usize,
    pub nb_small: usize,
}

impl Partition {
    /// Partitions a block into semi-equal pieces of symbols.
    ///
    /// # Parameters
    ///
    /// * `length`: The number to be partitioned.
    /// * `nb_source_symbols`: The number of pieces the number should be
    ///   partitioned into.
    ///
    /// This function follows the block partitioning algorithm specified in RFC
    /// 5053 section 5.3.1.2. It divides the number `i` into `j` semi-equal
    /// pieces and returns the sizes of the longer and shorter pieces,
    /// as well as the number of longer and shorter pieces.
    pub fn new(source_length: usize, nb_source_symbols: usize) -> Self {
        let mut il = (source_length as f64 / nb_source_symbols as f64).ceil() as usize;
        let mut is = (source_length as f64 / nb_source_symbols as f64).floor() as usize;
        let jl = source_length - (is * nb_source_symbols);
        let js = nb_source_symbols - jl;

        if jl == 0 {
            il = 0
        }
        if js == 0 {
            is = 0
        }

        Partition {
            long_size: il,
            nb_long: jl,
            small_size: is,
            nb_small: js,
        }
    }

    pub fn create_source_block<'a>(&self, source_data: &'a [u8]) -> Vec<EncodingSymbol<'a>> {
        let mut start: usize = 0;
        let mut output: Vec<EncodingSymbol> = Vec::new();
        let mut esi = 0;

        for _ in 0..self.nb_long {
            let end = start + self.long_size;
            output.push(EncodingSymbol::new(&source_data[start..end], esi));
            start += self.long_size;
            esi += 1;
        }

        for _ in 0..self.nb_small {
            let end = start + self.small_size;
            output.push(EncodingSymbol::new(&source_data[start..end], esi));
            start += self.small_size;
            esi += 1;

            if start >= source_data.len() {
                break;
            }
        }

        output
    }

    pub fn decode_source_block(&self, source_block: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();

        assert!(self.nb_long + self.nb_small == source_block.len());
        for i in 0..self.nb_long {
            out.extend(source_block[i][0..self.long_size].to_vec());
        }
        for i in 0..self.nb_small {
            out.extend(source_block[self.nb_long + i][0..self.small_size].to_vec());
        }
        out
    }
}
