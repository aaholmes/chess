// Generate all possible moves for a given position
// We divide it up into two functions: one for generating moves for a single piece, and one for generating moves for all pieces.
// The latter will call the former for each piece.
// For this version, we generate moves in the following order:
// 1. Captures/promotions in MVV-LVA order
// 2. All other moves, ordered by change in pesto eval (precomputed)
// I think this is relatively easy to do, and may even be close to optimal, even if we later change the eval function, e.g. to NNUE.
// We can also use quiescence search to only generate captures and promotions.
// For capture move orders, we should use 10*victim_value - attacker_value, where PNBRQK = 123456 e.g., PxQ = 10*5 - 1 = 49, KxN = 10*3 - 6 = 24, etc.
// For promotions, we should use 10*promoted_piece_value - attacker_value, so Q promotion = 10*5 - 1 = 49, etc.
// We want to use two heaps, one for captures and one for non-captures, with each element of the heap being a vector of sorted moves.
// Note that non-captures can be pre-sorted, but captures require the piece value of the captured piece and so they have to be internally sorted, which may be just as fast as sorting the entire vector.
// Note also that the pesto eval has 25 game modes, ranging from opening to endgame, so our non-capture move ordering should be different for each game mode.
// Note also that once we implement all of this, as well as quiescence search, we can profile the code to see where the bottlenecks are.


use crate::bitboard::{Bitboard, sq_ind_to_bit, WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK, WOCC, BOCC, OCC};
use crate::bits::bits;
use crate::magic_constants::{R_MAGICS, B_MAGICS, R_BITS, B_BITS, R_MASKS, B_MASKS};

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
const NOT_AB_FILE: u64 = 0xfcfcfcfcfcfcfcfc;
const NOT_GH_FILE: u64 = 0x3f3f3f3f3f3f3f3f;
const NOT_1_RANK: u64 = 0xffffffffffffff00;
const NOT_8_RANK: u64 = 0x00ffffffffffffff;
const NOT_12_RANK: u64 = 0xffffffffffff0000;
const NOT_78_RANK: u64 = 0x0000ffffffffffff;
const RANK_2: u64 = 0x000000000000ff00;
const RANK_7: u64 = 0x00ff000000000000;

pub(crate) struct MoveGen {
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
    let mut black: Vec<usize> = Vec::new();
    if from_bit & NOT_1_RANK & NOT_8_RANK != 0 {
        if from_bit & NOT_A_FILE != 0 {
            white.push(from_sq_ind + 7);
        }
        if from_bit & NOT_H_FILE != 0 {
            white.push(from_sq_ind + 9);
        }
        if from_bit & RANK_7 != 0 {
            white.push(from_sq_ind + 8);
        }
        if from_bit & NOT_A_FILE != 0 {
            black.push(from_sq_ind - 9);
        }
        if from_bit & NOT_H_FILE != 0 {
            black.push(from_sq_ind - 7);
        }
        if from_bit & RANK_2 != 0 {
            black.push(from_sq_ind - 8);
        }
    }
    (white, black)
}

fn init_pawn_moves(from_sq_ind: usize) -> (Vec<usize>, Vec<usize>) {
    // Initialize the pawn moves for a given square.
    // Separate for white and black.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut white: Vec<usize> = Vec::new();
    let mut black: Vec<usize> = Vec::new();
    if from_bit & NOT_1_RANK & NOT_8_RANK != 0 {
        if from_bit & RANK_2 != 0 {
            white.push(from_sq_ind + 16);
        }
        white.push(from_sq_ind + 8);
        if from_bit & RANK_7 != 0 {
            black.push(from_sq_ind - 16);
        }
        black.push(from_sq_ind - 8);
    }
    (white, black)
}

