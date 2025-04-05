//! Defines the trait for policy and value networks used in MCTS.

use crate::board::Board;
use crate::move_types::Move;
use std::collections::HashMap;

/// Trait for a policy and value network.
/// In AlphaZero-style MCTS, this typically involves a neural network.
/// Here, we can create implementations using PestoEval or other methods.
pub trait PolicyNetwork {
    /// Evaluates a board state, returning prior probabilities for legal moves
    /// and a state value estimate.
    ///
    /// # Arguments
    /// * `board` - The board state to evaluate.
    ///
    /// # Returns
    /// A tuple containing:
    /// * A map from legal `Move` to its prior probability (`f64`). The sum of probabilities should ideally be 1.0.
    /// * The estimated value (`f64`) of the state, typically in the range [-1.0, 1.0] or [0.0, 1.0]
    ///   from the perspective of the current player to move. Let's use [0.0, 1.0] where 1.0 is a win for the current player.
    fn evaluate(&self, board: &Board) -> (HashMap<Move, f64>, f64);
}

// Example implementation using PestoEval (and uniform policy priors)
// This demonstrates how to adapt the existing eval function.
use crate::eval::PestoEval;
use crate::move_generation::MoveGen;
use crate::mcts::node::MctsNode; // For get_legal_moves

pub struct PestoPolicy<'a> {
    evaluator: &'a PestoEval,
    move_gen: &'a MoveGen,
}

impl<'a> PestoPolicy<'a> {
    pub fn new(evaluator: &'a PestoEval, move_gen: &'a MoveGen) -> Self {
        PestoPolicy { evaluator, move_gen }
    }
}

impl<'a> PolicyNetwork for PestoPolicy<'a> {
    fn evaluate(&self, board: &Board) -> (HashMap<Move, f64>, f64) {
        // 1. Get Value from PestoEval
        // Pesto returns score relative to current player. Need to normalize.
        // Assuming score is in centipawns. Convert to win probability (e.g., using sigmoid).
        let score_cp = self.evaluator.eval(board, self.move_gen);
        // Simple sigmoid-like scaling: map [-800, 800] cp to approx [0, 1]
        // k determines steepness. Let k = 1 / 400 for reasonable scaling.
        let k = 1.0 / 400.0;
        let value = 1.0 / (1.0 + (-k * score_cp as f64).exp()); // Value in [0, 1] for current player

        // 2. Get Policy Priors (Uniform for now)
        let legal_moves = MctsNode::get_legal_moves(board, self.move_gen);
        let num_legal_moves = legal_moves.len();
        let mut priors = HashMap::new();
        if num_legal_moves > 0 {
            let uniform_prior = 1.0 / num_legal_moves as f64;
            for mv in legal_moves {
                priors.insert(mv, uniform_prior);
            }
        }

        (priors, value)
    }
}