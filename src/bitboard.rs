// Define the bitboard data type and its methods.

use std::fmt;
use std::ptr::write;

// Define the bitboard data type.
// We will use a 64-bit unsigned integer to represent the bitboard for each piece.
#[derive(Copy, Clone)]
pub struct Bitboard {
    pub(crate) w_to_move: bool,
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
    pub(crate) pieces: [u64; 12], // just for now, specify everything twice. once it's working, we can choose one of the two representations
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

pub fn flip_sq_ind_vertically(sq_ind: u8) -> u8 {
    8 * (7 - sq_ind / 8) + sq_ind % 8
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
        let wk: u64 = 0x0000000000000010;
        let wq: u64 = 0x0000000000000008;
        let wr: u64 = 0x0000000000000081;
        let wb: u64 = 0x0000000000000024;
        let wn: u64 = 0x0000000000000042;
        let wp: u64 = 0x000000000000FF00;
        let bk: u64 = 0x1000000000000000;
        let bq: u64 = 0x0800000000000000;
        let br: u64 = 0x8100000000000000;
        let bb: u64 = 0x2400000000000000;
        let bn: u64 = 0x4200000000000000;
        let bp: u64 = 0x00FF000000000000;
        Bitboard {
            w_to_move: true,
            w_castle_k: true,
            w_castle_q: true,
            b_castle_k: true,
            b_castle_q: true,
            en_passant: None,
            halfmove_clock: 0,
            wk,
            wq,
            wr,
            wb,
            wn,
            wp,
            bk,
            bq,
            br,
            bb,
            bn,
            bp,
            pieces: [wp, bp, wn, bn, wb, bb, wr, br, wq, bq, wk, bk]
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
        let mut flipped_pieces: [u64; 12] = [0; 12];
        for i in 0..12 {
            flipped_pieces[i] = flip_vertically(self.pieces[i]);
        }
        Bitboard {
            w_to_move: !self.w_to_move,
            w_castle_k: self.b_castle_k,
            w_castle_q: self.b_castle_q,
            b_castle_k: self.w_castle_k,
            b_castle_q: self.w_castle_q,
            en_passant: {if self.en_passant == None {None} else {Some(flip_sq_ind_vertically(self.en_passant.unwrap()))}},
            halfmove_clock: self.halfmove_clock,
            wk: flipped_pieces[10],
            wq: flipped_pieces[8],
            wr: flipped_pieces[6],
            wb: flipped_pieces[4],
            wn: flipped_pieces[2],
            wp: flipped_pieces[0],
            bk: flipped_pieces[11],
            bq: flipped_pieces[9],
            br: flipped_pieces[7],
            bb: flipped_pieces[5],
            bn: flipped_pieces[3],
            bp: flipped_pieces[1],
            pieces: flipped_pieces
        }
    }
}