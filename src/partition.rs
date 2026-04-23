use alloc::vec::Vec;

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

#[cfg(test)]
mod tests {
    use super::Partition;
    use alloc::vec::Vec;

    fn round_trip(source_length: usize, nb_source_symbols: usize) {
        let data: Vec<u8> = (0..source_length).map(|i| (i & 0xFF) as u8).collect();
        let p = Partition::new(source_length, nb_source_symbols);

        // Total elements must equal source length
        let total = p.nb_long * p.long_size + p.nb_small * p.small_size;
        assert_eq!(total, source_length, "partition total != source length");

        let symbols = p.create_source_block(&data);
        // Reassemble manually since decode_source_block expects Vec<Vec<u8>>
        let blocks: Vec<Vec<u8>> = symbols.iter().map(|s| s.data.to_vec()).collect();
        let reassembled = p.decode_source_block(&blocks);
        assert_eq!(reassembled, data, "round-trip failed");
    }

    #[test]
    fn test_partition_evenly_divisible() {
        // 100 / 10 = 10, so all symbols should be the same size (no "long")
        let p = Partition::new(100, 10);
        assert_eq!(p.long_size, 0);
        assert_eq!(p.nb_long, 0);
        assert_eq!(p.small_size, 10);
        assert_eq!(p.nb_small, 10);
        round_trip(100, 10);
    }

    #[test]
    fn test_partition_uneven() {
        // 103 / 10 -> 3 long of size 11 + 7 small of size 10 (total 33+70=103)
        let p = Partition::new(103, 10);
        assert_eq!(p.long_size * p.nb_long + p.small_size * p.nb_small, 103);
        round_trip(103, 10);
    }

    #[test]
    fn test_partition_single_symbol() {
        round_trip(42, 1);
    }

    #[test]
    fn test_partition_smaller_than_nb_symbols() {
        // 5 bytes split into 10 symbols -> some symbols are 1 byte, others 0
        round_trip(5, 10);
    }

    #[test]
    fn test_partition_large_round_trip() {
        round_trip(64 * 1024, 64);
        round_trip(100_000, 73);
    }
}
