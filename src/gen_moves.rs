// Generate all possible moves for a given position
// We divide it up into two functions: one for generating moves for a single piece, and one for generating moves for all pieces.
// The latter will call the former for each piece.
// For this version, we generate moves in the following order:
// 1. Captures/promotions in MVV-LVA order
// 2. Knight forks
// 3. All other moves, ordered by change in pesto eval (precomputed)
// I think this is relatively easy to do, and may even be close to optimal, even if we later change the eval function, e.g. to NNUE.
// We can also use quiescence search to only generate captures and promotions.
// For capture move orders, we should use 10*victim_value - attacker_value, where PNBRQK = 123456 e.g., PxQ = 10*5 - 1 = 49, KxN = 10*3 - 6 = 24, etc.
// For promotions, we should use 10*promoted_piece_value - attacker_value, so Q promotion = 10*5 - 1 = 49, etc.
// We want to use two heaps, one for captures and one for non-captures, with each element of the heap being a vector of sorted moves.
// Note that non-captures can be pre-sorted, but captures require the piece value of the captured piece and so they have to be internally sorted, which may be just as fast as sorting the entire vector.
// Note also that the pesto eval has 25 game modes, ranging from opening to endgame, so our non-capture move ordering should be different for each game mode.


use crate::bitboard::{Bitboard, sq_ind_to_bit, WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK, WOCC, BOCC, OCC};
use crate::bits::bits;
use crate::magic_constants::{R_MAGICS, B_MAGICS, R_BITS, B_BITS, R_MASKS, B_MASKS};
use rand;
use crate::eval::PestoEval;

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

// Struct representing a move
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<usize>
}

impl Move {
    pub fn new(from: usize, to: usize, promotion: Option<usize>) -> Move {
        // New move
        Move {
            from,
            to,
            promotion
        }
    }

    pub fn null() -> Move {
        // Null move
        Move {
            from: 0,
            to: 0,
            promotion: None
        }
    }
}

// Struct representing the move generator, that generates pseudo-legal moves
pub struct MoveGen {
    pub wp_captures: Vec<Vec<usize>>,
    pub bp_captures: Vec<Vec<usize>>,
    pub wp_capture_bitboard: [u64; 64],
    pub bp_capture_bitboard: [u64; 64],
    wp_promotions: Vec<Vec<usize>>,
    bp_promotions: Vec<Vec<usize>>,
    pub n_moves: Vec<Vec<usize>>,
    pub k_moves: Vec<Vec<usize>>,
    pub n_move_bitboard: [u64; 64],
    pub k_move_bitboard: [u64; 64],
    wp_moves: Vec<Vec<usize>>,
    bp_moves: Vec<Vec<usize>>,
    r_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    b_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    r_move_bitboard: Vec<Vec<u64>>,
    b_move_bitboard: Vec<Vec<u64>>,
    b_magics: [u64; 64],
    r_magics: [u64; 64],
}

fn find_magic_numbers() -> ([u64; 64], [u64; 64]) {
    // Find magic numbers for magic bitboards.
    // (blockers * magic) >> (64 - n_bits) should give a unique key for each blocker combination.

    let mut blockers: u64;
    let mut blocker_squares: Vec<usize> = Vec::new();
    let mut key: usize;
    let mut keys: Vec<usize> = Vec::new();
    let mut magic: u64;
    let mut b_magics: [u64; 64] = [0; 64];
    let mut r_magics: [u64; 64] = [0; 64];
    for is_bishop in [true, false].iter() {
        for from_sq_ind in 0..64 {
            // Mask blockers
            if *is_bishop {
                blockers = B_MASKS[from_sq_ind];
            } else {
                blockers = R_MASKS[from_sq_ind];
            }
            blocker_squares.clear();
            for i in bits(&blockers) {
                blocker_squares.push(i);
            }
            for _i_magic in 0..1000000 {
                magic = rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>();
                // Iterate over all possible blocker combinations
                // Require that they are all unique
                keys.clear();
                for blocker_ind in 0..(1 << blocker_squares.len()) {
                    if *is_bishop {
                        blockers = B_MASKS[from_sq_ind];
                    } else {
                        blockers = R_MASKS[from_sq_ind];
                    }
                    for i in 0..blocker_squares.len() {
                        if (blocker_ind & (1 << i)) != 0 {
                            blockers &= !sq_ind_to_bit(blocker_squares[i]);
                        }
                    }
                    if *is_bishop {
                        key = ((blockers * magic) >> (64 - B_BITS[from_sq_ind])) as usize;
                    } else {
                        key = ((blockers * magic) >> (64 - R_BITS[from_sq_ind])) as usize;
                    }
                    if keys.contains(&key) {
                        break;
                    } else {
                        keys.push(key);
                    }
                }
                if *is_bishop {
                    if keys.len() == (1 << B_BITS[from_sq_ind]) {
                        println!("Found bishop magic number for square {} with {} bits: {}", from_sq_ind, B_BITS[from_sq_ind], magic);
                        b_magics[from_sq_ind] = magic;
                        break;
                    }
                } else {
                    if keys.len() == (1 << R_BITS[from_sq_ind]) {
                        println!("Found rook magic number for square {} with {} bits: {}", from_sq_ind, R_BITS[from_sq_ind], magic);
                        r_magics[from_sq_ind] = magic;
                        break;
                    }
                }
            }
        }
    }
    (b_magics, r_magics)
}

