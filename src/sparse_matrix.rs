use alloc::vec;
use alloc::vec::Vec;

use crate::common;
/// Sparce Matrix
///
/// Original implementation
/// https://github.com/google/gofountain/blob/master/block.go
///
/// A^block = intermediate
pub struct SparseMatrix {
    /// Indices of the source blocks which are xor-ed together
    /// | 0 0 1 1 |          [[ 2, 3],
    /// | 0 1 0 1 |           [ 1, 3 ],
    /// | 1 1 1 0 | -> coeff  [ 0, 1, 2],
    /// | 1 0 0 0 |           [ 0 ] ]
    pub coeff: Vec<Vec<u32>>,

    /// Intermediate symbols
    pub intermediate: Vec<Vec<u8>>,
}

impl SparseMatrix {
    pub fn new(l: usize) -> Self {
        SparseMatrix {
            coeff: vec![Vec::new(); l],
            intermediate: vec![vec![0; l]; l],
        }
    }

    /// On the fly Gaussian  Elimination (OFG)
    ///
    /// Add an XOR equation to the sparse matrix
    ///
    /// # Arguments
    ///
    /// * `components` - A vector of u32 numbers representing the indices of the
    ///   source blocks
    /// * `b` - A vector of u8 numbers representing the intermediate symbols
    ///
    /// variant of Valerio Bioglio, Marco Grangetto algorithm,
    /// On the fly Gaussian Elimination for LT codes, 2009
    ///
    /// OFG builds a triangular matrix G by exploiting every received packet
    /// starting from the very first one.
    ///
    /// Spreads decoding complexity during packets reception
    pub fn add_equation(&mut self, components: Vec<u32>, b: Vec<u8>) {
        let mut components = components;
        let mut b = b;

        // while EqOnes > 0 and G[s][s] = 1 do
        while components.len() > 0 && self.coeff[components[0] as usize].len() > 0 {
            // s <- LeftmostOne
            let s = components[0];
            // if EqOnes ≥ NumOnes[s] then
            if components.len() >= self.coeff[s as usize].len() {
                // NewEq <- NewEq ^ G[s]
                common::symmetric_difference(&mut components, &self.coeff[s as usize]);
                // NewY <- NewY ^ Y [s]
                common::xor(&mut b, &self.intermediate[s as usize]);
            } else {
                // Swap matrix row with the new row
                core::mem::swap(&mut self.coeff[s as usize], &mut components);
                core::mem::swap(&mut self.intermediate[s as usize], &mut b);
            }
        }

        // if EqOnes > 0 then
        if components.len() > 0 {
            let s = components[0] as usize;
            // G[s] <- NewEq
            self.coeff[s] = components;
            // Y [s] <- NewY
            self.intermediate[s] = b;
        }
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        self.coeff.iter().find(|coeff| coeff.is_empty()).is_none()
    }

    /// Gaussian Elimination.  
    /// Algo from from gofountain project
    /// https://github.com/google/gofountain
    pub fn reduce(&mut self) {
        for i in (0..self.coeff.len()).rev() {
            let (inter_j, inter_i) = self.intermediate.split_at_mut(i);
            let first_coeff_i = self.coeff[i][0];
            for j in 0..i {
                for k in &self.coeff[j] {
                    if *k == first_coeff_i {
                        common::xor(&mut inter_j[j], &inter_i[0]);
                    }
                }
            }
            self.coeff[i].resize(1, 0);
        }
    }
}
