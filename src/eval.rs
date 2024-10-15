//! Pesto evaluation function module
//!
//! This module implements the Pesto evaluation function, which uses tapered evaluation
//! to interpolate between piece-square tables for opening and endgame, optimized by Texel tuning.
//!
//! TODO: Add pawn structure and king safety, possibly using a sum of over all pairs of adjacent pawns
//! and counting pieces and pawns in front of the king.

use std::cmp::min;
use crate::board_utils::flip_sq_ind_vertically;
use crate::bits::popcnt;
use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::piece_types::{PAWN, KNIGHT, ROOK, QUEEN, KING, WHITE, BLACK};
use crate::eval_constants::{MG_VALUE, MG_PESTO_TABLE, EG_VALUE, EG_PESTO_TABLE, GAMEPHASE_INC};

/// Struct representing the Pesto evaluation function
pub struct PestoEval {
    mg_table: [[[i32; 64]; 6]; 2], // [Color][PieceType][Square]
    eg_table: [[[i32; 64]; 6]; 2], // [Color][PieceType][Square]
}

impl PestoEval {
    /// Creates a new PestoEval instance
    ///
    /// Initializes the middlegame and endgame tables for all piece types
    pub fn new() -> PestoEval
    {
        let mut mg_table = [[[0; 64]; 6]; 2];
        let mut eg_table = [[[0; 64]; 6]; 2];

        // Initialize the piece square tables, flipping the board if necessary
        for p in 0..6 {
            for sq in 0..64 {
                mg_table[WHITE][p][sq] = MG_VALUE[p] + MG_PESTO_TABLE[p][flip_sq_ind_vertically(sq)];
                eg_table[WHITE][p][sq] = EG_VALUE[p] + EG_PESTO_TABLE[p][flip_sq_ind_vertically(sq)];
                mg_table[BLACK][p][sq] = MG_VALUE[p] + MG_PESTO_TABLE[p][sq];
                eg_table[BLACK][p][sq] = EG_VALUE[p] + EG_PESTO_TABLE[p][sq];
            }
        }

        PestoEval {
            mg_table,
            eg_table,
        }
    }

    /// Computes the eval (in centipawns) according to the Pesto evaluation function
    /// as well as the game phase
    ///
    /// # Arguments
    ///
    /// * `board` - A reference to the current Bitboard
    ///
    /// # Returns
    ///
    /// (eval, game_phase)
    fn eval_plus_game_phase(&self, board: &Board) -> (i32, i32) {

        let mut mg: [i32; 2] = [0, 0];
        let mut eg: [i32; 2] = [0, 0];
        let mut game_phase: i32 = 0;

        // Evaluate each piece
        for color in 0..2 {
            for piece in 0..6 {
                for sq in 0..64 {
                    if board.pieces[color][piece] & (1u64 << sq) != 0 {
                        mg[color] += self.mg_table[color][piece][sq];
                        eg[color] += self.eg_table[color][piece][sq];
                        game_phase += GAMEPHASE_INC[piece];
                    }
                }
            }
        }

        // Tapered eval
        let mg_score = mg[0] - mg[1]; // White - Black
        let eg_score = eg[0] - eg[1]; // White - Black

        let mg_phase: i32 = min(24, game_phase);
        let eg_phase: i32 = 24 - mg_phase;

        let score = (mg_score * mg_phase + eg_score * eg_phase) / 24;

        // Return score from the perspective of the side to move
        if board.w_to_move {
            (score, game_phase)
        } else {
            (-score, game_phase)
        }
    }

    /// Evaluates the current board position (in centipawns),
    /// relative to the side to move, according to the Pesto evaluation function
    ///
    /// # Arguments
    ///
    /// * `board` - A reference to the current Bitboard
    ///
    /// # Returns
    ///
    /// A tuple (i32, i32) representing the middlegame and endgame scores
    pub fn eval(&self, board: &Board) -> i32 {
        let (eval, _) = self.eval_plus_game_phase(board);
        eval
    }

    /// Evaluates and updates the board's evaluation and game phase
    ///
    /// This method computes the evaluation of the current position using the Pesto evaluation function
    /// and updates the board's evaluation and game phase. It uses a tapered evaluation that interpolates
    /// between middlegame and endgame piece-square tables based on the current game phase.
    ///
    /// # Arguments
    ///
    /// * `board` - A mutable reference to the current Bitboard
    ///
    /// # Returns
    ///
    /// An i32 representing the evaluation of the position in centipawns, relative to the side to move
    pub fn eval_update_board(&self, board: &mut Board) -> i32 {
        // Evaluate and save the eval and game phase so we can quickly compute move evals from this position
        let (score, game_phase) = self.eval_plus_game_phase(board);

        // Save eval and game phase
        board.eval = if board.w_to_move { score } else { -score };
        board.game_phase = game_phase;

        board.eval
    }

