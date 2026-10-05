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
    /// Coefficient rows, stored as bitsets of `words` u64 words.
    /// Bit `c` of row `i` is set when the intermediate symbol `c` is part of
    /// the XOR equation stored at row `i`.
    /// A non-empty row `i` always has its leftmost one at column `i`.
    /// An empty row means that no equation has its leftmost one at `i` yet.
    ///
    /// | 1 0 1 1 |          [ 0b1101,
    /// | 0 1 0 1 |            0b1010,
    /// | 0 0 0 0 | -> coeff   [],
    /// | 0 0 0 1 |            0b1000 ]
    coeff: Vec<Vec<u64>>,

    /// Number of ones of each coefficient row
    weight: Vec<u32>,

    /// Number of non-empty coefficient rows
    nb_rows: usize,

    /// Number of u64 words of a coefficient row
    words: usize,

    /// Intermediate symbols
    pub intermediate: Vec<Vec<u8>>,
}

/// Returns the index of the first one of `row` at or after column `from`
fn first_one(row: &[u64], from: usize) -> Option<usize> {
    let mut w = from / 64;
    let mut word = *row.get(w)? & (!0u64 << (from % 64));
    loop {
        if word != 0 {
            return Some(w * 64 + word.trailing_zeros() as usize);
        }
        w += 1;
        word = *row.get(w)?;
    }
}

/// `row ^= other`, returns the number of ones of the resulting row
///
/// On x86, the POPCNT instruction is selected at runtime when the CPU supports
/// it.
fn xor_row(row: &mut [u64], other: &[u64]) -> u32 {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_env = "sgx")
    ))]
    if common::cpu::has_popcnt() {
        // Safety: the CPU supports POPCNT
        return unsafe { xor_row_popcnt(row, other) };
    }

    xor_row_u64(row, other)
}

/// [`xor_row`] using the POPCNT instruction
///
/// # Safety
///
/// The CPU must support POPCNT
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_env = "sgx")
))]
#[target_feature(enable = "popcnt")]
unsafe fn xor_row_popcnt(row: &mut [u64], other: &[u64]) -> u32 {
    xor_row_u64(row, other)
}

#[inline(always)]
fn xor_row_u64(row: &mut [u64], other: &[u64]) -> u32 {
    let mut weight = 0;
    for (a, b) in row.iter_mut().zip(other) {
        *a ^= *b;
        weight += a.count_ones();
    }
    weight
}

impl SparseMatrix {
    pub fn new(l: usize) -> Self {
        SparseMatrix {
            coeff: vec![Vec::new(); l],
            weight: vec![0; l],
            nb_rows: 0,
            words: (l + 63) / 64,
            intermediate: vec![Vec::new(); l],
        }
    }

    /// On the fly Gaussian  Elimination (OFG)
    ///
    /// Add an XOR equation to the sparse matrix
    ///
    /// # Arguments
    ///
    /// * `components` - A vector of u32 numbers representing the indices of the
    ///   source blocks, sorted in strictly increasing order
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
        debug_assert!(components.windows(2).all(|w| w[0] < w[1]));
        debug_assert!(components.iter().all(|&c| (c as usize) < self.coeff.len()));

        let mut row = vec![0u64; self.words];
        for &c in &components {
            row[c as usize / 64] |= 1u64 << (c % 64);
        }
        let mut weight: u32 = row.iter().map(|w| w.count_ones()).sum();
        let mut b = b;

        // s <- LeftmostOne
        let mut s = match first_one(&row, 0) {
            Some(s) => s,
            None => return,
        };

