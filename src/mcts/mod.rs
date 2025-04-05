//! Monte Carlo Tree Search (MCTS) module.

pub mod node;
pub mod simulation;
// pub mod selection; // Or keep selection logic within node.rs
// pub mod expansion; // Or keep expansion logic within node.rs
// pub mod backpropagation; // Or keep backpropagation logic within node.rs

use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use std::time::{Duration, Instant};

// Import necessary components from submodules
pub use self::node::{MctsNode, select_leaf}; // Remove backpropagate as it's a method
pub use self::simulation::simulate_random_playout;

// Constants
const EXPLORATION_CONSTANT: f64 = 1.414; // sqrt(2), a common value for C in UCT

/// Performs MCTS search for a given number of iterations or time limit.
///
/// # Arguments
/// * `root_state` - The initial board state.
/// * `move_gen` - The move generator.
/// * `iterations` - Optional number of MCTS iterations to run.
/// * `time_limit` - Optional time duration limit for the search.
///
/// # Returns
/// The best move found for the root state. Returns None if no legal moves from root.
pub fn mcts_search<P: PolicyNetwork>( // Generic over PolicyNetwork trait
    root_state: Board,
    move_gen: &MoveGen,
    policy_network: &P, // Pass policy network implementation
    iterations: Option<u32>,
    time_limit: Option<Duration>,
) -> Option<Move> {
    if iterations.is_none() && time_limit.is_none() {
        panic!("MCTS search requires either iterations or time_limit to be set.");
    }

    let start_time = Instant::now();
    let root_node = MctsNode::new_root(root_state, move_gen);

    // Handle case where root node has no possible moves (e.g., immediate stalemate/checkmate)
    if root_node.borrow().children.is_empty() && root_node.borrow().untried_actions.is_empty() {
        return None;
    }

    let mut iteration_count = 0;

    loop {
        // --- Check Termination Conditions ---
        if let Some(max_iter) = iterations {
            if iteration_count >= max_iter {
                break;
            }
        }
        if let Some(limit) = time_limit {
            // Check time limit more frequently if iterations are high or time limit is short
            // Simple check every iteration for now:
            if start_time.elapsed() >= limit {
                break;
            }
        }

        // --- MCTS Cycle ---
        // 1. Selection: Traverse the tree using UCT to find a leaf node.
        let leaf_node = select_leaf(root_node.clone(), EXPLORATION_CONSTANT);

        // 2. Expansion: If the leaf is not terminal, expand it by adding one child.
        // 2. Expansion & Evaluation: If the leaf is not terminal, expand it.
        //    Expansion now also involves evaluating the node using the policy network.
        let (node_to_backpropagate, value_from_white_pov) = if !leaf_node.borrow().is_terminal() {
            // Expand returns the new child and the evaluated value (0-1 for White) of the *parent* (leaf_node).
            expand_with_policy(leaf_node.clone(), move_gen, policy_network)
        } else {
            // If terminal, get the exact value (0.0 loss, 0.5 draw, 1.0 win for White)
            let terminal_value = {
                let state = leaf_node.borrow().state.clone();
                let (is_mate, is_stalemate) = state.is_checkmate_or_stalemate(move_gen);
                if is_stalemate {
                    0.5
                } else if is_mate {
                    // If white is to move and it's mate, black won (0.0)
                    // If black is to move and it's mate, white won (1.0)
                    if state.w_to_move { 0.0 } else { 1.0 }
                } else {
                    // Should not happen if is_terminal is true
                    0.5 // Default to draw if logic error
                }
            };
            (leaf_node, terminal_value) // Backpropagate the terminal value from the leaf itself
        };

        // 3. Simulation: Removed. Evaluation is done during expansion.

        // 4. Backpropagation: Update visits and rewards up the tree.
        MctsNode::backpropagate(node_to_backpropagate, value_from_white_pov);

        iteration_count += 1;
    }

    // --- Select Best Move ---
    // Choose the child of the root with the highest visit count (most robust).
    // Handle the case where root might not have children if the game ended immediately.
    let best_move = root_node
        .borrow()
        .children
        .iter()
        .max_by_key(|child| child.borrow().visits)
        .map(|best_child| best_child.borrow().action.expect("Child node must have an action"));
    
    best_move
}