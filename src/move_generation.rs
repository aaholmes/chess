//! Move generation module.
//!
//! This module provides functions for generating pseudo-legal moves for a given
//! chess position. The moves are generated using a combination of bitboards and
//! precomputed tables.
//!
//! The main entry point for move generation is the `gen_pseudo_legal_moves`
//! function, which generates all pseudo-legal moves for a given position.
//!
//! The `gen_pseudo_legal_captures` function generates only the capture moves.
//!
//! The `gen_pawn_moves`, `gen_knight_moves`, `gen_bishop_moves`, `gen_rook_moves`,
//! `gen_queen_moves`, and `gen_king_moves` functions generate moves for specific
//! piece types.

use crate::move_types::Move;
use crate::board_utils::sq_ind_to_bit;
use crate::bits::bits;
use crate::board::Board;
use crate::magic_constants::{R_MAGICS, B_MAGICS, R_BITS, B_BITS, R_MASKS, B_MASKS};
use crate::magic_bitboard::{init_pawn_moves, init_knight_moves, init_bishop_moves, init_rook_moves, init_king_moves, init_pawn_captures_promotions, append_promotions};

use crate::eval::PestoEval;
use crate::piece_types::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};

/// Represents the move generator, which generates pseudo-legal moves.
///
/// This struct contains precomputed tables and bitboards used for move generation.
pub struct MoveGen {
    /// Precomputed tables for pawn captures.
    pub wp_captures: Vec<Vec<usize>>,
    /// Precomputed tables for pawn captures.
    pub bp_captures: Vec<Vec<usize>>,
    /// Bitboards for pawn captures.
    pub wp_capture_bitboard: [u64; 64],
    /// Bitboards for pawn captures.
    pub bp_capture_bitboard: [u64; 64],
    /// Precomputed tables for pawn promotions.
    wp_promotions: Vec<Vec<usize>>,
    /// Precomputed tables for pawn promotions.
    bp_promotions: Vec<Vec<usize>>,
    /// Precomputed tables for knight moves.
    pub n_moves: Vec<Vec<usize>>,
    /// Precomputed tables for king moves.
    pub k_moves: Vec<Vec<usize>>,
    /// Bitboards for knight moves.
    pub n_move_bitboard: [u64; 64],
    /// Bitboards for king moves.
    pub k_move_bitboard: [u64; 64],
    /// Precomputed tables for pawn moves.
    wp_moves: Vec<Vec<usize>>,
    /// Precomputed tables for pawn moves.
    bp_moves: Vec<Vec<usize>>,
    /// Precomputed tables for rook moves.
    r_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    /// Precomputed tables for bishop moves.
    b_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    /// Bitboards for rook moves.
    r_move_bitboard: Vec<Vec<u64>>,
    /// Bitboards for bishop moves.
    b_move_bitboard: Vec<Vec<u64>>,
    /// Magic numbers for bishop moves.
    b_magics: [u64; 64],
    /// Magic numbers for rook moves.
    r_magics: [u64; 64],
}

impl MoveGen {
    /// Creates a new `MoveGen` instance.
    ///
    /// This function initializes the precomputed tables and bitboards used for
    /// move generation.
    ///
    /// # Returns
    ///
    /// A new `MoveGen` instance.
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
        let mut _wp: Vec<usize>;
        let mut _bp: Vec<usize>;
        let mut _wp_cap: Vec<usize>;
        let mut _bp_cap: Vec<usize>;
        let mut _wp_prom: Vec<usize>;
        let mut _bp_prom: Vec<usize>;
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