        // while EqOnes > 0 and G[s][s] = 1 do
        while !self.coeff[s].is_empty() {
            // if EqOnes ≥ NumOnes[s] then
            if weight >= self.weight[s] {
                // NewEq <- NewEq ^ G[s]
                // (both rows have no ones before column s)
                let w = s / 64;
                weight = xor_row(&mut row[w..], &self.coeff[s][w..]);
                // NewY <- NewY ^ Y [s]
                common::xor(&mut b, &self.intermediate[s]);
                // s <- LeftmostOne, drop the equation when it becomes empty
                s = match first_one(&row, s + 1) {
                    Some(s) => s,
                    None => return,
                };
            } else {
                // Swap matrix row with the new row
                // (the new row also has its leftmost one at s)
                core::mem::swap(&mut self.coeff[s], &mut row);
                core::mem::swap(&mut self.weight[s], &mut weight);
                core::mem::swap(&mut self.intermediate[s], &mut b);
            }
        }

        // G[s] <- NewEq
        self.coeff[s] = row;
        self.weight[s] = weight;
        // Y [s] <- NewY
        self.intermediate[s] = b;
        self.nb_rows += 1;
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        self.nb_rows == self.coeff.len()
    }

    /// Gaussian Elimination.  
    /// Algo from from gofountain project
    /// https://github.com/google/gofountain
    ///
    /// Rows are reduced from the last one, so the symbols `c > i` of row `i`
    /// are already solved when row `i` is reduced. Each row receives the same
    /// XORs as in the gofountain algorithm, so the intermediate symbols
    /// (values and lengths) are identical.
    pub fn reduce(&mut self) {
        debug_assert!(self.fully_specified());
        for i in (0..self.coeff.len()).rev() {
            let (inter_i, inter_c) = self.intermediate.split_at_mut(i + 1);
            let row = &mut self.coeff[i];
            if row.is_empty() {
                continue;
            }
            // Walk the ones after the leftmost one and clear them,
            // the solved row only keeps its leftmost one
            let w0 = i / 64;
            let leftmost = 1u64 << (i % 64);
            row[w0] ^= leftmost;
            for (w, word) in row.iter_mut().enumerate().skip(w0) {
                let mut bits = core::mem::take(word);
                while bits != 0 {
                    let c = w * 64 + bits.trailing_zeros() as usize;
                    common::xor(&mut inter_i[i], &inter_c[c - i - 1]);
                    bits &= bits - 1;
                }
            }
            row[w0] = leftmost;
            self.weight[i] = 1;
        }
    }

    /// Indices of the ones of a coefficient row
    #[cfg(test)]
    pub fn row_indices(&self, i: usize) -> Vec<u32> {
        let mut indices = Vec::new();
        let mut from = 0;
        while let Some(c) = first_one(&self.coeff[i], from) {
            indices.push(c as u32);
            from = c + 1;
        }
        indices
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use super::SparseMatrix;
    use crate::common;

    /// Previous implementation (rows stored as sorted index lists), used as a
    /// reference: the bitset implementation must give exactly the same rows
    /// and the same intermediate symbols (values and lengths)
    struct RefMatrix {
        coeff: Vec<Vec<u32>>,
        intermediate: Vec<Vec<u8>>,
    }

    fn symmetric_difference(row_1: &mut Vec<u32>, row_2: &[u32]) {
        let mut result = Vec::with_capacity(row_1.len() + row_2.len());
        let mut i = 0;
        let mut j = 0;

        while i < row_1.len() && j < row_2.len() {
            use core::cmp::Ordering;
            match row_1[i].cmp(&row_2[j]) {
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                Ordering::Less => {
                    result.push(row_1[i]);
                    i += 1;
                }
                Ordering::Greater => {
                    result.push(row_2[j]);
                    j += 1;
                }
            }
        }

        result.extend_from_slice(&row_1[i..]);
        result.extend_from_slice(&row_2[j..]);
        *row_1 = result;
    }

    impl RefMatrix {
        fn new(l: usize) -> Self {
            RefMatrix {
                coeff: vec![Vec::new(); l],
                intermediate: vec![Vec::new(); l],
            }
        }

        fn add_equation(&mut self, components: Vec<u32>, b: Vec<u8>) {
            let mut components = components;
            let mut b = b;

            while !components.is_empty() && !self.coeff[components[0] as usize].is_empty() {
                let s = components[0] as usize;
                if components.len() >= self.coeff[s].len() {
                    symmetric_difference(&mut components, &self.coeff[s]);
                    common::xor(&mut b, &self.intermediate[s]);
                } else {
                    core::mem::swap(&mut self.coeff[s], &mut components);
                    core::mem::swap(&mut self.intermediate[s], &mut b);
                }
            }

            if !components.is_empty() {
                let s = components[0] as usize;
                self.coeff[s] = components;
                self.intermediate[s] = b;
            }
        }

        fn fully_specified(&self) -> bool {
            self.coeff.iter().all(|coeff| !coeff.is_empty())
        }

        fn reduce(&mut self) {
            let l = self.coeff.len();
            let mut reverse_index: Vec<Vec<usize>> = vec![Vec::new(); l];
            for (j, row) in self.coeff.iter().enumerate() {
                for &k in row {
                    reverse_index[k as usize].push(j);
                }
            }

            for i in (0..l).rev() {
                let first_coeff_i = self.coeff[i][0];
                let (inter_j, inter_i) = self.intermediate.split_at_mut(i);
                for &j in &reverse_index[first_coeff_i as usize] {
                    if j < i {
                        common::xor(&mut inter_j[j], &inter_i[0]);
                    }
                }
                self.coeff[i].resize(1, 0);
            }
        }
    }

    fn assert_same(matrix: &SparseMatrix, reference: &RefMatrix) {
        assert_eq!(matrix.coeff.len(), reference.coeff.len());
        assert_eq!(matrix.fully_specified(), reference.fully_specified());
        for (i, row) in reference.coeff.iter().enumerate() {
            assert_eq!(&matrix.row_indices(i), row, "row {}", i);
            assert_eq!(matrix.coeff[i].is_empty(), row.is_empty(), "row {}", i);
            assert_eq!(matrix.weight[i] as usize, row.len(), "row {}", i);
        }
        assert_eq!(matrix.intermediate, reference.intermediate);
    }

    /// Random equation, either sparse (like LT rows) or dense (like half
    /// symbol rows), with random data lengths
    fn random_equation(rng: &mut StdRng, l: usize) -> (Vec<u32>, Vec<u8>) {
        let degree = if rng.random_range(0..4) == 0 {
            rng.random_range(0..=l)
        } else {
            rng.random_range(1..=l.min(5))
        };
        let mut indices: Vec<u32> = (0..l as u32).collect();
        for i in 0..degree {
            let j = rng.random_range(i..l);
            indices.swap(i, j);
        }
        indices.truncate(degree);
        indices.sort_unstable();
        let len = rng.random_range(0..12);
        let data = (0..len).map(|_| rng.random()).collect();
        (indices, data)
    }

    #[test]
    fn test_same_as_reference_implementation() {
        crate::tests::init();
        let sizes = [1usize, 2, 3, 8, 63, 64, 65, 127, 128, 129, 200];
        for (seed, &l) in sizes.iter().enumerate() {
            let mut rng = StdRng::seed_from_u64(seed as u64);
            let mut matrix = SparseMatrix::new(l);
            let mut reference = RefMatrix::new(l);
            assert_same(&matrix, &reference);

            let mut nb_equations = 0;
            while !reference.fully_specified() {
                let (indices, data) = random_equation(&mut rng, l);
                matrix.add_equation(indices.clone(), data.clone());
                reference.add_equation(indices, data);
                assert_same(&matrix, &reference);
                nb_equations += 1;
                assert!(nb_equations < 100 * l, "l={} not fully specified", l);
            }

            matrix.reduce();
            reference.reduce();
            assert_same(&matrix, &reference);

            // Equations added after the reduction, then reduce again
            for _ in 0..l.min(20) {
                let (indices, data) = random_equation(&mut rng, l);
                matrix.add_equation(indices.clone(), data.clone());
                reference.add_equation(indices, data);
                assert_same(&matrix, &reference);
            }
            matrix.reduce();
            reference.reduce();
            assert_same(&matrix, &reference);
        }
    }
}
