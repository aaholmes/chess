//! Useful bitwise functions
//! Bits(n) iterates over set bits in n, bit_pairs(n) iterates over pairs of set bits in n,
//! plus functions for computing parity and getting and setting bits

/// Iterate over set bits in a u64
pub fn bits(n: &u64) -> impl Iterator<Item = usize> {
    let mut bits_left = *n;
    std::iter::from_fn(move || {
        if bits_left == 0 {
            None
        } else {
            let res: usize = bits_left.trailing_zeros() as usize;
            bits_left &= !(1 << res);
            Some(res)
        }
    })
}

// Bit operations named after Fortran intrinsics...

pub fn ibset(n: u128, b: i32) -> u128 {
    n | (1 << b)
}

pub fn ibclr(n: u128, b: i32) -> u128 {
    n & !(1 << b)
}

pub fn btest(n: u128, b: i32) -> bool {
    !(n & (1 << b) == 0)
}

pub fn parity(mut n: u128) -> i32 {
    // Returns 1 if even number of bits, -1 if odd number
    n ^= n >> 64;
    n ^= n >> 32;
    n ^= n >> 16;
    n ^= n >> 8;
    n ^= n >> 4;
    n ^= n >> 2;
    n ^= n >> 1;
    1 - 2 * ((n & 1) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_iters() {
    //     let det: u128 = 19273;
    //     println!("Bits:");
    //     for i in bits(det) {
    //         println!("{}", i);
    //     }
    //     println!("Bit pairs:");
    //     for (i, j) in bit_pairs(det) {
    //         println!("{} {}", i, j);
    //     }
    //     println!("Bits and bit pairs:");
    //     for bbp in bits_and_bit_pairs(&Config{up: det, dn: det}) {
    //         match bbp.1 {
    //             None => {
    //                 match bbp.0 {
    //                     Orbs::Double((p, q)) => println!("Opposite spin: ({}, {})", p, q),
    //                     Orbs::Single(p) => println!("Should not happen")
    //                 }
    //             },
    //             Some(is_alpha) => {
    //                 match bbp.0 {
    //                     Orbs::Double((p, q)) => {
    //                         if is_alpha {
    //                             println!("Same spin, up: ({}, {})", p, q);
    //                         } else {
    //                             println!("Same spin, dn: ({}, {})", p, q);
    //                         }
    //                     },
    //                     Orbs::Single(p) => {
    //                         if is_alpha {
    //                             println!("Single, up: {}", p);
    //                         } else {
    //                             println!("Single, dn: {}", p);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     assert_eq!(1, 1);
    // }

    fn parity_brute_force(n: u128) -> i32 {
        let mut out: i32 = 0;
        for _ in bits(n) {
            out ^= 1;
        }
        1 - 2 * out
    }

    #[test]
    fn test_parity() {
        for i in vec![
            14,
            15,
            27,
            1919,
            4958202,
            15 << 64,
            1 << 127,
            (1 << 126) + (1 << 65),
        ] {
            println!("Parity({}) = {} = {}", i, parity(i), parity_brute_force(i));
            assert_eq!(parity(i), parity_brute_force(i));
        }
    }
}
