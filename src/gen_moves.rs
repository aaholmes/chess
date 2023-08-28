// Generate all possible moves for a given position
// We divide it up into two functions: one for generating moves for a single piece, and one for generating moves for all pieces.
// The latter will call the former for each piece.
// For this version, we generate moves in the following order:
// 1. Pawn captures and promotions
// 2. Knight captures
// 3. Bishop captures
// 4. Rook captures
// 5. Queen captures
// 6. King captures
// 7. All other moves, ordered by change in pesto eval (precomputed)
// This is not the most efficient order, but it is easy to implement and has the effect of favoring captures.
// Also it will be easy to incorporate MVV-LVA ordering later.


use crate::bitboard::{sq_ind_to_bit, WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK, Bitboard};
use crate::bits::bits;

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
const NOT_AB_FILE: u64 = 0xfcfcfcfcfcfcfcfc;
const NOT_GH_FILE: u64 = 0x3f3f3f3f3f3f3f3f;
const NOT_1_RANK: u64 = 0xffffffffffffff00;
const NOT_8_RANK: u64 = 0x00ffffffffffffff;
const NOT_12_RANK: u64 = 0x0000ffffffffffff;
const NOT_78_RANK: u64 = 0xffffffffff000000;
const RANK_2: u64 = 0x000000000000ff00;
const RANK_7: u64 = 0x00ff000000000000;

struct MoveGen {
    // Generate all possible moves for a given position.
    // For now, only non-sliding moves.
    wp_captures_promotions: Vec<Vec<usize>>,
    bp_captures_promotions: Vec<Vec<usize>>,
    n_moves: Vec<Vec<usize>>,
    k_moves: Vec<Vec<usize>>,
    wp_moves: Vec<Vec<usize>>,
    bp_moves: Vec<Vec<usize>>
}

fn init_king_moves(from_sq_ind: usize) -> Vec<usize> {
    // Initialize the king moves for a given square.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut out: Vec<usize> = Vec::new();
    // Castling
    if from_sq_ind == 4 {
        out.push(6);
        out.push(2);
    } else if from_sq_ind == 60 {
        out.push(62);
        out.push(58);
    }
    // Regular king moves
    if from_bit & NOT_A_FILE != 0 {
        out.push(from_sq_ind - 1);
    }
    if from_bit & NOT_8_RANK != 0 {
        if from_bit & NOT_A_FILE != 0 {
            out.push(from_sq_ind + 7);
        }
        out.push(from_sq_ind + 8);
        if from_bit & NOT_H_FILE != 0 {
            out.push(from_sq_ind + 9);
        }
    }
    if from_bit & NOT_H_FILE != 0 {
        out.push(from_sq_ind + 1);
    }
    if from_bit & NOT_1_RANK != 0 {
        if from_bit & NOT_H_FILE != 0 {
            out.push(from_sq_ind - 7);
        }
        out.push(from_sq_ind - 8);
        if from_bit & NOT_A_FILE != 0 {
            out.push(from_sq_ind - 9);
        }
    }
    out
}

fn init_knight_moves(from_sq_ind: usize) -> Vec<usize> {
    // Initialize the knight moves a given square.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut out: Vec<usize> = Vec::new();
    if from_bit & NOT_A_FILE & NOT_78_RANK != 0 {
        out.push(from_sq_ind + 15);
    }
    if from_bit & NOT_H_FILE & NOT_78_RANK != 0 {
        out.push(from_sq_ind + 17);
    }
    if from_bit & NOT_AB_FILE & NOT_8_RANK != 0 {
        out.push(from_sq_ind + 6);
    }
    if from_bit & NOT_GH_FILE & NOT_8_RANK != 0 {
        out.push(from_sq_ind + 10);
    }
    if from_bit & NOT_AB_FILE & NOT_1_RANK != 0 {
        out.push(from_sq_ind - 10);
    }
    if from_bit & NOT_GH_FILE & NOT_1_RANK != 0 {
        out.push(from_sq_ind - 6);
    }
    if from_bit & NOT_A_FILE & NOT_12_RANK != 0 {
        out.push(from_sq_ind - 17);
    }
    if from_bit & NOT_H_FILE & NOT_12_RANK != 0 {
        out.push(from_sq_ind - 15);
    }
    out
}

fn init_pawn_captures_promotions(from_sq_ind: usize) -> (Vec<usize>, Vec<usize>) {
    // Initialize the pawn captures and promotions for a given square.
    // Separate for white and black.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut white: Vec<usize> = Vec::new();
    if from_bit & NOT_A_FILE != 0 {
        white.push(from_sq_ind + 7);
    }
    if from_bit & NOT_H_FILE != 0 {
        white.push(from_sq_ind + 9);
    }
    if from_bit & RANK_7 != 0 {
        white.push(from_sq_ind + 8);
    }
    let mut black: Vec<usize> = Vec::new();
    if from_bit & NOT_A_FILE != 0 {
        black.push(from_sq_ind - 9);
    }
    if from_bit & NOT_H_FILE != 0 {
        black.push(from_sq_ind - 7);
    }
    if from_bit & RANK_2 != 0 {
        black.push(from_sq_ind - 8);
    }
    (white, black)
}

fn init_pawn_moves(from_sq_ind: usize) -> (Vec<usize>, Vec<usize>) {
    // Initialize the pawn moves for a given square.
    // Separate for white and black.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut black: Vec<usize> = Vec::new();
    if from_bit & RANK_2 != 0 {
        black.push(from_sq_ind + 16);
    }
    black.push(from_sq_ind + 8);
    let mut white: Vec<usize> = Vec::new();
    if from_bit & RANK_7 != 0 {
        white.push(from_sq_ind - 16);
    }
    white.push(from_sq_ind - 8);
    (white, black)
}

impl MoveGen {
    pub fn new() -> MoveGen {
        // Initialize the move generator by creating the iterators for Pawn, Knight, and King moves.
        let mut wp_captures_promotions: Vec<Vec<usize>> = Vec::new();
        let mut bp_captures_promotions: Vec<Vec<usize>> = Vec::new();
        let mut n_moves: Vec<Vec<usize>> = Vec::new();
        let mut k_moves: Vec<Vec<usize>> = Vec::new();
        let mut wp_moves: Vec<Vec<usize>> = Vec::new();
        let mut bp_moves: Vec<Vec<usize>> = Vec::new();
        let mut wp: Vec<usize>;
        let mut bp: Vec<usize>;
        for from_sq_ind in 0..64 {
            wp = vec![];
            bp = vec![];
            (wp, bp) = init_pawn_captures_promotions(from_sq_ind);
            wp_captures_promotions.push(wp.clone());
            bp_captures_promotions.push(bp.clone());
            n_moves.push(init_knight_moves(from_sq_ind));
            k_moves.push(init_king_moves(from_sq_ind));
            wp = vec![];
            bp = vec![];
            (wp, bp) = init_pawn_moves(from_sq_ind);
            wp_moves.push(wp.clone());
            bp_moves.push(bp.clone());
        }
        MoveGen {
            wp_captures_promotions: wp_captures_promotions,
            bp_captures_promotions: bp_captures_promotions,
            n_moves: n_moves,
            k_moves: k_moves,
            wp_moves: wp_moves,
            bp_moves: bp_moves
        }
    }


    pub fn gen_knight_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible knight moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    } else {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    } else {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    pub fn gen_king_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible king moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WK]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    } else {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BK]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    } else {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }
}