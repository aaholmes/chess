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
    en_passant: Option<u8>, // index of square where en passant is possible
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

pub fn flip_vertically(bit: u64) -> u64 {
    return  ( (bit << 56)                           ) |
        ( (bit << 40) & (0x00ff000000000000) ) |
        ( (bit << 24) & (0x0000ff0000000000) ) |
        ( (bit <<  8) & (0x000000ff00000000) ) |
        ( (bit >>  8) & (0x00000000ff000000) ) |
        ( (bit >> 24) & (0x0000000000ff0000) ) |
        ( (bit >> 40) & (0x000000000000ff00) ) |
        ( (bit >> 56) );
}

impl Bitboard {
    pub(crate) fn new() -> Bitboard {
        Bitboard {
            w_to_move: true,
            w_castle_k: true,
            w_castle_q: true,
            b_castle_k: true,
            b_castle_q: true,
            en_passant: None,
            halfmove_clock: 0,
            wk: 0x0000000000000010,
            wq: 0x0000000000000008,
            wr: 0x0000000000000081,
            wb: 0x0000000000000024,
            wn: 0x0000000000000042,
            wp: 0x000000000000FF00,
            bk: 0x1000000000000000,
            bq: 0x0800000000000000,
            br: 0x8100000000000000,
            bb: 0x2400000000000000,
            bn: 0x4200000000000000,
            bp: 0x00FF000000000000,
        }
    }

    pub fn print(self) {
        println!("  +-----------------+");
        for rank in (0..8).rev() {
            print!("{} | ", rank + 1);
            for file in 0..8 {
                let sq_ind = coords_to_sq_ind(file, rank);
                let bit = sq_ind_to_bit(sq_ind);
                if bit & self.wk != 0 {
                    print!("K ");
                } else if bit & self.wq != 0 {
                    print!("Q ");
                } else if bit & self.wr != 0 {
                    print!("R ");
                } else if bit & self.wb != 0 {
                    print!("B ");
                } else if bit & self.wn != 0 {
                    print!("N ");
                } else if bit & self.wp != 0 {
                    print!("P ");
                } else if bit & self.bk != 0 {
                    print!("k ");
                } else if bit & self.bq != 0 {
                    print!("q ");
                } else if bit & self.br != 0 {
                    print!("r ");
                } else if bit & self.bb != 0 {
                    print!("b ");
                } else if bit & self.bn != 0 {
                    print!("n ");
                } else if bit & self.bp != 0 {
                    print!("p ");
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }

    pub fn flip_vertically(self) -> Bitboard {
        // Flip the board vertically, returning a new board.
        Bitboard {
            w_to_move: !self.w_to_move,
            w_castle_k: self.b_castle_k,
            w_castle_q: self.b_castle_q,
            b_castle_k: self.w_castle_k,
            b_castle_q: self.w_castle_q,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            wk: flip_vertically(self.wk),
            wq: flip_vertically(self.wq),
            wr: flip_vertically(self.wr),
            wb: flip_vertically(self.wb),
            wn: flip_vertically(self.wn),
            wp: flip_vertically(self.wp),
            bk: flip_vertically(self.bk),
            bq: flip_vertically(self.bq),
            br: flip_vertically(self.br),
            bb: flip_vertically(self.bb),
            bn: flip_vertically(self.bn),
            bp: flip_vertically(self.bp)
        }
    }
}