fn init_king_moves(from_sq_ind: usize) -> Vec<usize> {
    // Initialize the king moves for a given square, ignoring castling
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut out: Vec<usize> = Vec::new();
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

fn init_pawn_captures_promotions(from_sq_ind: usize) -> (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>) {
    // Initialize the pawn captures and promotions for a given square.
    // Separate for white and black.
    let from_bit: u64 = sq_ind_to_bit(from_sq_ind);
    let mut w_cap: Vec<usize> = Vec::new();
    let mut b_cap: Vec<usize> = Vec::new();
    let mut w_prom: Vec<usize> = Vec::new();
    let mut b_prom: Vec<usize> = Vec::new();
    if from_bit & NOT_1_RANK & NOT_8_RANK != 0 {
        if from_bit & NOT_A_FILE != 0 {
            w_cap.push(from_sq_ind + 7);
        }
        if from_bit & NOT_H_FILE != 0 {
            w_cap.push(from_sq_ind + 9);
        }
        if from_bit & RANK_7 != 0 {
            w_prom.push(from_sq_ind + 8);
        }
        if from_bit & NOT_A_FILE != 0 {
            b_cap.push(from_sq_ind - 9);
        }
        if from_bit & NOT_H_FILE != 0 {
            b_cap.push(from_sq_ind - 7);
        }
        if from_bit & RANK_2 != 0 {
            b_prom.push(from_sq_ind - 8);
        }
    }
    (w_cap, w_prom, b_cap, b_prom)
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

fn init_bishop_moves(b_magics: [u64; 64]) -> (Vec<Vec<(Vec<usize>, Vec<usize>)>>, Vec<Vec<u64>>) {
    // Initialize the bishop moves for each square and blocker combination.
    // Uses magic bitboards.
    // Returns, for each square and key, both vectors of captures and moves, as well as a bitboard of potential captures (for detecting check).
    let mut out1: Vec<Vec<(Vec<usize>, Vec<usize>)>> = Vec::new();
    let mut out2: Vec<Vec<u64>> = Vec::new();
    let mut blockers: u64;
    let mut key: usize;
    let mut blocker_squares: Vec<usize> = Vec::new();
    for from_sq_ind in 0..64 {
        out1.push(vec![(vec![], vec![]); 4096]);
        out2.push(vec![0; 4096]);
        // Mask blockers
        blockers = B_MASKS[from_sq_ind];
        blocker_squares.clear();
        for i in bits(&blockers) {
            blocker_squares.push(i);
        }

        // Iterate over all possible blocker combinations
        for blocker_ind in 0..(1 << blocker_squares.len()) {
            blockers = B_MASKS[from_sq_ind];
            for i in 0..blocker_squares.len() {
                if (blocker_ind & (1 << i)) != 0 {
                    blockers &= !sq_ind_to_bit(blocker_squares[i]);
                }
            }

            // Generate the key using a multiplication and right shift
            key = ((blockers.wrapping_mul(b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

            // Assign the captures and moves for this blocker combination
            out1[from_sq_ind][key] = bishop_attacks(from_sq_ind, blockers);
            for i in out1[from_sq_ind][key].0.iter_mut() {
                out2[from_sq_ind][key] |= sq_ind_to_bit(*i);
            }
        }
    }
    (out1, out2)
}

fn init_rook_moves(r_magics: [u64; 64]) -> (Vec<Vec<(Vec<usize>, Vec<usize>)>>, Vec<Vec<u64>>) {
    // Initialize the rook moves for each square and blocker combination.
    // Uses magic bitboards.
    // Returns, for each square and key, both vectors of captures and moves, as well as a bitboard of potential captures (for detecting check).
    let mut out1: Vec<Vec<(Vec<usize>, Vec<usize>)>> = Vec::new();
    let mut out2: Vec<Vec<u64>> = Vec::new();
    let mut blockers: u64;
    let mut key: usize;
    let mut blocker_squares: Vec<usize> = Vec::new();
    for from_sq_ind in 0..64 {
        out1.push(vec![(vec![], vec![]); 4096]);
        out2.push(vec![0; 4096]);
        // Mask blockers
        blockers = R_MASKS[from_sq_ind];
        blocker_squares.clear();
        for i in bits(&blockers) {
            blocker_squares.push(i);
        }

        // Iterate over all possible blocker combinations
        for blocker_ind in 0..(1 << blocker_squares.len()) {
            blockers = R_MASKS[from_sq_ind];
            for i in 0..blocker_squares.len() {
                if (blocker_ind & (1 << i)) != 0 {
                    blockers &= !sq_ind_to_bit(blocker_squares[i]);
                }
            }

            // Generate the key using a multiplication and right shift
            key = ((blockers.wrapping_mul(r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

            // Assign the captures and moves for this blocker combination
            out1[from_sq_ind][key] = rook_attacks(from_sq_ind, blockers);
            for i in out1[from_sq_ind][key].0.iter_mut() {
                out2[from_sq_ind][key] |= sq_ind_to_bit(*i);
            }
        }
    }
    (out1, out2)
}

fn rook_attacks(sq: usize, block: u64) -> (Vec<usize>, Vec<usize>) {
    // Return the attacks for a rook on a given square, given a blocking mask.
    // Return the captures and moves separately
    // Following the tradition of using blocking masks, this routine has two quirks:
    // 1. Captures can include capturing own pieces
    // 2. For unblocked moves to end of the board, for now we store that as both a capture and a move
    // These are both because of the way the blocking masks are defined.
    let rk = sq / 8;
    let fl = sq % 8;
    let mut captures: Vec<usize> = Vec::new();
    let mut moves: Vec<usize> = Vec::new();
    for r in rk + 1 .. 8 {
        if r == 7 {
            captures.push(fl + r * 8);
            moves.push(fl + r * 8);
        } else {
            if (block & (1 << (fl + r * 8))) != 0 {
                captures.push(fl + r * 8);
                break;
            } else {
                moves.push(fl + r * 8);
            }
        }
    }
    for r in (0 .. rk).rev() {
        if r == 0 {
            captures.push(fl + r * 8);
            moves.push(fl + r * 8);
        } else {
            if (block & (1 << (fl + r * 8))) != 0 {
                captures.push(fl + r * 8);
                break;
            } else {
                moves.push(fl + r * 8);
            }
        }
    }
    for f in fl + 1 .. 8 {
        if f == 7 {
            captures.push(f + rk * 8);
            moves.push(f + rk * 8);
        } else {
            if (block & (1 << (f + rk * 8))) != 0 {
                captures.push(f + rk * 8);
                break;
            } else {
                moves.push(f + rk * 8);
            }
        }
    }
    for f in (0 .. fl).rev() {
        if f == 0 {
            captures.push(f + rk * 8);
            moves.push(f + rk * 8);
        } else {
            if (block & (1 << (f + rk * 8))) != 0 {
                captures.push(f + rk * 8);
                break;
            } else {
                moves.push(f + rk * 8);
            }
        }
    }
    (captures, moves)
}

fn bishop_attacks(sq: usize, block: u64) -> (Vec<usize>, Vec<usize>) {
    // Return the attacks for a bishop on a given square, given a blocking mask.
    // Return the captures and moves separately
    // Following the tradition of using blocking masks, this routine has two quirks:
    // 1. Captures can include capturing own pieces
    // 2. For unblocked moves to end of the board, for now we store that as both a capture and a move
    // These are both because of the way the blocking masks are defined.
    let rk = sq / 8;
    let fl = sq % 8;
    let mut f: usize;
    let mut captures: Vec<usize> = Vec::new();
    let mut moves: Vec<usize> = Vec::new();
    if rk < 7 && fl < 7 {
        for r in rk + 1..8 {
            f = fl + r - rk;
            if r == 7 || f == 7 {
                captures.push(f + r * 8);
                moves.push(f + r * 8);
                break;
            } else {
                if (block & (1 << (f + r * 8))) != 0 {
                    captures.push(f + r * 8);
                    break;
                } else {
                    moves.push(f + r * 8);
                }
            }
        }
    }
    if rk < 7 && fl > 0 {
        for r in rk + 1 .. 8 {
            f = fl + rk - r;
            if r == 7 || f == 0 {
                captures.push(f + r * 8);
                moves.push(f + r * 8);
                break;
            } else {
                if (block & (1 << (f + r * 8))) != 0 {
                    captures.push(f + r * 8);
                    break;
                } else {
                    moves.push(f + r * 8);
                }
            }
        }
    }
    if rk > 0 && fl > 0 {
        for r in (0 .. rk).rev() {
            f = fl + r - rk;
            if r == 0 || f == 0 {
                captures.push(f + r * 8);
                moves.push(f + r * 8);
                break;
            } else {
                if (block & (1 << (f + r * 8))) != 0 {
                    captures.push(f + r * 8);
                    break;
                } else {
                    moves.push(f + r * 8);
                }
            }
        }
    }
    if rk > 0 && fl < 7 {
        for r in (0..rk).rev() {
            f = fl + rk - r;
            if r == 0 || f == 7 {
                captures.push(f + r * 8);
                moves.push(f + r * 8);
                break;
            } else {
                if (block & (1 << (f + r * 8))) != 0 {
                    captures.push(f + r * 8);
                    break;
                } else {
                    moves.push(f + r * 8);
                }
            }
        }
    }
    (captures, moves)
}

fn append_promotions(promotions: &mut Vec<Move>, from_sq_ind: usize, to_sq_ind: &usize, w_to_move: bool) {
    if w_to_move {
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(WQ)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(WR)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(WN)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(WB)));
    } else {
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(BQ)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(BR)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(BN)));
        promotions.push(Move::new(from_sq_ind, *to_sq_ind, Some(BB)));
    }
}

impl MoveGen {
    pub fn new() -> MoveGen {
        // Initialize the move generator by creating the iterators for Pawn, Knight, and King moves.
        let mut wp_captures: Vec<Vec<usize>> = Vec::new();
        let mut bp_captures: Vec<Vec<usize>> = Vec::new();
        let mut wp_capture_bitboard: [u64; 64] = [0; 64];
        let mut bp_capture_bitboard: [u64; 64] = [0; 64];
        let mut wp_promotions: Vec<Vec<usize>> = Vec::new();
        let mut bp_promotions: Vec<Vec<usize>> = Vec::new();
        let mut n_moves: Vec<Vec<usize>> = Vec::new();
        let mut k_moves: Vec<Vec<usize>> = Vec::new();
        let mut n_move_bitboard: [u64; 64] = [0; 64];
        let mut k_move_bitboard: [u64; 64] = [0; 64];
        let mut wp_moves: Vec<Vec<usize>> = Vec::new();
        let mut bp_moves: Vec<Vec<usize>> = Vec::new();
        let mut wp: Vec<usize>;
        let mut bp: Vec<usize>;
        let mut wp_cap: Vec<usize>;
        let mut bp_cap: Vec<usize>;
        let mut wp_prom: Vec<usize>;
        let mut bp_prom: Vec<usize>;
        for from_sq_ind in 0..64 {
            let (wp_cap, wp_prom, bp_cap, bp_prom) = init_pawn_captures_promotions(from_sq_ind);
            wp_captures.push(wp_cap.clone());
            bp_captures.push(bp_cap.clone());
            for i in &wp_captures[from_sq_ind] {
                wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            // Also add pawn moves from pawns on the first rank, as this is used in reverse to determine checks by pawns
            if from_sq_ind < 8 {
                if from_sq_ind < 7 {
                    wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind + 9);
                }
                if from_sq_ind > 0 {
                    wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind + 7);
                }
            }
            for i in &bp_captures[from_sq_ind] {
                bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            // Also add pawn moves from pawns on the first rank, as this is used in reverse to determine checks by pawns
            if from_sq_ind > 55 {
                if from_sq_ind > 56 {
                    bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind - 9);
                }
                if from_sq_ind < 63 {
                    bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind - 7);
                }
            }
            wp_promotions.push(wp_prom.clone());
            bp_promotions.push(bp_prom.clone());
            n_moves.push(init_knight_moves(from_sq_ind));
            k_moves.push(init_king_moves(from_sq_ind));
            for i in &n_moves[from_sq_ind] {
                n_move_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            for i in &k_moves[from_sq_ind] {
                k_move_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            let (wp, bp) = init_pawn_moves(from_sq_ind);
            wp_moves.push(wp.clone());
            bp_moves.push(bp.clone());
        }
        let mut move_gen: MoveGen = MoveGen {
            wp_captures,
            bp_captures,
            wp_capture_bitboard,
            bp_capture_bitboard,
            wp_promotions,
            bp_promotions,
            n_moves,
            k_moves,
            n_move_bitboard,
            k_move_bitboard,
            wp_moves,
            bp_moves,
            r_moves: vec![],
            b_moves: vec![],
            r_move_bitboard: vec![],
            b_move_bitboard: vec![],
            b_magics: [0; 64],
            r_magics: [0; 64],
        };
        // Generate magic numbers for sliding pieces
        // let (b_magics, r_magics) = find_magic_numbers();
        move_gen.b_magics = B_MAGICS;
        move_gen.r_magics = R_MAGICS;
        let (b_moves, b_move_bitboard) = init_bishop_moves(move_gen.b_magics);
        move_gen.b_moves = b_moves;
        move_gen.b_move_bitboard = b_move_bitboard;
        let (r_moves, r_move_bitboard) = init_rook_moves(move_gen.r_magics);
        move_gen.r_moves = r_moves;
        move_gen.r_move_bitboard = r_move_bitboard;
        move_gen
    }

    pub fn gen_pseudo_legal_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all pseudo-legal moves for the current position, i.e., these moves may move into check.
        // Elsewhere we need to check for legality and perform move ordering.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, promotion).
        // Includes promotions as captures here
        let (mut captures, mut promotions, mut moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, mut moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, mut moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, mut moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, mut moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, mut moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);
        moves.append(&mut moves_knights);
        moves.append(&mut moves_bishops);
        moves.append(&mut moves_rooks);
        moves.append(&mut moves_queens);
        moves.append(&mut moves_kings);
        (captures, moves)
    }

    pub fn gen_pseudo_legal_moves_with_evals(&self, board: &mut Bitboard, pesto: &PestoEval) -> (Vec<Move>, Vec<Move>) {
        // Generate all pseudo-legal moves for the current position, i.e., these moves may move into check.
        // Elsewhere we need to check for legality and perform move ordering.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, promotion).
        // Includes promotions as captures here
        let (mut captures, mut promotions, mut moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, mut moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, mut moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, mut moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, mut moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, mut moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);
        moves.append(&mut moves_knights);
        moves.append(&mut moves_bishops);
        moves.append(&mut moves_rooks);
        moves.append(&mut moves_queens);
        moves.append(&mut moves_kings);

        // Here let's sort captures by MVV-LVA
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

        // Also sort moves by pesto eval change
        pesto.eval_update_board(board);
        moves.sort_unstable_by_key(|m| -pesto.move_eval(board, self, m.from, m.to));

        (captures, moves)
    }
    pub fn gen_pseudo_legal_captures(&self, board: &Bitboard) -> Vec<Move> {
        // Same as above, but only generate captures
        let (mut captures, mut promotions, _moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, _moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, _moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, _moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, _moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, _moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);

        // Here let's sort captures by MVV-LVA
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

    captures
    }

    fn mvv_lva(&self, board: &Bitboard, from_sq_ind: usize, to_sq_ind: usize) -> i32 {
        // Return the MVV-LVA score for a capture move.
        // To enable sorting by MVV, then by LVA, we return the score as 10 * victim - attacker,
        // where value is 012345 for kpnbrq
        if board.get_piece(to_sq_ind) == None {
            return 0;
        }
        let victim = board.get_piece(to_sq_ind).unwrap() / 2;
        let attacker = board.get_piece(from_sq_ind).unwrap() / 2;
        10 * victim as i32 - attacker as i32
    }

    fn gen_pawn_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>, Vec<Move>) {
        // Generate all possible pawn moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Treats promotions as captures.
        // Lists promotions in the following order: queen, rook, knight, bishop, since bishop promotions are very rare.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut promotions: Vec<Move> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WP]) {
                for to_sq_ind in &self.wp_captures[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.wp_promotions[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                    }
                }
                for to_sq_ind in &self.wp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            if from_sq_ind > 7 && from_sq_ind < 16 {
                                if board.pieces[OCC] & (1 << (from_sq_ind + 8)) == 0 {
                                    moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                                }
                            } else {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        }
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BP]) {
                for to_sq_ind in &self.bp_captures[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.bp_promotions[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                    }
                }
                for to_sq_ind in &self.bp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind) == None {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            if from_sq_ind > 47 && from_sq_ind < 56 {
                                if board.pieces[OCC] & (1 << (from_sq_ind - 8)) == 0 {
                                    moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                                }
                            } else {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        }
                    }
                }
            }
        }
        (captures, promotions, moves)
    }

    fn gen_knight_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible knight moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_king_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible king moves for the current position.
        // For castling, checks whether in check and whether the king moves through check.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        if board.w_to_move {
            // White to move
            if board.w_castle_k {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WR] & (1 << 7) != 0 {
                    if board.pieces[OCC] & ((1 << 5) | (1 << 6)) == 0 {
                        if !board.is_square_attacked(4, false, self) && !board.is_square_attacked(5, false, self) && !board.is_square_attacked(6, false, self) {
                            moves.push(Move::new(4, 6, None));
                        }
                    }
                }
            }
            if board.w_castle_q {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WR] & (1 << 0) != 0 {
                    if board.pieces[OCC] & ((1 << 1) | (1 << 2) | (1 << 3)) == 0 {
                        if !board.is_square_attacked(4, false, self) && !board.is_square_attacked(3, false, self) && !board.is_square_attacked(2, false, self) {
                            moves.push(Move::new(4, 2, None));
                        }
                    }
                }
            }
            for from_sq_ind in bits(&board.pieces[WK]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            if board.b_castle_k {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BR] & (1 << 63) != 0 {
                    if board.pieces[OCC] & ((1 << 61) | (1 << 62)) == 0 {
                        if !board.is_square_attacked(60, true, self) && !board.is_square_attacked(61, true, self) && !board.is_square_attacked(62, true, self) {
                            moves.push(Move::new(60, 62, None));
                        }
                    }
                }
            }
            if board.b_castle_q {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BR] & (1 << 56) != 0 {
                    if board.pieces[OCC] & ((1 << 57) | (1 << 58) | (1 << 59)) == 0 {
                        if !board.is_square_attacked(60, true, self) && !board.is_square_attacked(59, true, self) && !board.is_square_attacked(58, true, self) {
                            moves.push(Move::new(60, 58, None));
                        }
                    }
                }
            }
            for from_sq_ind in bits(&board.pieces[BK]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_rook_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible rook moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WR]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BR]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    pub fn gen_bishop_potential_captures(&self, board: &Bitboard, from_sq_ind: usize) -> u64 {
        // Generate potential bishop captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = board.pieces[OCC] & B_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.b_move_bitboard[from_sq_ind][key]
    }

    pub fn gen_rook_potential_captures(&self, board: &Bitboard, from_sq_ind: usize) -> u64 {
        // Generate potential rook captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = board.pieces[OCC] & R_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.r_move_bitboard[from_sq_ind][key]
    }

    fn gen_bishop_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible bishop moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WB]) {
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BB]) {
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_queen_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible queen moves for the current position.
        // Returns a vector of captures and a vector of non-captures, both in the form tuples (from_sq_ind, to_sq_ind, None).
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WQ]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BQ]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }
}