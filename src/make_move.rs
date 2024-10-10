//! Module for making moves on the chess board
//!
//! This module provides functionality to apply and undo moves on the Bitboard representation of a chess position.

use crate::bitboard::{Bitboard, sq_ind_to_bit, WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK, WOCC, BOCC, OCC};
use crate::move_types::Move;

impl Bitboard {
    /// Makes a move on the board, returning a new board with the move applied
    ///
    /// This method assumes the move is legal and does not perform any legality checks.
    ///
    /// # Arguments
    ///
    /// * `the_move` - The Move to be applied to the board
    ///
    /// # Returns
    ///
    /// A new Bitboard representing the position after the move has been made
    pub fn make_move(&self, the_move: Move) -> Bitboard {
        // Make a move, returning a new board.
        // Assumes the move is legal.

        let mut new_board: Bitboard = self.clone();
        new_board.halfmove_clock += 1;

        let from_bit = sq_ind_to_bit(the_move.from);
        let to_bit = sq_ind_to_bit(the_move.to);

        let from_piece = self.get_piece(the_move.from);
        if from_piece.is_none() {
            panic!("No piece at from_sq_ind");
        }

        let to_piece = self.get_piece(the_move.to);
        if to_piece.is_some() {
            // Capture: Remove the captured piece before moving.
            new_board.pieces[to_piece.unwrap()] ^= to_bit;
            new_board.halfmove_clock = 0;
        }

        if from_piece.unwrap() < 2 {
            // En passant
            if new_board.en_passant.is_some() && the_move.to == new_board.en_passant.unwrap() {
                // Capture the pawn.
                if new_board.w_to_move {
                    new_board.pieces[BP] ^= sq_ind_to_bit(the_move.to - 8);
                } else {
                    new_board.pieces[WP] ^= sq_ind_to_bit(the_move.to + 8);
                }
            }
        }
        // Reset the en passant rule.
        new_board.en_passant = None;
        if from_piece.unwrap() < 2 {
            // Pawn move: Reset halfmove clock.
            new_board.halfmove_clock = 0;
            if ((the_move.to as i8) - (the_move.from as i8)).abs() == 16 {
                // Pawn double move: Set en passant square.
                new_board.en_passant = Some((the_move.from + the_move.to) / 2);
            }
        }

        // Finally, move the piece.
        new_board.pieces[from_piece.unwrap()] ^= from_bit;
        new_board.pieces[from_piece.unwrap()] ^= to_bit;

        // Handle promotions
        if let Some(promotion) = the_move.promotion {
            new_board.pieces[from_piece.unwrap()] ^= to_bit;
            new_board.pieces[promotion] ^= to_bit;
        }

        // Handle castling
        if from_piece.unwrap() == WK {
            // White king
            if the_move.from == 4 && the_move.to == 6 {
                // White king-side castle
                new_board.pieces[WR] ^= sq_ind_to_bit(5);
                new_board.pieces[WR] ^= sq_ind_to_bit(7);
            } else if the_move.from == 4 && the_move.to == 2 {
                // White queen-side castle
                new_board.pieces[WR] ^= sq_ind_to_bit(3);
                new_board.pieces[WR] ^= sq_ind_to_bit(0);
            }
            new_board.w_castle_k = false;
            new_board.w_castle_q = false;
        } else if from_piece.unwrap() == BK {
            // Black king
            if the_move.from == 60 && the_move.to == 62 {
                // Black king-side castle
                new_board.pieces[BR] ^= sq_ind_to_bit(61);
                new_board.pieces[BR] ^= sq_ind_to_bit(63);
            } else if the_move.from == 60 && the_move.to == 58 {
                // Black queen-side castle
                new_board.pieces[BR] ^= sq_ind_to_bit(59);
                new_board.pieces[BR] ^= sq_ind_to_bit(56);
            }
            new_board.b_castle_k = false;
            new_board.b_castle_q = false;
        } else if from_piece.unwrap() == WR {
            // White rook
            if the_move.from == 0 {
                new_board.w_castle_q = false;
            } else if the_move.from == 7 {
                new_board.w_castle_k = false;
            }
        } else if from_piece.unwrap() == BR {
            // Black rook
            if the_move.from == 56 {
                new_board.b_castle_q = false;
            } else if the_move.from == 63 {
                new_board.b_castle_k = false;
            }
        }

        new_board.w_to_move = !new_board.w_to_move;
        if new_board.w_to_move {
            new_board.fullmove_clock += 1;
        }

        // Null move: remove en passant ability and return new board
        if the_move.from == 0 && the_move.to == 0 {
            new_board.en_passant = None;
            return new_board;
        }
        new_board.pieces[WOCC] = new_board.pieces[WP] | new_board.pieces[WN] | new_board.pieces[WB] | new_board.pieces[WR] | new_board.pieces[WQ] | new_board.pieces[WK];
        new_board.pieces[BOCC] = new_board.pieces[BP] | new_board.pieces[BN] | new_board.pieces[BB] | new_board.pieces[BR] | new_board.pieces[BQ] | new_board.pieces[BK];
        new_board.pieces[OCC] = new_board.pieces[WOCC] | new_board.pieces[BOCC];

        new_board
    }
}
