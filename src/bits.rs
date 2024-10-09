//! Useful bitwise functions for chess engine operations.
//!
//! This module provides various bitwise operations that are commonly used in chess engines,
//! including bit iteration, bit manipulation, and bit counting functions.

/// Iterate over set bits in a u64.
///
/// This function returns an iterator that yields the indices of set bits in the input number.
///
/// # Arguments
///
/// * `n` - A reference to a 64-bit unsigned integer.
///
/// # Returns
///
/// An iterator over the indices of set bits in `n`.
///
/// # Examples
///
/// ```
/// use crate::bits::bits;
///
/// let n = 0b1010;
/// let set_bits: Vec<usize> = bits(&n).collect();
/// assert_eq!(set_bits, vec![1, 3]);
/// ```
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

/// Set a bit in a 128-bit unsigned integer.
///
/// # Arguments
///
/// * `n` - The number to modify.
/// * `b` - The index of the bit to set (0-127).
///
/// # Returns
///
/// A new 128-bit unsigned integer with the specified bit set.
pub fn ibset(n: u128, b: i32) -> u128 {
    n | (1 << b)
}

/// Clear a bit in a 128-bit unsigned integer.
///
/// # Arguments
///
/// * `n` - The number to modify.
/// * `b` - The index of the bit to clear (0-127).
///
/// # Returns
///
/// A new 128-bit unsigned integer with the specified bit cleared.
pub fn ibclr(n: u128, b: i32) -> u128 {
    n & !(1 << b)
}

/// Test a bit in a 128-bit unsigned integer.
///
/// # Arguments
///
/// * `n` - The number to test.
/// * `b` - The index of the bit to test (0-127).
///
/// # Returns
///
/// `true` if the specified bit is set, `false` otherwise.
pub fn btest(n: u128, b: i32) -> bool {
    n & (1 << b) != 0
}

/// Count the number of set bits in a 64-bit unsigned integer.
///
/// This function implements the population count (popcnt) operation.
///
/// # Arguments
///
/// * `n` - The number to count set bits in.
///
/// # Returns
///
/// The number of set bits in `n`.
pub fn popcnt(n: u64) -> i32 {
    let mut count = 0;
    let mut nn = n;
    while nn != 0 {
        nn &= nn - 1;
        count += 1;
    }
    count
}

/// Compute the parity of a 128-bit unsigned integer.
///
/// # Arguments
///
/// * `n` - The number to compute parity for.
///
/// # Returns
///
/// 1 if the number of set bits is even, -1 if odd.
pub fn parity(mut n: u128) -> i32 {
    n ^= n >> 64;
    n ^= n >> 32;
    n ^= n >> 16;
    n ^= n >> 8;
    n ^= n >> 4;
    n ^= n >> 2;
    n ^= n >> 1;
    1 - 2 * ((n & 1) as i32)
}