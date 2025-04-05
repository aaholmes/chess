//! Monte Carlo Tree Search (MCTS) module.

pub mod node;
pub mod simulation; // Keep for testing/alternative use
pub mod policy;

use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};

// Import necessary components from submodules
pub use self::node::{MctsNode, select_leaf_for_expansion, MoveCategory};
// Don't re-export PolicyNetwork to avoid name conflicts
// pub use self::simulation::simulate_random_playout; // Don't export by default

// Constants
const EXPLORATION_CONSTANT: f64 = 1.414; // sqrt(2), a common value for UCT/PUCT

/// Backpropagates an evaluation result through the tree.
/// Updates visits and total_value for each node from the given leaf to the root.
/// Assumes 'value' is from White's perspective [0.0, 1.0].
fn backpropagate(node: Rc<RefCell<MctsNode>>, value: f64) {
    let mut current_node_opt = Some(node);
    while let Some(current_node_rc) = current_node_opt {
        { // Borrow scope
            let mut current_node = current_node_rc.borrow_mut();
            current_node.visits += 1;
            // Add value relative to the player whose turn it *was* in the parent
            // (i.e., the player who made the move *into* current_node)
            let reward_to_add = if current_node.state.w_to_move {
                 1.0 - value // Black just moved to get here, add Black's score (1 - White's score)
            } else {
                 value // White just moved to get here, add White's score
            };
            current_node.total_value += reward_to_add;
        } // End borrow scope

        // Move to parent
        current_node_opt = {
            let current_node = current_node_rc.borrow();
            if let Some(parent_weak) = &current_node.parent {
                parent_weak.upgrade()
            } else {
                None // Reached root
            }
        };
    }
}

/// Performs Monte Carlo Tree Search on a given position.
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
    let root_node_rc = MctsNode::new_root(root_state, move_gen);
    
    // Initialize unexplored moves for the root
    {
        let mut root_node = root_node_rc.borrow_mut();
        if !root_node.is_terminal {
            // Generate all legal moves for the root
            let legal_moves = MctsNode::get_legal_moves(&root_node.state, move_gen);
            
            // Store them in the unexplored_moves_by_cat map
            if !legal_moves.is_empty() {
                // For simplicity, just use Quiet category for all moves
                let cat = MoveCategory::Quiet;
                root_node.unexplored_moves_by_cat.insert(cat, legal_moves);
                root_node.current_priority_category = Some(cat);
            }
        }
    }

    // Handle case where root node is already terminal or has no legal moves
    if root_node_rc.borrow().is_terminal || root_node_rc.borrow().unexplored_moves_by_cat.is_empty() {
        return None; // No moves possible
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
        // 1. Selection: Find a leaf node suitable for expansion/evaluation.
        let leaf_node_rc = node::select_leaf_for_expansion(root_node_rc.clone(), EXPLORATION_CONSTANT);

        // --- 2. Simulation and Backpropagation ---
        let node_to_backprop_rc: Rc<RefCell<MctsNode>>;
        let value_to_backprop: f64; // Value relative to White [0.0, 1.0]

        { // Scope for borrowing leaf_node
            let mut leaf_node = leaf_node_rc.borrow_mut();

            // Check if the node is terminal
            if leaf_node.is_terminal {
                // Terminal state, use stored value
                if leaf_node.state.w_to_move {
                    // White's turn and is checkmate = white loss (0.0)
                    // White's turn and is stalemate = draw (0.5)
                    value_to_backprop = if leaf_node.is_game_terminal() { 0.0 } else { 0.5 };
                } else {
                    // Black's turn and is checkmate = white win (1.0)
                    // Black's turn and is stalemate = draw (0.5)
                    value_to_backprop = if leaf_node.is_game_terminal() { 1.0 } else { 0.5 };
                }
                node_to_backprop_rc = leaf_node_rc.clone(); // Backpropagate from leaf
            } else if leaf_node.children.is_empty() {
                // Leaf node that needs expansion
                
                // Initialize unexplored moves if needed
                if leaf_node.unexplored_moves_by_cat.is_empty() {
                    // Generate all legal moves for this node
                    let legal_moves = MctsNode::get_legal_moves(&leaf_node.state, move_gen);
                    
                    // Store them in the unexplored_moves_by_cat map
                    if !legal_moves.is_empty() {
                        // For simplicity, just use Quiet category for all moves
                        let cat = MoveCategory::Quiet;
                        leaf_node.unexplored_moves_by_cat.insert(cat, legal_moves);
                        leaf_node.current_priority_category = Some(cat);
                    }
                }
                
                // If it has unexplored moves, expand one
                if let Some(action_to_expand) = leaf_node.get_best_unexplored_move() {
                    let next_state = leaf_node.state.apply_move_to_board(action_to_expand);
                    let is_white_to_move = next_state.w_to_move; // Save this before moving next_state
                    let parent_weak = Rc::downgrade(&leaf_node_rc);
                    
                    // Clone next_state before passing ownership to new_child
                    let new_child_rc = MctsNode::new_child(parent_weak, action_to_expand, next_state.clone(), move_gen);
                    
                    // Perform simulation on the new node
                    let simulation_result = simulation::simulate_random_playout(&next_state, move_gen);
                    
                    // Transform result to white's perspective if needed
                    value_to_backprop = if is_white_to_move {
                        simulation_result
                    } else {
                        1.0 - simulation_result
                    };
                    
                    leaf_node.children.push(new_child_rc.clone());
                    node_to_backprop_rc = leaf_node_rc.clone(); // Backpropagate from parent
                } else {
                    // No more unexplored moves - this can happen if all moves are illegal
                    value_to_backprop = 0.5; // Default to draw
                    node_to_backprop_rc = leaf_node_rc.clone();
                }
            } else {
                // Should not happen with properly implemented selection
                value_to_backprop = 0.5; // Default to draw
                node_to_backprop_rc = leaf_node_rc.clone();
            }
        } // End borrow scope for leaf_node

        // --- 3. Backpropagation ---
        backpropagate(node_to_backprop_rc, value_to_backprop);

        iteration_count += 1;
    }

    // --- Select Best Move ---
    // Choose the child of the root with the highest visit count (most robust).
    // Save the result before returning to avoid lifetime issues
    let best_move = {
        let root_node = root_node_rc.borrow();
        root_node
            .children
            .iter()
            .max_by_key(|child| child.borrow().visits)
            .map(|best_child| best_child.borrow().action.expect("Child node must have an action"))
    };
    
    best_move
}