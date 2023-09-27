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
    n & (1 << b) != 0
}

pub fn popcnt(n: u64) -> i32 {
    // Returns number of set bits
    let mut count = 0;
    let mut nn = n;
    while nn != 0 {
        nn &= nn - 1;
        count += 1;
    }
    count
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
