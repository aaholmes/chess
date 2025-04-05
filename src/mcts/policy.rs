//! Policy network interface for MCTS.
//!
//! This module defines interfaces for policy networks used in MCTS.
//! It allows for different implementations of policy networks.

use crate::board::Board;
use crate::move_types::Move;
use std::collections::HashMap;

/// Trait for policy networks used in MCTS.
///
/// A policy network evaluates a position and returns:
/// 1. A prior probability distribution over legal moves
/// 2. A value estimate for the current position
pub trait PolicyNetwork {
    /// Evaluates a position and returns (policy, value).
    ///
    /// # Arguments
    /// * `board` - The chess position to evaluate.
    ///
    /// # Returns
    /// * `HashMap<Move, f64>` - Prior probabilities for each legal move.
    /// * `f64` - Value estimate for the position from the perspective of the player to move.
    ///           Should be in range [0.0, 1.0] where 1.0 means certain win for current player.
    fn evaluate(&self, board: &Board) -> (HashMap<Move, f64>, f64);
}

/// A simple random policy network for testing.
/// Returns uniform prior probabilities and random value.
pub struct RandomPolicy;

impl PolicyNetwork for RandomPolicy {
    fn evaluate(&self, _board: &Board) -> (HashMap<Move, f64>, f64) {
        // This is just a placeholder implementation
        // In a real implementation, you would generate all legal moves
        // and assign meaningful probabilities based on position analysis
        (HashMap::new(), 0.5)
    }
}

/// A simple material-based policy network.
/// Returns uniform prior probabilities but evaluates position based on material.
pub struct MaterialPolicy;

impl PolicyNetwork for MaterialPolicy {
    fn evaluate(&self, board: &Board) -> (HashMap<Move, f64>, f64) {
        // Placeholder: Assign uniform priors
        let priors = HashMap::new();

        // Simple material evaluation for value
        use crate::piece_types::*;

        // Use common piece values (in centipawns)
        let piece_values = [100, 320, 330, 500, 900, 0]; // P, N, B, R, Q, K

        let mut white_material = 0;
        let mut black_material = 0;

        // Count white material
        for piece_type in 0..6 {
            let pieces = board.get_piece_bitboard(WHITE, piece_type);
            let count = pieces.count_ones();
            white_material += count as i32 * piece_values[piece_type];
        }

        // Count black material
        for piece_type in 0..6 {
            let pieces = board.get_piece_bitboard(BLACK, piece_type);
            let count = pieces.count_ones();
            black_material += count as i32 * piece_values[piece_type];
        }

        // Calculate advantage from perspective of player to move
        let advantage = if board.w_to_move {
            white_material - black_material
        } else {
            black_material - white_material
        };

        // Convert material advantage to probability with sigmoid
        // Map advantage from centipawns to [0.0, 1.0] range
        let value = 1.0 / (1.0 + (-advantage as f64 / 400.0).exp());

        (priors, value)
    }
}
