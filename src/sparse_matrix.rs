use crate::common;

/// Sparce Matrix
///
/// Original implementation
/// https://github.com/google/gofountain/blob/master/block.go
///
pub struct SparseMatrix {
    // Indices of the source blocks which are xor-ed together
    // | 0 0 1 1 |          [[ 2, 3],
    // | 0 1 0 1 |           [ 1, 3 ],
    // | 1 1 1 0 | -> coeff  [ 0, 1, 2],
    // | 1 0 0 0 |           [ 0 ] ]
    pub coeff: Vec<Vec<u32>>,

    // Intermediate symbols
    pub intermediate: Vec<Vec<u8>>,
}

impl SparseMatrix {
    pub fn new(l: usize) -> Self {
        SparseMatrix {
            coeff: vec![Vec::new(); l],
            intermediate: vec![vec![0; l]; l],
        }
    }

    ///
    /// algo from gofountain project
    /// https://github.com/google/gofountain
    ///
    pub fn add_equation(&mut self, components: Vec<u32>, b: Vec<u8>) {
        let mut components = components;
        let mut b = b;
        while components.len() > 0 && self.coeff[components[0] as usize].len() > 0 {
            let s = components[0];
            if components.len() >= self.coeff[s as usize].len() {
                common::xor(&mut b, &self.intermediate[s as usize]);
                components = common::disjunctive_union(&self.coeff[s as usize], &components);
            } else {
                // Swap matrix row with the new row
                (self.coeff[s as usize], components) = (components, self.coeff[s as usize].clone());
                (self.intermediate[s as usize], b) = (b, self.intermediate[s as usize].clone());
            }
        }

        if components.len() > 0 {
            self.coeff[components[0] as usize] = components.clone();
            self.intermediate[components[0] as usize] = b.clone();
        }
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        self.coeff.iter().find(|coeff| coeff.is_empty()).is_none()
    }

    /// xor of 2 intermediate rows
    fn intermediate_xor(&mut self, row_1: usize, row_2: usize) {
        let l_2 = self.intermediate[row_2].len();
        if self.intermediate[row_1].len() < l_2 {
            self.intermediate[row_1].resize(l_2, 0);
        }
        for k in 0..l_2 {
            self.intermediate[row_1][k] ^= self.intermediate[row_2][k];
        }
    }

    /// Gaussian Elimination.  
    /// Algo from from gofountain project
    /// https://github.com/google/gofountain
    pub fn reduce(&mut self) {
        for i in (0..self.coeff.len()).rev() {
            for j in 0..i {
                for k in 1..self.coeff[j].len() {
                    if self.coeff[j][k] == self.coeff[i][0] {
                        self.intermediate_xor(j, i);
                        continue;
                    }
                }
            }

            self.coeff[i].resize(1, 0);
        }
    }
}
