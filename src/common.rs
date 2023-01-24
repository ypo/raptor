use crate::tables::{SYSTEMATIC_INDEX, V0, V1};

/// Computes the number of intermediate symbols (L), the first prime number greater than or equal to L (L_prime),
/// the number of LDPC symbols (S), and the number of half-symbols (H) from the number of source symbols (K),
/// as specified in RFC section 5.4.2.3.
///
/// # Parameters
///
/// * `k`: The number of source symbols.
///
/// # Returns
///
/// A tuple containing:
/// * `L`: The number of intermediate symbols desired (K+S+H)
/// * `L_prime`: The first prime number greater than or equal to L
/// * `S`: The number of LDPC symbols
/// * `H`: The number of half-symbols
/// * `H_prime`: ceil(H/2)
///
pub fn intermediate_symbols(k: u32) -> (u32, u32, u32, u32, u32) {
    // X be the smallest positive integer such that X*(X-1) >= 2*K.
    // X^2 - X - 2k >= 0
    // det = b^ - 4ac
    let x = ((1f64 + f64::sqrt(1f64 + (8f64 * k as f64))) / 2f64).ceil() as u64;

    //S be the smallest prime integer such that S >= ceil(0.01*K) + X
    let s = (0.01f64 * k as f64).ceil() as u64 + x;
    let s = prime_greater_or_equal(s);

    // H is the smallest integer such that choose(H, ceil(H/2)) >= K + S
    let mut h = 1;
    while choose(h, ((h as f64) / 2.0).ceil() as u64) < k as u64 + s {
        h += 1
    }

    let hp = (h as f32 / 2.0).ceil() as u32;
    let l = k as u64 + s + h;
    let l_prime = prime_greater_or_equal(l);

    (l as u32, l_prime as u32, s as u32, h as u32, hp)
}

fn prime_greater_or_equal(p: u64) -> u64 {
    let mut p = p;
    while !primes::is_prime(p) {
        p += 1;
    }
    p
}

///
/// number of ways n objects can be chosen from among r objects without repetition.
///
/// # Returns
/// n! / (r! * (n - r)!)
fn choose(n: u64, r: u64) -> u64 {
    factorial(n) / (factorial(r) * factorial(n - r))
}

/// Calculate n!
fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

// how many bits in x are set.
// This algorithm basically uses shifts and ANDs to sum up the bits in
// a tree fashion.
fn nb_bits_set(x: u64) -> u64 {
    let mut x = x;
    x -= (x >> 1) & 0x5555555555555555;
    x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
    x = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
    (x * 0x0101010101010101) >> 56
}

pub fn bit_set(x: u32, b: u32) -> bool {
    return (x >> b) & 1 == 1;
}

/// Sequence of gray number that have exactly b bits set
pub fn gray_sequence(length: usize, b: u32) -> Vec<u32> {
    let mut s = vec![0u32; length];
    let mut i = 0;

    let mut x = 0u64;
    loop {
        let g = (x >> 1) ^ x; // Gray code
        if nb_bits_set(g) == b as u64 {
            s[i] = g as u32;
            i += 1;
            if i >= length {
                break;
            }
        }
        x += 1
    }
    s
}

pub fn rand(x: u32, i: u32, m: u32) -> u32 {
    let v0 = V0[((x + i) % 256) as usize];
    let v1 = V1[(((x / 256) + i) % 256) as usize];
    (v0 ^ v1) % m
}

/// REF 5.4.4.2. Degree Generator
pub fn deg(v: u32) -> u32 {
    static F: [u32; 8] = [0, 10241, 491582, 712794, 831695, 948446, 1032189, 1048576];
    static D: [u32; 8] = [0, 1, 2, 3, 4, 10, 11, 40];

    for j in 1..F.len() {
        // f[j-1] <= v < f[j]
        // Note: F[j - 1] <= v is always true if v < F[j] is true
        if v < F[j] {
            return D[j];
        }
    }

    log::error!("Cannot find valid degree");
    D[D.len() - 1]
}

/// RFC section 5.4.4.4. Triple Generator
///
/// # Parameters
///
/// k the number of source symbols.
/// l the number of intermediate symbols desired (K+S+H)
/// lp smallest prime that is greater than or equal to L
/// x an encoding symbol ID (ESI)
///
/// # Returns
///
/// (d, a, b)
fn triple(k: u32, x: u32, _l: u32, l_prime: u32) -> (u32, u32, u32) {
    const Q: u64 = 65521; // largest prime smaller than 2^^16.
                          // systematic index associated with K
    let jk = SYSTEMATIC_INDEX[k as usize] as u64;

    // A = (53591 + J(K)*997) % Q
    let a = (53591u64 + (jk * 997u64)) % Q;
    // B = 10267*(J(K)+1) % Q
    let b = (10267u64 * (jk + 1u64)) % Q;
    //  Y = (B + X*A) % Q
    let y = (b + (x as u64 * a)) % Q;
    // v = Rand[Y, 0, 2^^20]
    let v = rand(y as u32, 0, 1048576);
    // d = Deg[v]
    let d = deg(v);
    // a = 1 + Rand[Y, 1, L'-1]
    let a = 1 + rand(y as u32, 1, (l_prime - 1) as u32);
    // b = Rand[Y, 2, L']
    let b = rand(y as u32, 2, l_prime as u32);

    (d, a, b)
}

pub fn find_lt_indices(k: u32, x: u32, l: u32, l_prime: u32) -> Vec<u32> {
    let (mut d, a, mut b) = triple(k, x, l, l_prime);
    if d > l {
        d = l;
    }

    let mut indices = Vec::new();
    while b >= l {
        b = (b + a) % l_prime;
    }
    indices.push(b);

    for _ in 1..d {
        b = (b + a) % l_prime;
        while b >= l {
            b = (b + a) % l_prime;
        }
        indices.push(b);
    }

    indices.sort();
    indices
}

