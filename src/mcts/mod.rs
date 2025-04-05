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
pub fn mcts_search(
    root_state: Board,
    move_gen: &MoveGen,
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
        let node_to_simulate = if !leaf_node.borrow().is_terminal() {
            // Expansion needs the Rc to create the Weak parent ref.
            // We call the static expand function, passing the Rc of the leaf.
            MctsNode::expand(leaf_node.clone(), move_gen)
        } else {
            leaf_node // If terminal, simulate from the terminal leaf itself
        };

        // 3. Simulation: Run a random playout from the newly expanded node (or terminal leaf).
        //    Result is from the perspective of the player whose turn it was at node_to_simulate.
        //    We need to convert this to White's perspective (1.0=W win, 0.0=B win) for backpropagation.
        let simulation_result_from_white_pov = {
            // Clone the state *before* borrowing to avoid holding borrow across simulation
            let sim_state = node_to_simulate.borrow().state.clone();
            let result_from_sim_player = simulate_random_playout(&sim_state, move_gen);

            if sim_state.w_to_move { // If White was to move at start of sim
                result_from_sim_player
            } else { // If Black was to move at start of sim
                1.0 - result_from_sim_player
            }
        };

        // 4. Backpropagation: Update visits and rewards up the tree.
        MctsNode::backpropagate(node_to_simulate, simulation_result_from_white_pov);

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