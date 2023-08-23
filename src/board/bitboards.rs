// Define the bitboard data type and its methods.

use std::fmt;

// Define the bitboard data type.
// We will use a 64-bit unsigned integer to represent the bitboard for each piece.
#[derive(Copy, Clone)]
pub struct Bitboard {
    w_to_move: bool,
    w_castle_k: bool,
    w_castle_q: bool,
    b_castle_k: bool,
    b_castle_q: bool,
    en_passant: u8, // file of en passant square (-1 if none)
    halfmove_clock: u8, // number of halfmoves since last capture or pawn advance
    wk: u64,
    wq: u64,
    wr: u64,
    wb: u64,
    wn: u64,
    wp: u64,
    bk: u64,
    bq: u64,
    br: u64,
    bb: u64,
    bn: u64,
    bp: u64,
}

// Little Endian Rank Mapping

// Least Significant File
// ind = 8 * rank + file

pub fn coords_to_sq_ind(file: u8, rank: u8) -> u8 {
    8 * rank + file
}

pub fn sq_ind_to_coords(sq_ind: u8) -> (u8, u8) {
    (sq_ind % 8, sq_ind / 8)
}

pub fn sq_ind_to_bit(sq_ind: u8) -> u64 {
    1 << sq_ind
}

pub fn bit_to_sq_ind(bit: u64) -> u8 {
    bit.trailing_zeros() as u8
}

pub fn sq_ind_to_algebraic(sq_ind: u8) -> String {
    let (file, rank) = sq_ind_to_coords(sq_ind);
    let file = (file + 97) as char;
    let rank = (rank + 49) as char;
    format!("{}{}", file, rank)
}

pub fn algebraic_to_sq_ind(algebraic: &str) -> u8 {
    let mut chars = algebraic.chars();
    let file = chars.next().unwrap() as u8 - 97;
    let rank = chars.next().unwrap() as u8 - 49;
    coords_to_sq_ind(file, rank)
}

pub fn algebraic_to_bit(algebraic: &str) -> u64 {
    let sq_ind = algebraic_to_sq_ind(algebraic);
    sq_ind_to_bit(sq_ind)
}

pub fn bit_to_algebraic(bit: u64) -> String {
    let sq_ind = bit_to_sq_ind(bit);
    sq_ind_to_algebraic(sq_ind)
}