fn append_promotions(captures: &mut Vec<(usize, usize, Option<usize>)>, from_sq_ind: usize, to_sq_ind: &usize, w_to_move: bool) {
    if w_to_move {
        captures.push((from_sq_ind, *to_sq_ind, Some(WQ)));
        captures.push((from_sq_ind, *to_sq_ind, Some(WR)));
        captures.push((from_sq_ind, *to_sq_ind, Some(WN)));
        captures.push((from_sq_ind, *to_sq_ind, Some(WB)));
    } else {
        captures.push((from_sq_ind, *to_sq_ind, Some(BQ)));
        captures.push((from_sq_ind, *to_sq_ind, Some(BR)));
        captures.push((from_sq_ind, *to_sq_ind, Some(BN)));
        captures.push((from_sq_ind, *to_sq_ind, Some(BB)));
    }
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

    pub fn gen_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        self.gen_pawn_moves(board)
    }

    fn gen_pawn_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible pawn moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Treats promotions as captures.
        // Lists promotions in the following order: queen, rook, knight, bishop, since bishop promotions are very rare.
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WP]) {
                for to_sq_ind in &self.wp_captures_promotions[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push((from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.wp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            if from_sq_ind > 7 && from_sq_ind < 16 {
                                if board.get_piece(*to_sq_ind + 8) == None {
                                    moves.push((from_sq_ind, *to_sq_ind, None));
                                }
                            } else {
                                moves.push((from_sq_ind, *to_sq_ind, None));
                            }
                        }
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BP]) {
                for to_sq_ind in &self.bp_captures_promotions[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push((from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.bp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            if from_sq_ind > 47 && from_sq_ind < 56 {
                                if board.get_piece(*to_sq_ind - 8) == None {
                                    moves.push((from_sq_ind, *to_sq_ind, None));
                                }
                            } else {
                                moves.push((from_sq_ind, *to_sq_ind, None));
                            }
                        }
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_knight_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible knight moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    } else if {board.pieces[WOCC] & (1 << to_sq_ind) == 0} {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    } else if {board.pieces[BOCC] & (1 << to_sq_ind) == 0} {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_king_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible king moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WK]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    } else if {board.pieces[WOCC] & (1 << to_sq_ind) == 0} {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BK]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push((from_sq_ind, *to_sq_ind, None));
                    } else if {board.pieces[BOCC] & (1 << to_sq_ind) == 0} {
                        moves.push((from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_rook_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible rook moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut m: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut c: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WR]) {
                // Mask blockers
                blockers = board.pieces(OCC) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * R_MAGICS[from_sq_ind]) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = R_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[BOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BR]) {
                // Mask blockers
                blockers = board.pieces(OCC) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * R_MAGICS[from_sq_ind]) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = R_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[WOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        }
        (captures, moves)
    }

    fn gen_bishop_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible bishop moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut m: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut c: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WB]) {
                // Mask blockers
                blockers = board.pieces(OCC) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * B_MAGICS[from_sq_ind]) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = B_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[BOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BB]) {
                // Mask blockers
                blockers = board.pieces(OCC) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * B_MAGICS[from_sq_ind]) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = B_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[WOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        }
        (captures, moves)
    }

    fn gen_queen_moves(&self, board: &Bitboard) -> (Vec<(usize, usize, Option<usize>)>, Vec<(usize, usize, Option<usize>)>) {
        // Generate all possible queen moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut captures: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut m: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut c: Vec<(usize, usize, Option<usize>)> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WQ]) {
                // Mask blockers
                blockers = board.pieces(OCC) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * R_MAGICS[from_sq_ind]) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = R_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[BOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
                // Mask blockers
                blockers = board.pieces(OCC) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * B_MAGICS[from_sq_ind]) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = B_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[BOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BQ]) {
                // Mask blockers
                blockers = board.pieces(OCC) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * R_MAGICS[from_sq_ind]) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = R_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[WOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
                // Mask blockers
                blockers = board.pieces(OCC) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers * B_MAGICS[from_sq_ind]) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                // TODO: make this table contain both captures and noncaptures as separate vectors
                (c, m) = B_TABLE[from_sq_ind][key];
                for (from, to, _) in c {
                    if board.pieces[WOCC] & (1 << to) != 0 {
                        captures.push((from, to, None));
                    }
                }
                for (from, to, _) in m {
                    moves.push((from, to, None));
                }
            }
        }
        (captures, moves)
    }
}