pub fn lt_encode(k: u32, x: u32, l: u32, l_prime: u32, c: &[Vec<u8>]) -> Vec<u8> {
    let indices = find_lt_indices(k, x, l, l_prime);
    let mut block: Vec<u8> = Vec::new();
    for i in indices {
        xor(&mut block, &c[i as usize]);
    }
    block
}

/// block partitioning function from RFC 5053 S.5.3.1.2
/// See http://tools.ietf.org/html/rfc5053
/// Partitions a number i (a size) into j semi-equal pieces. The details are
/// in the return values: there are jl longer pieces of size il, and js shorter
/// (il, is, jl, js)
pub fn partition(i: usize, j: usize) -> (usize, usize, usize, usize) {
    let mut il = (i as f64 / j as f64).ceil() as usize;
    let mut is = (i as f64 / j as f64).floor() as usize;
    let jl = i - (is * j);
    let js = j - jl;

    if jl == 0 {
        il = 0
    }
    if js == 0 {
        is = 0
    }

    (il, is, jl, js)
}

pub fn xor(row_1: &mut Vec<u8>, row_2: &[u8]) {
    if row_1.len() < row_2.len() {
        row_1.resize(row_2.len(), 0);
    }

    row_1
        .iter_mut()
        .zip(row_2.iter())
        .for_each(|(v1, v2)| *v1 ^= *v2);
}

///
/// disjunctive union of row, assuming row coeff are sorted
///  
pub fn disjunctive_union(row_1: &[u32], row_2: &[u32]) -> Vec<u32> {
    let mut output: Vec<u32> = Vec::new();

    let mut i = 0;
    let mut j = 0;
    while i < row_1.len() && j < row_2.len() {
        let v_1 = row_1[i];
        let v_2 = row_2[j];
        if v_1 == v_2 as u32 {
            // remove intersection
            i += 1;
            j += 1;
        } else if v_1 < v_2 {
            output.push(v_1);
            i += 1;
        } else {
            output.push(v_2);
            j += 1;
        }
    }

    // Append remaining elements from row1 and row2 if any
    output.extend(&row_1[i..]);
    output.extend(&row_2[j..]);
    output
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_triple() {
        crate::tests::init();

        struct TripleTest {
            k: u32,
            x: u32,
            d: u32,
            a: u32,
            b: u32,
        }

        let test_vector: Vec<TripleTest> = vec![
            TripleTest {
                k: 0,
                x: 3,
                d: 2,
                a: 4,
                b: 3,
            },
            TripleTest {
                k: 1,
                x: 4,
                d: 4,
                a: 2,
                b: 5,
            },
            TripleTest {
                k: 4,
                x: 0,
                d: 10,
                a: 13,
                b: 1,
            },
            TripleTest {
                k: 4,
                x: 4,
                d: 4,
                a: 6,
                b: 2,
            },
            TripleTest {
                k: 500,
                x: 514,
                d: 2,
                a: 107,
                b: 279,
            },
            TripleTest {
                k: 1000,
                x: 52918,
                d: 3,
                a: 1070,
                b: 121,
            },
        ];

        for test in &test_vector {
            let (l, l_prime, _, _, _) = super::intermediate_symbols(test.k);
            let (d, a, b) = super::triple(test.k, test.x, l, l_prime);

            log::info!("{}/{} {}/{} {}/{}", d, test.d, a, test.a, b, test.b);

            assert!(d == test.d);
            assert!(a == test.a);
            assert!(b == test.b);
        }
    }

    #[test]
    fn test_intermediate_symbols() {
        crate::tests::init();

        struct Test {
            k: u32,
            l: u32,
            s: u32,
            h: u32,
        }

        let test_vector: Vec<Test> = vec![
            Test {
                k: 0,
                l: 4,
                s: 2,
                h: 2,
            },
            Test {
                k: 1,
                l: 8,
                s: 3,
                h: 4,
            },
            Test {
                k: 10,
                l: 23,
                s: 7,
                h: 6,
            },
            Test {
                k: 14,
                l: 28,
                s: 7,
                h: 7,
            },
            Test {
                k: 500,
                l: 553,
                s: 41,
                h: 12,
            },
            Test {
                k: 5000,
                l: 5166,
                s: 151,
                h: 15,
            },
        ];

        for test in &test_vector {
            let (l, _, s, h, _) = super::intermediate_symbols(test.k);
            assert!(l == test.l);
            assert!(s == test.s);
            assert!(h == test.h);
        }
    }

    #[test]
    fn test_lt_indices() {
        struct Test {
            k: u32,
            x: u32,
            indices: Vec<u32>,
        }

        let test_vector = vec![
            Test {
                k: 4,
                x: 0,
                indices: vec![1, 2, 3, 4, 6, 7, 8, 10, 11, 12],
            },
            Test {
                k: 4,
                x: 4,
                indices: vec![2, 3, 8, 9],
            },
            Test {
                k: 100,
                x: 1,
                indices: vec![51, 104],
            },
            Test {
                k: 1000,
                x: 727,
                indices: vec![306, 687, 1040],
            },
            Test {
                k: 10,
                x: 57279,
                indices: vec![19, 20, 21, 22],
            },
        ];

        for test in &test_vector {
            let (l, l_prime, _, _, _) = super::intermediate_symbols(test.k);
            let indices = super::find_lt_indices(test.k, test.x, l, l_prime);
            log::info!("{:?} / {:?}", indices, test.indices);
            assert!(indices == test.indices);
        }
    }
}
