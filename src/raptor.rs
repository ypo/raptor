use alloc::vec;
use alloc::vec::Vec;

use crate::common;
use crate::encodingsymbols::EncodingSymbol;
use crate::partition::Partition;
use crate::sparse_matrix::SparseMatrix;

pub struct Raptor {
    k: u32,
    l: u32,
    l_prime: u32,
    matrix: SparseMatrix,
}

impl Raptor {
    pub fn new(k: u32) -> Self {
        let (l, l_prime, s, h, hp) = common::intermediate_symbols(k);
        let mut matrix = SparseMatrix::new(l as usize);

        #[rustfmt::skip]
        // Generate the matrix A
        /*
          K               S       H
          +-----------------------+-------+-------+
          |                       |       |       |
        S |        G_LDPC         |  I_S  | 0_SxH |
          |                       |       |       |
          +-----------------------+-------+-------+
          |                               |       |
        H |        G_Half                 |  I_H  |
          |                               |       |
          +-------------------------------+-------+
          |                                       |
          |                                       |
        K |                 G_LT                  |
          |                                       |
          |                                       |
          +---------------------------------------+
          */

        // G_LDPC
        let mut composition: Vec<Vec<u32>> = vec![Vec::new(); s as usize];
        for i in 0..k {
            let a = 1 + (i as f64 / s as f64).floor() as u32 % (s - 1);
            let b = i % s;
            composition[b as usize].push(i);
            let b = (b + a) % s;
            composition[b as usize].push(i);
            let b = (b + a) % s;
            composition[b as usize].push(i);
        }

        for i in 0..s {
            // Push I_S
            composition[i as usize].push(k + i);
            matrix.add_equation(composition[i as usize].clone(), Vec::new());
        }

        // H Half symbols
        let mut compositions: Vec<Vec<u32>> = vec![Vec::new(); h as usize];
        let m = common::gray_sequence(k as usize + s as usize, hp);
        for i in 0..h {
            for j in 0..k + s {
                if common::bit_set(m[j as usize], i) {
                    compositions[i as usize].push(j);
                }
            }
            compositions[i as usize].push(k + s + i);
            matrix.add_equation(compositions[i as usize].clone(), Vec::new())
        }

        let raptor = Raptor {
            k,
            l,
            l_prime,
            matrix,
        };

        raptor
    }

    pub fn get_l(&self) -> u32 {
        self.l
    }

    pub fn get_l_prime(&self) -> u32 {
        self.l_prime
    }

    pub fn add_encoding_symbol(&mut self, encoding_symbol: &EncodingSymbol) {
        let indices = common::find_lt_indices(self.k, encoding_symbol.esi, self.l, self.l_prime);
        self.matrix
            .add_equation(indices, encoding_symbol.data.to_vec());
    }

    pub fn add_encoding_symbols(&mut self, encoding_symbols: &[EncodingSymbol]) -> bool {
        for symbols in encoding_symbols {
            self.add_encoding_symbol(symbols);
        }
        self.matrix.fully_specified()
    }

    pub fn reduce(&mut self) {
        self.matrix.reduce()
    }

    pub fn intermediate_symbols(&self) -> &[Vec<u8>] {
        &self.matrix.intermediate
    }

    pub fn decode(&mut self, size: usize) -> Option<Vec<u8>> {
        if !self.matrix.fully_specified() {
            return None;
        }

        self.reduce();

        let mut source_block: Vec<Vec<u8>> = Vec::new();
        for i in 0..self.k {
            let block =
                common::lt_encode(self.k, i, self.l, self.l_prime, &self.matrix.intermediate);
            source_block.push(block);
        }

        let partition = Partition::new(size, self.k as usize);
        Some(partition.decode_source_block(&source_block))
    }

    pub fn fully_specified(&self) -> bool {
        self.matrix.fully_specified()
    }
}

#[cfg(test)]
mod tests {

    // Unit test from gofountain project
    // https://github.com/google/gofountain

    use alloc::vec;
    use alloc::vec::Vec;

    use crate::partition::Partition;

    #[test]
    fn test_raptor_matrix() {
        crate::tests::init();
        let raptor = super::Raptor::new(10);
        assert!(raptor.matrix.coeff[0] == vec![0, 5, 6, 7, 10]);
        assert!(raptor.matrix.coeff[1] == vec![1, 2, 3, 8, 13]);
        assert!(raptor.matrix.coeff[2] == vec![2, 3, 4, 7, 9, 14]);
    }

    #[test]
    fn test_raptor() {
        crate::tests::init();

        let input: Vec<u8> = vec![1, 2, 7, 4, 0, 2, 54, 4, 1, 1, 10, 200, 1, 21, 3, 80];
        let partition = Partition::new(input.len(), 4);
        let encoding_symbols = partition.create_source_block(&input);

        let mut raptor = super::Raptor::new(encoding_symbols.len() as u32);
        raptor.add_encoding_symbols(&encoding_symbols);

        assert!(raptor.fully_specified());

        let out = raptor.decode(input.len()).unwrap();

        log::debug!("{:?} / {:?}", out, input);
        assert!(out.len() == input.len());
        assert!(out == input);
    }

    #[test]
    fn test_decode_empty() {
        let mut raptor = super::Raptor::new(64);
        assert!(raptor.fully_specified() == false);
        let out = raptor.decode(1024);
        assert!(out.is_none());
    }
}
