//! Monte Carlo Tree Search (MCTS) module.

pub mod node;
pub mod policy;
pub mod simulation; // Keep for testing/alternative use

use crate::board::Board;
use crate::boardstack::BoardStack; // Needed for mate_search
use crate::mcts::policy::PolicyNetwork;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use crate::search::mate_search; // Import mate_search function
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant}; // Use trait from submodule

// Import necessary components from submodules
pub use self::node::{select_leaf_for_expansion, MctsNode, MoveCategory};
pub use self::policy::PolicyNetwork; // Re-export trait
                                     // pub use self::simulation::simulate_random_playout; // Don't export by default

// Constants
const EXPLORATION_CONSTANT: f64 = 1.414; // sqrt(2), a common value for UCT/PUCT
const PESSIMISTIC_FACTOR: f64 = 1.0; // Factor 'k' for pessimistic value calculation (Q - k * std_err)

/// Backpropagates an evaluation result through the tree.
/// Updates visits, total_value, and total_value_squared for each node.
/// Assumes 'value' is from White's perspective [0.0, 1.0].
fn backpropagate(node: Rc<RefCell<MctsNode>>, value: f64) {
    let mut current_node_opt = Some(node);
    while let Some(current_node_rc) = current_node_opt {
        {
            // Borrow scope
            let mut current_node = current_node_rc.borrow_mut();
            current_node.visits += 1;
            // Add value relative to the player whose turn it *was* in the parent
            let reward_to_add = if current_node.state.w_to_move {
                1.0 - value // Black just moved to get here, add Black's score
            } else {
                value // White just moved to get here, add White's score
            };
            current_node.total_value += reward_to_add;
            current_node.total_value_squared += reward_to_add.powi(2); // Accumulate squared reward
            current_node.total_value_squared += reward_to_add.powi(2); // Accumulate squared reward
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

/// Performs MCTS search using Policy/Value evaluation and Mate Search First strategy.
///
/// # Arguments
/// * `root_state` - The initial board state.
/// * `move_gen` - The move generator.
/// * `policy_network` - Implementation of the PolicyNetwork trait.
/// * `mate_search_depth` - Depth for the classical mate search check. Set <= 0 to disable.
/// * `iterations` - Optional number of MCTS iterations to run.
/// * `time_limit` - Optional time duration limit for the search.
///
/// # Returns
/// The best move found for the root state based on visit counts. Returns None if no legal moves from root.
pub fn mcts_search<P: PolicyNetwork>(
    root_state: Board,
    move_gen: &MoveGen,
    policy_network: &P,
    mate_search_depth: i32,
    iterations: Option<u32>,
    time_limit: Option<Duration>,
) -> Option<Move> {
    if iterations.is_none() && time_limit.is_none() {
        panic!("MCTS search requires either iterations or time_limit to be set.");
    }
    if mate_search_depth < 0 {
        panic!("Mate search depth cannot be negative.");
    }

    let start_time = Instant::now();
    let root_node_rc = MctsNode::new_root(root_state, move_gen);

    // Handle case where root node is already terminal
    if root_node_rc.borrow().is_terminal {
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
            if (iteration_count & 63) == 0 && start_time.elapsed() >= limit {
                // Check time periodically
                break;
            }
        }

        // --- MCTS Cycle ---
        // 1. Selection: Find a leaf node suitable for expansion/evaluation.
        let leaf_node_rc = select_leaf_for_expansion(root_node_rc.clone(), EXPLORATION_CONSTANT);

        // --- 2. Mate Check / Evaluation / Expansion ---
        let mut node_to_propagate_from: Rc<RefCell<MctsNode>>;
        let mut value_to_propagate: f64; // Value relative to White [0.0, 1.0]

        {
            // Scope for borrowing leaf_node
            let mut leaf_node = leaf_node_rc.borrow_mut();

            // --- 2a. Check Terminal/Mate Status (only once per node) ---
            if leaf_node.terminal_or_mate_value.is_none() {
                let (is_mate, is_stalemate) = leaf_node.state.is_checkmate_or_stalemate(move_gen);
                if is_stalemate {
                    leaf_node.terminal_or_mate_value = Some(0.5);
                } else if is_mate {
                    leaf_node.terminal_or_mate_value =
                        Some(if leaf_node.state.w_to_move { 0.0 } else { 1.0 });
                // White's perspective
                } else if mate_search_depth > 0 {
                    // Not terminal, run mate search
                    let mut mate_search_stack = BoardStack::with_board(leaf_node.state.clone());
                    let (mate_score, _, _) =
                        mate_search(&mut mate_search_stack, move_gen, mate_search_depth, false);
                    if mate_score >= 1_000_000 {
                        // Mate found for current player
                        leaf_node.terminal_or_mate_value =
                            Some(if leaf_node.state.w_to_move { 1.0 } else { 0.0 });
                    } else if mate_score <= -1_000_000 {
                        // Mated
                        leaf_node.terminal_or_mate_value =
                            Some(if leaf_node.state.w_to_move { 0.0 } else { 1.0 });
                    } else {
                        leaf_node.terminal_or_mate_value = Some(-999.0); // Sentinel: No mate found
                    }
                } else {
                    leaf_node.terminal_or_mate_value = Some(-999.0); // Mate search disabled
                }
            }

            // --- 2b. Decide Value and Node for Backpropagation ---
            if let Some(exact_value) = leaf_node.terminal_or_mate_value {
                if exact_value >= 0.0 {
                    // Mate found or terminal state (0.0, 0.5, 1.0)
                    value_to_propagate = exact_value;
                    node_to_propagate_from = leaf_node_rc.clone(); // Backpropagate exact value from leaf
                } else {
                    // No mate found (sentinel -999.0), proceed to evaluate/expand
                    // --- 2c. Evaluate Leaf (if not already done) ---
                    if leaf_node.nn_value.is_none() {
                        let (priors, value_current_player) =
                            policy_network.evaluate(&leaf_node.state);
                        let value_white_pov = if leaf_node.state.w_to_move {
                            value_current_player
                        } else {
                            1.0 - value_current_player
                        };
                        leaf_node.nn_value = Some(value_white_pov);
                        leaf_node.store_priors_and_categorize_moves(priors, move_gen); // Now categorize moves
                        value_to_propagate = value_white_pov;
                        node_to_propagate_from = leaf_node_rc.clone(); // Backpropagate evaluated value from leaf
                    } else {
                        // --- 2d. Expand Leaf ---
                        if let Some(action_to_expand) = leaf_node.get_best_unexplored_move() {
                            let next_state = leaf_node.state.apply_move_to_board(action_to_expand);
                            let parent_weak = Rc::downgrade(&leaf_node_rc);
                            let new_child_rc = MctsNode::new_child(
                                parent_weak,
                                action_to_expand,
                                next_state,
                                move_gen,
                            );
                            leaf_node.children.push(new_child_rc); // Add child

                            // Backpropagate the *parent's* stored NN value when expanding (AlphaZero style)
                            value_to_propagate = leaf_node.nn_value.unwrap();
                            node_to_propagate_from = leaf_node_rc.clone(); // Start backprop from parent of new child
                        } else {
                            // Node evaluated but fully explored. This can happen.
                            // Backpropagate the known value again to reinforce parent nodes.
                            value_to_propagate = leaf_node.nn_value.unwrap();
                            node_to_propagate_from = leaf_node_rc.clone();
                        }
                    }
                }
            } else {
                panic!("Node terminal/mate status should have been determined");
            }
        } // End borrow scope for leaf_node

        // --- 3. Backpropagation ---
        backpropagate(node_to_propagate_from, value_to_propagate);

        iteration_count += 1;
    }

    // --- Select Best Move --- (Using Pessimistic Value: Q - k * StdErr)
    let best_move = {
        let root_node = root_node_rc.borrow();
        root_node.children.iter()
            .filter(|child_rc| child_rc.borrow().visits > 0) // Only consider visited children
            .max_by(|a_rc, b_rc| {
                let a = a_rc.borrow();
                let b = b_rc.borrow();
                let visits_a = a.visits as f64;
                let visits_b = b.visits as f64;

                // Calculate Q (average value from White's perspective)
                let q_a = a.total_value / visits_a;
                let q_b = b.total_value / visits_b;

                // Calculate Standard Error = sqrt(Var(X) / N) = sqrt( (E[X^2] - E[X]^2) / N )
                let var_a = (a.total_value_squared / visits_a) - q_a.powi(2);
                let var_b = (b.total_value_squared / visits_b) - q_b.powi(2);
                // Ensure variance is non-negative due to potential floating point inaccuracies
                let std_err_a = (var_a.max(0.0) / visits_a).sqrt();
                let std_err_b = (var_b.max(0.0) / visits_b).sqrt();

                // Calculate Pessimistic Score (Lower Confidence Bound from White's perspective)
                let pessimistic_a = q_a - PESSIMISTIC_FACTOR * std_err_a;
                let pessimistic_b = q_b - PESSIMISTIC_FACTOR * std_err_b;

                // Compare based on whose turn it is at the root
                if root_node.state.w_to_move {
                    // White wants to maximize the pessimistic score
                    pessimistic_a.partial_cmp(&pessimistic_b).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    // Black wants to minimize White's pessimistic score
                    // This is equivalent to maximizing Black's pessimistic score: (1-Q) - k*StdErr
                    // Or minimizing White's Upper Confidence Bound: Q + k*StdErr
                    let ucb_a = q_a + PESSIMISTIC_FACTOR * std_err_a;
                    let ucb_b = q_b + PESSIMISTIC_FACTOR * std_err_b;
                    // Black chooses the move with the *lowest* UCB for White
                    ucb_b.partial_cmp(&ucb_a).unwrap_or(std::cmp::Ordering::Equal)
                }
            })
            .map(|best_child| best_child.borrow().action.expect("Child node must have an action"))
    };

    best_move
}
