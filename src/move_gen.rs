// Generate moves
use crate::bitboard::{Bitboard, sq_ind_to_bit, WP, BP, WR, BR, WK, BK};

impl Bitboard {
    pub fn make_move(self, from_sq_ind: usize, to_sq_ind: usize, promotion: Option<usize>) -> Bitboard {
        // Make a move, returning a new board.

        let mut new_board: Bitboard = self.clone();

        let from_bit = sq_ind_to_bit(from_sq_ind);
        let to_bit = sq_ind_to_bit(to_sq_ind);

        let from_piece = self.get_piece(from_sq_ind);
        if from_piece == None {
            panic!("No piece at from_sq_ind");
        }

        let to_piece = self.get_piece(to_sq_ind);
        if to_piece != None {
            // Capture: Remove the captured piece before moving.
            new_board.pieces[to_piece.unwrap() as usize] ^= to_bit;
            new_board.halfmove_clock = 0;
        }

        // Reset the en passant rule.
        new_board.en_passant = None;

        if from_piece.unwrap() < 2 {
            // Pawn move: Reset halfmove clock.
            new_board.halfmove_clock = 0;
            if ((to_sq_ind - from_sq_ind) as i8).abs() == 16 {
                // Pawn double move: Set en passant square.
                new_board.en_passant = Some((from_sq_ind + to_sq_ind) / 2);
            }
            // En passant
            if to_sq_ind == new_board.en_passant.unwrap() {
                // Capture the pawn.
                if new_board.w_to_move {
                    new_board.pieces[WP] ^= sq_ind_to_bit(to_sq_ind - 8);
                } else {
                    new_board.pieces[BP] ^= sq_ind_to_bit(to_sq_ind + 8);
                }
            }
        }

        // Finally, move the piece.
        new_board.pieces[from_piece.unwrap()] ^= from_bit;
        new_board.pieces[from_piece.unwrap()] ^= to_bit;

        // Promotion
        if promotion != None {
            new_board.pieces[from_piece.unwrap()] ^= to_bit;
            new_board.pieces[promotion.unwrap()] ^= to_bit;
        }

        // Castling, loss of castling rights
        if from_piece.unwrap() == WK {
            // White king
            if from_sq_ind == 4 && to_sq_ind == 6 {
                // White king-side castle
                new_board.pieces[WR] ^= sq_ind_to_bit(5);
                new_board.pieces[WR] ^= sq_ind_to_bit(7);
            } else if from_sq_ind == 4 && to_sq_ind == 2 {
                // White queen-side castle
                new_board.pieces[WR] ^= sq_ind_to_bit(3);
                new_board.pieces[WR] ^= sq_ind_to_bit(0);
            }
            new_board.w_castle_k = false;
            new_board.w_castle_q = false;
        } else if from_piece.unwrap() == BK {
            // Black king
            if from_sq_ind == 60 && to_sq_ind == 62 {
                // Black king-side castle
                new_board.pieces[BR] ^= sq_ind_to_bit(61);
                new_board.pieces[BR] ^= sq_ind_to_bit(63);
            } else if from_sq_ind == 60 && to_sq_ind == 58 {
                // Black queen-side castle
                new_board.pieces[BR] ^= sq_ind_to_bit(59);
                new_board.pieces[BR] ^= sq_ind_to_bit(56);
            }
            new_board.b_castle_k = false;
            new_board.b_castle_q = false;
        } else if from_piece.unwrap() == 6 {
            // White rook
            if from_sq_ind == 0 {
                new_board.w_castle_q = false;
            } else if from_sq_ind == 7 {
                new_board.w_castle_k = false;
            }
        } else if from_piece.unwrap() == 7 {
            // Black rook
            if from_sq_ind == 56 {
                new_board.b_castle_q = false;
            } else if from_sq_ind == 63 {
                new_board.b_castle_k = false;
            }
        }

        new_board.w_to_move = !new_board.w_to_move;
        return new_board;
    }
}