    /// Evaluates a move based on the Pesto evaluation function
    ///
    /// # Arguments
    ///
    /// * `board` - A reference to the current Bitboard
    /// * `move_gen` - A reference to the MoveGen instance
    /// * `from_sq_ind` - The starting square index of the move
    /// * `to_sq_ind` - The ending square index of the move
    ///
    /// # Returns
    ///
    /// An i32 representing the evaluation of the move in centipawns
    pub fn move_eval(&self, board: &Board, move_gen: &MoveGen, from_sq_ind: usize, to_sq_ind: usize) -> i32 {
        // Evaluate the move (in centipawns) according to the Pesto evaluation function
        // Since Pesto only depends on piece square tables, we can just use the change in value of the moved piece
        // We don't include captures here, since we will use MVV-LVA for that instead
        // We also don't include promotions, since we will also treat those separately
        // However, we rank pawn and knight forks above other non-captures
        // Note that our implementation doesn't detect a fork of two queens, since that is very rare
        // This yields the following move order:
        // captures in MVV-LVA order, promotions, pawn and knight forks, other moves in pesto order
        // Note that this is relative to the side to move

        let piece = board.get_piece(from_sq_ind).unwrap();

        // Pawn forks
        if board.w_to_move {
            if piece == (WHITE, PAWN) {
                if move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][KING] != 0 && move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN] != 0 {
                    // Fork king and queen
                    return 1000;
                } else if move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][KING] != 0 && move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK] != 0 {
                    // Fork king and rook
                    return 900;
                } else if popcnt(move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN]) == 2 {
                    // Fork two queens
                    return 850;
                } else if move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN] != 0 && move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK] != 0 {
                    // Fork queen and rook
                    return 800;
                } else if popcnt(move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK]) == 2 {
                    // Fork two rooks
                    return 700;
                } else if popcnt(move_gen.wp_capture_bitboard[to_sq_ind] & board.pieces_occ[BLACK] & !board.pieces[BLACK][PAWN]) == 2 {
                    // Fork two non-pawn pieces
                    return 600;
                }
            } else if piece == (BLACK, PAWN) {
                if move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][KING] != 0 && move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN] != 0 {
                    // Fork king and queen
                    return 1000;
                } else if move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][KING] != 0 && move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK] != 0 {
                    // Fork king and rook
                    return 900;
                } else if popcnt(move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN]) == 2 {
                    // Fork two queens
                    return 850;
                } else if move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN] != 0 && move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK] != 0 {
                    // Fork queen and rook
                    return 800;
                } else if popcnt(move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK]) == 2 {
                    // Fork two rooks
                    return 700;
                } else if popcnt(move_gen.bp_capture_bitboard[to_sq_ind] & board.pieces_occ[WHITE] & !board.pieces[WHITE][PAWN]) == 2 {
                    // Fork two non-pawn pieces
                    return 600;
                }
            }
        }

        // Knight forks
        if board.w_to_move {
            if piece == (WHITE, KNIGHT) {
                if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][KING] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN] != 0 {
                    // Fork king and queen
                    return 975;
                } else if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][KING] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK] != 0 {
                    // Fork king and rook
                    return 875;
                } else if popcnt(move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN]) == 2 {
                    // Fork two queens
                    return 825;
                } else if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][QUEEN] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK] != 0 {
                    // Fork queen and rook
                    return 775;
                } else if popcnt(move_gen.n_move_bitboard[to_sq_ind] & board.pieces[BLACK][ROOK]) == 2 {
                    // Fork two rooks
                    return 675;
                }
            } else if piece == (BLACK, KNIGHT) {
                if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][KING] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN] != 0 {
                    // Fork king and queen
                    return 975;
                } else if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][KING] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK] != 0 {
                    // Fork king and rook
                    return 875;
                } else if popcnt(move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN]) == 2 {
                    // Fork two queens
                    return 825;
                } else if move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][QUEEN] != 0 && move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK] != 0 {
                    // Fork queen and rook
                    return 775;
                } else if popcnt(move_gen.n_move_bitboard[to_sq_ind] & board.pieces[WHITE][ROOK]) == 2 {
                    // Fork two rooks
                    return 675;
                }
            }
        }

        let mut mg_score: i32 = self.mg_table[piece.0][piece.1][to_sq_ind] - self.mg_table[piece.0][piece.1][from_sq_ind];
        let eg_score: i32 = self.eg_table[piece.0][piece.1][to_sq_ind] - self.eg_table[piece.0][piece.1][from_sq_ind];

        // Castling
        if piece == (WHITE, KING) && from_sq_ind == 4 {
            if to_sq_ind == 6 { // White kingside castle
                mg_score += self.mg_table[WHITE][ROOK][5] - self.mg_table[WHITE][ROOK][7];
            } else if to_sq_ind == 2 { // White queenside castle
                mg_score += self.mg_table[WHITE][ROOK][3] - self.mg_table[WHITE][ROOK][0];
            }
        } else if piece == (BLACK, KING) && from_sq_ind == 60 {
            if to_sq_ind == 62 { // Black kingside castle
                mg_score += self.mg_table[BLACK][ROOK][61] - self.mg_table[BLACK][ROOK][63];
            } else if to_sq_ind == 58 { // Black queenside castle
                mg_score += self.mg_table[BLACK][ROOK][59] - self.mg_table[BLACK][ROOK][56];
            }
        }

        let mg_phase: i32 = min(24, board.game_phase); // Can exceed 24 in case of early promotion
        let eg_phase: i32 = 24 - mg_phase;

        (mg_score * mg_phase + eg_score * eg_phase) / 24
    }
}