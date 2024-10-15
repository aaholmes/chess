// Little Endian Rank Mapping
// Least Significant File
// ind = 8 * rank + file

/// Converts file and rank coordinates to a square index.
///
/// # Arguments
///
/// * `file` - The file (0-7, where 0 is the a-file)
/// * `rank` - The rank (0-7, where 0 is the first rank)
///
/// # Returns
///
/// The square index (0-63)
pub fn coords_to_sq_ind(file: usize, rank: usize) -> usize {
    8 * rank + file
}

/// Converts a square index to file and rank coordinates.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A tuple (file, rank) where file and rank are 0-7
pub fn sq_ind_to_coords(sq_ind: usize) -> (usize, usize) {
    (sq_ind % 8, sq_ind / 8)
}

/// Converts a square index to a bitboard representation.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A 64-bit integer with only the bit at the given square index set
pub fn sq_ind_to_bit(sq_ind: usize) -> u64 {
    1 << sq_ind
}

/// Converts a bitboard with a single bit set to its square index.
///
/// # Arguments
///
/// * `bit` - A 64-bit integer with only one bit set
///
/// # Returns
///
/// The index of the set bit (0-63)
pub fn bit_to_sq_ind(bit: u64) -> usize {
    bit.trailing_zeros() as usize
}

/// Converts a square index to algebraic notation.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A string representing the square in algebraic notation (e.g., "e4")
pub fn sq_ind_to_algebraic(sq_ind: usize) -> String {
    let (file, rank) = sq_ind_to_coords(sq_ind);
    let file = (file + 97) as u8 as char;
    let rank = (rank + 49) as u8 as char;
    format!("{}{}", file, rank)
}

/// Converts algebraic notation to a square index.
///
/// # Arguments
///
/// * `algebraic` - A string representing a square in algebraic notation (e.g., "e4")
///
/// # Returns
///
/// The corresponding square index (0-63)
pub fn algebraic_to_sq_ind(algebraic: &str) -> usize {
    let mut chars = algebraic.chars();
    let file = chars.next().unwrap() as usize - 97;
    let rank = chars.next().unwrap() as usize - 49;
    coords_to_sq_ind(file, rank)
}

/// Converts algebraic notation to a bitboard representation.
///
/// # Arguments
///
/// * `algebraic` - A string representing a square in algebraic notation (e.g., "e4")
///
/// # Returns
///
/// A 64-bit integer with only the bit at the given square set
pub fn algebraic_to_bit(algebraic: &str) -> u64 {
    let sq_ind = algebraic_to_sq_ind(algebraic);
    sq_ind_to_bit(sq_ind)
}

/// Converts a bitboard with a single bit set to algebraic notation.
///
/// # Arguments
///
/// * `bit` - A 64-bit integer with only one bit set
///
/// # Returns
///
/// A string representing the square in algebraic notation (e.g., "e4")
pub fn bit_to_algebraic(bit: u64) -> String {
    let sq_ind = bit_to_sq_ind(bit);
    sq_ind_to_algebraic(sq_ind)
}

/// Flips a square index vertically on the board.
///
/// # Arguments
///
/// * `sq_ind` - The square index to flip (0-63)
///
/// # Returns
///
/// The vertically flipped square index (0-63)
pub fn flip_sq_ind_vertically(sq_ind: usize) -> usize {
    8 * (7 - sq_ind / 8) + sq_ind % 8
}

/// Flips a bitboard vertically.
///
/// # Arguments
///
/// * `bit` - The bitboard to flip
///
/// # Returns
///
/// The vertically flipped bitboard
pub fn flip_vertically(bit: u64) -> u64 {
    ( (bit << 56)                    ) |
        ( (bit << 40) & (0x00ff000000000000) ) |
        ( (bit << 24) & (0x0000ff0000000000) ) |
        ( (bit <<  8) & (0x000000ff00000000) ) |
        ( (bit >>  8) & (0x00000000ff000000) ) |
        ( (bit >> 24) & (0x0000000000ff0000) ) |
        ( (bit >> 40) & (0x000000000000ff00) ) |
        ( (bit >> 56) )
}