    /// Internal helper to generate all pseudo-legal moves, separated.
    fn _generate_all_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>, Vec<Move>) {
        // Returns (captures, promotions, quiet_moves)
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

        moves.append(&mut moves_knights);
        moves.append(&mut moves_bishops);
        moves.append(&mut moves_rooks);
        moves.append(&mut moves_queens);
        moves.append(&mut moves_kings);

        (captures, promotions, moves)
    }

    /// Generates all pseudo-legal moves for a given position.
    ///
    /// This function generates all pseudo-legal moves for the given position,
    /// including captures and non-captures. Promotions are included in the captures list.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves (including promotions) and non-capture moves.
    pub fn gen_pseudo_legal_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        let (mut captures, mut promotions, moves) = self._generate_all_moves(board);
        captures.append(&mut promotions); // Combine promotions with captures
        (captures, moves)
    }

    /// Generates all pseudo-legal moves, sorted for search efficiency.
    ///
    /// Generates captures (including promotions) sorted by MVV-LVA, and
    /// quiet moves sorted by a heuristic evaluation (`PestoEval::move_eval`).
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    /// * `pesto` - The evaluation function instance for move scoring.
    ///
    /// # Returns
    ///
    /// A tuple containing the sorted capture moves and sorted non-capture moves.
    pub fn gen_pseudo_legal_moves_with_evals(&self, board: &Board, pesto: &PestoEval) -> (Vec<Move>, Vec<Move>) {
        let (mut captures, mut promotions, mut moves) = self._generate_all_moves(board);
        captures.append(&mut promotions); // Combine promotions with captures

        // Sort captures by MVV-LVA (descending)
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

        // Sort quiet moves by heuristic eval change (descending)
        moves.sort_unstable_by_key(|m| -pesto.move_eval(board, self, m.from, m.to));

        (captures, moves)
    }

    /// Generates only the capture moves for a given position.
    ///
    /// This function generates only the capture moves for the given position.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A vector of capture moves.
    /// Generates only the capture and promotion moves, sorted by MVV-LVA.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A vector of capture and promotion moves, sorted by MVV-LVA (descending).
    pub fn gen_pseudo_legal_captures(&self, board: &Board) -> Vec<Move> {
        let (mut captures, mut promotions, _moves) = self._generate_all_moves(board);
        captures.append(&mut promotions); // Combine promotions with captures

        // Sort captures by MVV-LVA (descending)
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

        captures
    }

    pub fn mvv_lva(&self, board: &Board, from_sq_ind: usize, to_sq_ind: usize) -> i32 {
        // Return the MVV-LVA score for a capture move.
        // To enable sorting by MVV, then by LVA, we return the score as 10 * victim - attacker,
        // where value is 012345 for kpnbrq
        if board.get_piece(to_sq_ind).is_none() {
            return 0;
        }
        let victim = board.get_piece(to_sq_ind).unwrap().1;
        let attacker = board.get_piece(from_sq_ind).unwrap().1;
        10 * victim as i32 - attacker as i32
    }

    /// Generates moves for a pawn on a specific square.
    ///
    /// This function generates moves for a pawn on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    /// * `from_sq_ind` - The index of the square the pawn is on (0-63).
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the pawn.
    pub fn gen_pawn_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>, Vec<Move>) {
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut promotions: Vec<Move> = Vec::new();

        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WHITE][PAWN]) {
                let is_promotion_rank = from_sq_ind > 47 && from_sq_ind < 56;

                // Handle captures and en passant
                for to_sq_ind in &self.wp_captures[from_sq_ind] {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind as u8) {
                        if is_promotion_rank {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }

                // Handle regular moves and promotions
                for to_sq_ind in &self.wp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        if is_promotion_rank {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else if from_sq_ind > 7 && from_sq_ind < 16 {
                            // Double pawn push
                            if (board.pieces_occ[BLACK] + board.pieces_occ[WHITE]) & (1u64 << (from_sq_ind + 8)) == 0 {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        } else {
                            moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
            }
        } else {
            // Black to move (similar logic, but for black pawns)
            for from_sq_ind in bits(&board.pieces[BLACK][PAWN]) {
                let is_promotion_rank = from_sq_ind > 7 && from_sq_ind < 16;

                // Handle captures and en passant
                for to_sq_ind in &self.bp_captures[from_sq_ind] {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind as u8) {
                        if is_promotion_rank {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }

                // Handle regular moves and promotions
                for to_sq_ind in &self.bp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        if is_promotion_rank {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else if from_sq_ind > 47 && from_sq_ind < 56 {
                            // Double pawn push
                            if (board.pieces_occ[WHITE] + board.pieces_occ[BLACK]) & (1u64 << (from_sq_ind - 8)) == 0 {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        } else {
                            moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
            }
        }

        (captures, promotions, moves)
    }

    /// Generates moves for a knight on a specific square.
    ///
    /// This function generates moves for a knight on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the knight.
    fn gen_knight_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible knight moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        let mut moves: Vec<Move> = Vec::with_capacity(16);
        let mut captures: Vec<Move> = Vec::with_capacity(16);
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WHITE][KNIGHT]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BLACK][KNIGHT]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    /// Generates moves for a king on a specific square.
    ///
    /// This function generates moves for a king on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the king.
    fn gen_king_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible king moves for the current position.
        // For castling, checks whether in check and whether the king moves through check.
        // Returns a vector of captures and a vector of non-captures.
        let mut moves: Vec<Move> = Vec::with_capacity(10);
        let mut captures: Vec<Move> = Vec::with_capacity(8);
        if board.w_to_move {
            // White to move
            if board.castling_rights.white_kingside {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WHITE][ROOK] & (1u64 << 7) != 0 &&
                    (board.pieces_occ[WHITE] | board.pieces_occ[BLACK]) & ((1u64 << 5) | (1u64 << 6)) == 0 &&
                    !board.is_square_attacked(4, false, self) &&
                    !board.is_square_attacked(5, false, self) &&
                    !board.is_square_attacked(6, false, self) {
                    moves.push(Move::new(4, 6, None));
                }
            }
            if board.castling_rights.white_queenside {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WHITE][ROOK] & (1u64 << 0) != 0 &&
                    (board.pieces_occ[WHITE] | board.pieces_occ[BLACK]) & ((1u64 << 1) | (1u64 << 2) | (1u64 << 3)) == 0 &&
                    !board.is_square_attacked(4, false, self) &&
                    !board.is_square_attacked(3, false, self) &&
                    !board.is_square_attacked(2, false, self) {
                    moves.push(Move::new(4, 2, None));
                }
            }
            for from_sq_ind in bits(&board.pieces[WHITE][KING]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            if board.castling_rights.black_kingside {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BLACK][ROOK] & (1u64 << 63) != 0 &&
                    (board.pieces_occ[WHITE] | board.pieces_occ[BLACK]) & ((1u64 << 61) | (1u64 << 62)) == 0 &&
                    !board.is_square_attacked(60, true, self) &&
                    !board.is_square_attacked(61, true, self) &&
                    !board.is_square_attacked(62, true, self) {
                    moves.push(Move::new(60, 62, None));
                }
            }
            if board.castling_rights.black_queenside {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BLACK][ROOK] & (1u64 << 56) != 0 &&
                    (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59)) == 0 &&
                    !board.is_square_attacked(60, true, self) &&
                    !board.is_square_attacked(59, true, self) &&
                    !board.is_square_attacked(58, true, self) {
                    moves.push(Move::new(60, 58, None));
                }
            }
            for from_sq_ind in bits(&board.pieces[BLACK][KING]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    /// Generates moves for a rook on a specific square.
    ///
    /// This function generates moves for a rook on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the rook.
    fn gen_rook_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible rook moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WHITE][ROOK]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BLACK][ROOK]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    pub fn gen_bishop_potential_captures(&self, board: &Board, from_sq_ind: usize) -> u64 {
        // Generate potential bishop captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & B_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.b_move_bitboard[from_sq_ind][key]
    }

    pub fn gen_rook_potential_captures(&self, board: &Board, from_sq_ind: usize) -> u64 {
        // Generate potential rook captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & R_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.r_move_bitboard[from_sq_ind][key]
    }

    /// Generates moves for a bishop on a specific square.
    ///
    /// This function generates moves for a bishop on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the bishop.
    fn gen_bishop_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible bishop moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WHITE][BISHOP]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BLACK][BISHOP]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    /// Generates moves for a queen on a specific square.
    ///
    /// This function generates moves for a queen on the given square, including
    /// captures and non-captures.
    ///
    /// # Arguments
    ///
    /// * `board` - The current chess position.
    ///
    /// # Returns
    ///
    /// A tuple containing the capture moves and non-capture moves for the queen.
    fn gen_queen_moves(&self, board: &Board) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible queen moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WHITE][QUEEN]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BLACK][QUEEN]) {
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = (board.pieces_occ[BLACK] | board.pieces_occ[WHITE]) & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces_occ[WHITE] & (1u64 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces_occ[BLACK] & (1u64 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }
}