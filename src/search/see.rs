use std::cmp::max;
use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::piece_types::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING};

// Piece values for SEE (simple centipawn values)
// Order: P, N, B, R, Q, K (index 6 for NO_PIECE_TYPE is 0)
const SEE_PIECE_VALUES: [i32; 7] = [100, 320, 330, 500, 975, 10000, 0];

/// Calculates the Static Exchange Evaluation (SEE) for a move to a target square.
/// Determines if a sequence of captures on `target_sq` initiated by the current side to move
/// is likely to win material.
///
/// # Arguments
/// * `board` - The current board state.
/// * `move_gen` - The move generator (needed for finding attackers).
/// * `target_sq` - The square where the capture sequence occurs.
/// * `initial_attacker_sq` - The square of the piece making the initial capture.
///
/// # Returns
/// The estimated material balance after the exchange sequence. Positive means gain, negative means loss.
/// Returns 0 if the target square is empty or the initial attacker is invalid.
pub fn see(board: &Board, move_gen: &MoveGen, target_sq: usize, initial_attacker_sq: usize) -> i32 {
    let mut gain = [0; 32]; // Max possible captures in a sequence
    let mut depth = 0;
    let mut current_board = board.clone(); // Clone board to simulate captures
    let mut side_to_move = board.w_to_move;

    // Get initial captured piece type and value
    let captured_piece_type = match current_board.get_piece_type_on_sq(target_sq) {
        Some(pt) => pt,
        None => return 0, // Target square is empty, not a capture
    };
    gain[depth] = SEE_PIECE_VALUES[captured_piece_type as usize];

    // Get initial attacker piece type
    let mut attacker_piece_type = match current_board.get_piece_type_on_sq(initial_attacker_sq) {
        Some(pt) => pt,
        None => return 0, // Should not happen for a valid capture move
    };

    // Simulate the initial capture
    current_board.clear_square(initial_attacker_sq);
    current_board.set_square(target_sq, attacker_piece_type, side_to_move);
    current_board.update_occupancy();
    side_to_move = !side_to_move; // Switch sides

    loop {
        depth += 1;
        // Score relative to previous capture. gain[d] = value of attacker - gain[d-1]
        gain[depth] = SEE_PIECE_VALUES[attacker_piece_type as usize] - gain[depth - 1];

        if max(-gain[depth - 1], gain[depth]) < 0 {
            break;
        }

        let attackers_bb = move_gen.attackers_to(&current_board, target_sq, side_to_move);
        if attackers_bb == 0 {
            break; // No more attackers for the current side
        }

        let next_attacker_sq = find_least_valuable_attacker_sq(&current_board, attackers_bb, side_to_move);
        if next_attacker_sq == 64 {
            break;
        }

        attacker_piece_type = current_board.get_piece_type_on_sq(next_attacker_sq).unwrap();

        current_board.clear_square(next_attacker_sq);
        current_board.set_square(target_sq, attacker_piece_type, side_to_move);
        current_board.update_occupancy();
        side_to_move = !side_to_move;
    }

    // Calculate final score by propagating the gains/losses back up the sequence
    while depth > 0 {
        depth -= 1;
        gain[depth] = -max(-gain[depth], gain[depth + 1]);
    }
    gain[0]
}

/// Helper function to find the square of the least valuable attacker from a bitboard of attackers.
fn find_least_valuable_attacker_sq(board: &Board, attackers_bb: u64, side: bool) -> usize {
    let color_index = side as usize;
    for piece_type_idx in PAWN..=KING {
        let piece_bb = board.pieces[color_index][piece_type_idx as usize];
        let intersection = attackers_bb & piece_bb;
        if intersection != 0 {
            return intersection.trailing_zeros() as usize;
        }
    }
    64 // Indicate no attacker found (error condition)
} 