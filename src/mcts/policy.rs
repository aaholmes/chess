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
    /// Evaluates a position given the legal moves and returns (policy, value).
    ///
    /// # Arguments
    /// * `board` - The chess position to evaluate.
    /// * `legal_moves` - A slice containing the legal moves for the current position.
    ///
    /// # Returns
    /// * `HashMap<Move, f64>` - Prior probabilities for each legal move provided.
    /// * `f64` - Value estimate for the position from the perspective of the player to move.
    ///           Should be in range [0.0, 1.0] where 1.0 means certain win for current player.
    fn evaluate(&self, board: &Board, legal_moves: &[Move]) -> (HashMap<Move, f64>, f64);
}

/// A simple random policy network for testing.
/// Returns uniform prior probabilities and random value.
pub struct RandomPolicy;

impl PolicyNetwork for RandomPolicy {
    fn evaluate(&self, _board: &Board, legal_moves: &[Move]) -> (HashMap<Move, f64>, f64) {
        let mut policy = HashMap::new();
        let num_moves = legal_moves.len();
        if num_moves > 0 {
            let uniform_prob = 1.0 / num_moves as f64;
            for mv in legal_moves {
                policy.insert(*mv, uniform_prob);
            }
        }
        (policy, 0.5) // Return uniform policy and neutral value
    }
}

/// A simple material-based policy network.
/// Returns uniform prior probabilities but evaluates position based on material.
pub struct MaterialPolicy;

impl PolicyNetwork for MaterialPolicy {
    fn evaluate(&self, board: &Board, legal_moves: &[Move]) -> (HashMap<Move, f64>, f64) {
        // Assign uniform priors for policy part
        let mut policy = HashMap::new();
        let num_moves = legal_moves.len();
        if num_moves > 0 {
            let uniform_prob = 1.0 / num_moves as f64;
            for mv in legal_moves {
                policy.insert(*mv, uniform_prob);
            }
        }

        // Simple material evaluation for value part
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

        (policy, value)
    }
}

/// Placeholder for a policy network loaded from an ONNX model.
pub struct OnnxPolicyNetwork {
    // Placeholder for model path, runtime session, etc.
    // model_path: String,
    // session: Option<onnxruntime::session::Session<'static>>, // Example using onnxruntime crate
}

impl OnnxPolicyNetwork {
    pub fn new(_model_path: &str) -> Self {
        // TODO: Implement model loading logic here
        OnnxPolicyNetwork {
            // model_path: model_path.to_string(),
            // session: None, // Load session here
        }
    }
}

impl PolicyNetwork for OnnxPolicyNetwork {
    fn evaluate(&self, _board: &Board, _legal_moves: &[Move]) -> (HashMap<Move, f64>, f64) {
        // TODO: Implement actual model inference here
        // 1. Convert board state to model input tensor
        // 2. Run inference using the ONNX session
        // 3. Convert model output (policy logits, value) back to HashMap<Move, f64> and f64
        // 4. Ensure policy probabilities are normalized and only include legal moves

        // Placeholder implementation:
        eprintln!("Warning: OnnxPolicyNetwork::evaluate is not implemented yet.");
        (HashMap::new(), 0.5) // Return empty policy and neutral value
    }
}
