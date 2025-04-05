//! Helper functions and common imports for evaluation tests.

// Allow dead code for helper function not used in every test file
#![allow(dead_code)]

use kingfisher::bits;
use kingfisher::board::Board;
use kingfisher::board_utils;
use kingfisher::eval::PestoEval;
use kingfisher::eval_constants::{
    CASTLING_RIGHTS_BONUS, DOUBLED_ROOKS_ON_SEVENTH_BONUS, ISOLATED_PAWN_PENALTY,
    KING_SAFETY_PAWN_SHIELD_BONUS, MOBILE_PAWN_DUO_BONUS_EG, MOBILE_PAWN_DUO_BONUS_MG,
    PASSED_PAWN_BONUS_EG, PASSED_PAWN_BONUS_MG, PAWN_CHAIN_BONUS, PAWN_DUO_BONUS,
    ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS, ROOK_BEHIND_PASSED_PAWN_BONUS, ROOK_HALF_OPEN_FILE_BONUS,
    ROOK_ON_SEVENTH_BONUS, ROOK_OPEN_FILE_BONUS, TWO_BISHOPS_BONUS,
};
use kingfisher::piece_types::{BISHOP, BLACK, KING, PAWN, ROOK, WHITE};

// Helper to get the raw MG/EG scores *before* tapering and side-to-move adjustment
// This allows testing the contribution of individual terms more easily.
// Modified to use accessor methods instead of private fields
pub fn get_raw_scores(evaluator: &PestoEval, board: &Board) -> (i32, i32) {
    let mut mg = [0, 0];
    let mut eg = [0, 0];

    // Base Pesto PST scores
    for color in [WHITE, BLACK] {
        for piece in 0..6 {
            let mut piece_bb = board.get_piece_bitboard(color, piece);
            while piece_bb != 0 {
                let sq = piece_bb.trailing_zeros() as usize;
                // Access tables through accessor methods
                mg[color] += evaluator.get_mg_score(color, piece, sq);
                eg[color] += evaluator.get_eg_score(color, piece, sq);
                piece_bb &= piece_bb - 1;
            }
        }
    }

    // --- Add Bonus Terms ---
    for color in [WHITE, BLACK] {
        let enemy_color = 1 - color;

        // 1. Two Bishops Bonus
        if bits::popcnt(board.get_piece_bitboard(color, BISHOP)) >= 2 {
            mg[color] += TWO_BISHOPS_BONUS[0];
            eg[color] += TWO_BISHOPS_BONUS[1];
        }

        // --- Pawn Structure ---
        let friendly_pawns = board.get_piece_bitboard(color, PAWN);
        let enemy_pawns = board.get_piece_bitboard(enemy_color, PAWN);
        let mut chain_bonus_mg = 0;
        let mut chain_bonus_eg = 0;
        let mut duo_bonus_mg = 0;
        let mut duo_bonus_eg = 0;

        for sq in bits::bits(&friendly_pawns) {
            let file = board_utils::sq_to_file(sq);

            // 2. Passed Pawn Bonus
            let passed_mask = board_utils::get_passed_pawn_mask(color, sq);
            if (passed_mask & enemy_pawns) == 0 {
                let rank = board_utils::sq_to_rank(sq);
                let bonus_rank = if color == WHITE { rank } else { 7 - rank };
                mg[color] += PASSED_PAWN_BONUS_MG[bonus_rank];
                eg[color] += PASSED_PAWN_BONUS_EG[bonus_rank];
            }

            // 4. Isolated Pawn Penalty
            let adjacent_mask = board_utils::get_adjacent_files_mask(sq);
            if (adjacent_mask & friendly_pawns) == 0 {
                mg[color] += ISOLATED_PAWN_PENALTY[0];
                eg[color] += ISOLATED_PAWN_PENALTY[1];
            }

            // 5. Pawn Chain Bonus (Diagonal defense)
            let (defend1_sq_opt, defend2_sq_opt) = if color == WHITE {
                (sq.checked_sub(9), sq.checked_sub(7)) // Check squares diagonally behind (SW, SE)
            } else {
                (sq.checked_add(7), sq.checked_add(9)) // Check squares diagonally behind (NW, NE)
            };

            if let Some(defend1_sq) = defend1_sq_opt {
                if defend1_sq < 64
                    && (sq % 2 == defend1_sq % 2)
                    && (friendly_pawns & board_utils::sq_ind_to_bit(defend1_sq) != 0)
                {
                    chain_bonus_mg += PAWN_CHAIN_BONUS[0];
                    chain_bonus_eg += PAWN_CHAIN_BONUS[1];
                }
            }
            if let Some(defend2_sq) = defend2_sq_opt {
                if defend2_sq < 64
                    && (sq % 2 == defend2_sq % 2)
                    && (friendly_pawns & board_utils::sq_ind_to_bit(defend2_sq) != 0)
                {
                    chain_bonus_mg += PAWN_CHAIN_BONUS[0];
                    chain_bonus_eg += PAWN_CHAIN_BONUS[1];
                }
            }

            // 6. Pawn Duo Bonus (Side-by-side) - Check only right neighbor to avoid double counting
            if file < 7 {
                let neighbor_sq = sq + 1;
                if (friendly_pawns & board_utils::sq_ind_to_bit(neighbor_sq)) != 0 {
                    duo_bonus_mg += PAWN_DUO_BONUS[0];
                    duo_bonus_eg += PAWN_DUO_BONUS[1];

                    // 7. Mobile Pawn Duo Bonus
                    let front1_mask = board_utils::get_pawn_front_square_mask(color, sq);
                    let front2_mask = board_utils::get_pawn_front_square_mask(color, neighbor_sq);
                    let occupied = board.get_all_occupancy();
                    if (front1_mask & occupied) == 0 && (front2_mask & occupied) == 0 {
                        let bonus_sq = if color == WHITE {
                            sq
                        } else {
                            board_utils::flip_sq_ind_vertically(sq)
                        };
                        mg[color] += MOBILE_PAWN_DUO_BONUS_MG[bonus_sq];
                        eg[color] += MOBILE_PAWN_DUO_BONUS_EG[bonus_sq];
                    }
                }
            }
        } // End of loop through friendly_pawns
        mg[color] += chain_bonus_mg;
        eg[color] += chain_bonus_eg;
        mg[color] += duo_bonus_mg;
        eg[color] += duo_bonus_eg;

        // --- King Safety ---
        let king_bb = board.get_piece_bitboard(color, KING);
        if king_bb != 0 {
            // Ensure king exists
            let king_sq = king_bb.trailing_zeros() as usize;
            // 3. Pawn Shield Bonus
            let shield_zone_mask = board_utils::get_king_shield_zone_mask(color, king_sq);
            let shield_pawns = bits::popcnt(shield_zone_mask & friendly_pawns);
            mg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[0];
            eg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[1];
        }

        // --- Rook Bonuses ---
        let friendly_rooks = board.get_piece_bitboard(color, ROOK);
        let seventh_rank = if color == WHITE { 6 } else { 1 };
        let seventh_rank_mask = board_utils::get_rank_mask(seventh_rank);
        let rooks_on_seventh = friendly_rooks & seventh_rank_mask;
        let num_rooks_on_seventh = bits::popcnt(rooks_on_seventh);

        for rook_sq in bits::bits(&friendly_rooks) {
            let rank = board_utils::sq_to_rank(rook_sq);
            let file = board_utils::sq_to_file(rook_sq);
            let file_mask = board_utils::get_file_mask(file);

            // Rook on Open/Half-Open File
            let friendly_pawns_on_file = friendly_pawns & file_mask;
            let enemy_pawns_on_file = enemy_pawns & file_mask;

            if friendly_pawns_on_file == 0 {
                if enemy_pawns_on_file == 0 {
                    // Open File
                    mg[color] += ROOK_OPEN_FILE_BONUS[0];
                    eg[color] += ROOK_OPEN_FILE_BONUS[1];
                } else {
                    // Half-Open File
                    mg[color] += ROOK_HALF_OPEN_FILE_BONUS[0];
                    eg[color] += ROOK_HALF_OPEN_FILE_BONUS[1];
                }
            }

            // Rook behind friendly passed pawn
            let friendly_file_pawns = friendly_pawns & file_mask;
            for pawn_sq in bits::bits(&friendly_file_pawns) {
                let passed_mask = board_utils::get_passed_pawn_mask(color, pawn_sq);
                if (passed_mask & enemy_pawns) == 0 {
                    let pawn_rank = board_utils::sq_to_rank(pawn_sq);
                    if (color == WHITE && rank < pawn_rank) || (color == BLACK && rank > pawn_rank)
                    {
                        mg[color] += ROOK_BEHIND_PASSED_PAWN_BONUS[0];
                        eg[color] += ROOK_BEHIND_PASSED_PAWN_BONUS[1];
                        break;
                    }
                }
            }

            // Rook behind enemy passed pawn
            let enemy_file_pawns = enemy_pawns & file_mask;
            for pawn_sq in bits::bits(&enemy_file_pawns) {
                let passed_mask = board_utils::get_passed_pawn_mask(enemy_color, pawn_sq);
                if (passed_mask & friendly_pawns) == 0 {
                    let pawn_rank = board_utils::sq_to_rank(pawn_sq);
                    if (color == WHITE && rank > pawn_rank) || (color == BLACK && rank < pawn_rank)
                    {
                        mg[color] += ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS[0];
                        eg[color] += ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS[1];
                        break;
                    }
                }
            }
        } // End loop through friendly_rooks

        // Doubled Rooks on 7th - Additional bonus
        if num_rooks_on_seventh >= 2 {
            mg[color] += DOUBLED_ROOKS_ON_SEVENTH_BONUS[0];
            eg[color] += DOUBLED_ROOKS_ON_SEVENTH_BONUS[1];
        }

        // --- Castling Rights Bonus ---
        if color == WHITE {
            if board.castling_rights.white_kingside {
                mg[color] += CASTLING_RIGHTS_BONUS[0];
            }
            if board.castling_rights.white_queenside {
                mg[color] += CASTLING_RIGHTS_BONUS[0];
            }
        } else {
            // BLACK
            if board.castling_rights.black_kingside {
                mg[color] += CASTLING_RIGHTS_BONUS[0];
            }
            if board.castling_rights.black_queenside {
                mg[color] += CASTLING_RIGHTS_BONUS[0];
            }
        }
    } // End of loop through colors [WHITE, BLACK]

    (mg[WHITE] - mg[BLACK], eg[WHITE] - eg[BLACK]) // Return raw W-B score
}

// Helper to create test boards
pub fn create_test_board(fen: &str) -> Board {
    Board::new_from_fen(fen)
}

pub fn create_initial_board() -> Board {
    Board::new()
}
