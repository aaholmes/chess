# Design Doc: Hybrid MCTS with Prioritized Selection and Mate Search

## 1. Goals

*   Implement a Monte Carlo Tree Search (MCTS) algorithm for the Kingfisher chess engine.
*   Integrate the existing linear evaluation function (`PestoEval`) for state value estimation, ensuring its weights (`eval_constants.rs`) are structured for future trainability.
*   Implement the "Mate Search First" strategy: Utilize the existing classical `mate_search` function to find forced mates at leaf nodes, using the exact result (win/loss) for backpropagation and bypassing policy/value evaluation if a mate is found.
*   Implement a modified MCTS **Selection** strategy that prioritizes exploring moves based on predefined heuristic categories (e.g., Mates, Captures, Killers, Checks, Quiet) before applying the PUCT selection metric.
*   Define a `PolicyNetwork` interface to provide prior probabilities (`P(a|s)`) for moves, initially using uniform priors via `PestoPolicy`.
*   Structure the implementation modularly within `src/mcts/`.

## 2. Background

Standard MCTS relies heavily on random simulations (rollouts) or a learned value/policy network (like AlphaZero). This design aims for a hybrid approach leveraging existing classical components:
1.  **Trainable Linear Evaluation:** Use `PestoEval` as `V(s)`, making its weights parameters for future learning via self-play, avoiding the need for a deep NN value head initially.
2.  **Classical Search Integration:** Incorporate `mate_search` results directly into the MCTS value estimation and use heuristic move categorization to guide MCTS exploration more intelligently than uniform random selection or basic PUCT alone, especially before a policy network is trained.

## 3. Proposed Changes

1.  **Refactor `PestoEval`:** Modify `PestoEval` and `eval_constants.rs` to store evaluation weights as struct fields rather than global constants, enabling future tuning.
2.  **Modify `MctsNode`:** Enhance the node structure to support the new selection and evaluation strategies.
3.  **Implement Prioritized Selection:** Modify the selection phase logic to use heuristic move categories.
4.  **Implement Mate Search First Evaluation:** Modify the expansion/evaluation phase to run `mate_search` before consulting the `PolicyNetwork`.
5.  **Update Backpropagation:** Ensure backpropagation correctly handles values derived from both mate search (exact 1.0/0.0) and `PestoEval` (normalized 0.0-1.0).
6.  **Update Main Loop:** Modify `mcts_search` to orchestrate the new cycle.

## 4. Data Structures

### 4.1. `MoveCategory` Enum

Define an enum to represent move priorities. The order defines the priority (lower discriminant = higher priority).

```rust
// (Location: Potentially mcts/node.rs or mcts/policy.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum MoveCategory {
    ForcedMate,      // Priority 0 (Highest) - If known via shallow search/checkmate
    PromotionToQueen,// Priority 1
    WinningCapture,  // Priority 2 (SEE > threshold)
    KillerMove1,     // Priority 3 (Quiet move)
    KillerMove2,     // Priority 4 (Quiet move)
    EqualCapture,    // Priority 5 (abs(SEE) <= threshold)
    Check,           // Priority 6 (Non-capture, non-promo, non-killer)
    HistoryHeuristic, // Priority 7 (Quiet moves with high history scores)
    OtherQuiet,      // Priority 8
    LosingCapture,   // Priority 9 (SEE < -threshold)
}
4.2. MctsNode Modifications (mcts/node.rs)
use crate::move_types::Move;
use crate::board::Board;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap; // Needed for categorized moves

// ... (Existing imports, NEXT_NODE_ID) ...
use super::policy::PolicyNetwork; // Assuming policy.rs exists

#[derive(Debug)]
pub struct MctsNode {
    pub id: usize,
    pub state: Board,
    pub parent: Option<Weak<RefCell<MctsNode>>>,
    pub children: Vec<Rc<RefCell<MctsNode>>>, // Explored children
    pub action: Option<Move>, // Move from parent

    // --- Statistics ---
    pub visits: u32,
    pub total_value: f64, // Accumulated value (0-1 relative to White) from backpropagation

    // --- Evaluation / Mate Status ---
    /// Stores the exact value if determined by mate search (1.0 W win, 0.0 B win) or terminal state.
    pub terminal_or_mate_value: Option<f64>,
    /// Stores the value from the policy/evaluation network (0-1 relative to White) if evaluated.
    pub nn_value: Option<f64>,
    /// Stores the policy priors for all legal moves from this state, evaluated once.
    pub priors: Option<HashMap<Move, f64>>,

    // --- Expansion / Selection Control ---
    /// Stores unexplored legal moves, categorized by priority.
    /// Key: MoveCategory, Value: List of Moves in that category.
    pub unexplored_moves_by_cat: HashMap<MoveCategory, Vec<Move>>,
    /// Tracks the current highest-priority category being explored.
    pub current_priority_category: MoveCategory,
    /// Stores the policy prior P(action|state) for the action leading to this node.
    pub policy_prior: f64,
}

impl MctsNode {
    // --- Constructor Updates ---
    // new_root: Initializes fields, potentially calls categorize_moves if policy known immediately.
    // new_child: Needs policy_prior, initializes other fields.

    // --- Helper for Categorization (Called once per node, likely after policy evaluation) ---
    // fn store_priors_and_categorize_moves(&mut self, priors: HashMap<Move, f64>, move_gen: &MoveGen)

    // --- PUCT Calculation (Uses stored policy_prior) ---
    // pub fn puct_value(...) -> f64

    // --- Backpropagation (Remains similar, uses total_value) ---
    // pub fn backpropagate(...)

    // --- Other helpers ---
    // pub fn is_fully_explored(&self) -> bool // Checks if unexplored_moves_by_cat is empty
    // pub fn advance_priority_category(&mut self) -> bool // Moves to next category, returns false if all exhausted
    // pub fn get_best_unexplored_move(&mut self) -> Option<Move> // Gets & removes move from current category
}
Self-Correction: Added priors: Option<HashMap<Move, f64>> to store policy results. Modified relevant method comments.

5. Algorithm Details
5.1. Refactor PestoEval
Modify PestoEval struct to hold weights (e.g., two_bishops_bonus: [i32; 2], etc.) loaded from constants or configuration.
Update eval_plus_game_phase to use self.weight_field instead of global constants.
Implement normalization (e.g., sigmoid) within PestoPolicy::evaluate to convert centipawn score to a [0.0, 1.0] value relative to the current player.
5.2. PolicyNetwork Trait (mcts/policy.rs)
evaluate(&self, board: &Board) -> (HashMap<Move, f64>, f64) remains the same.
PestoPolicy::evaluate implementation:
Gets legal moves.
Calculates normalized value v using PestoEval::eval and sigmoid.
Calculates uniform priors P(a|s) = 1 / N for all legal moves a.
Returns (priors_map, v).
5.3. MCTS Cycle (mcts_search in mcts/mod.rs)
The core loop needs restructuring to accommodate the "Mate Search First" and prioritized expansion logic.

pub fn mcts_search<P: PolicyNetwork>(
    root_state: Board,
    move_gen: &MoveGen,
    policy_network: &P,
    // evaluator: &PestoEval, // Assuming PolicyNetwork provides value
    mate_search_depth: i32,
    iterations: Option<u32>,
    time_limit: Option<Duration>,
) -> Option<Move> {
    let root_node_rc = MctsNode::new_root(root_state, move_gen); // Does not categorize yet

    loop {
        // --- Termination Check ---
        // ...

        // --- 1. Selection (Prioritized) ---
        // Traverses the tree using PUCT on *explored* children,
        // but prioritizes expanding nodes based on category if available.
        let leaf_node_rc = select_leaf_for_expansion(root_node_rc.clone(), EXPLORATION_CONSTANT);

        // --- 2. Mate Check ---
        let mut leaf_node = leaf_node_rc.borrow_mut(); // Mutable borrow needed
        let mut value_to_propagate: Option<f64> = None; // Value relative to White [0.0, 1.0]

        if leaf_node.terminal_or_mate_value.is_none() { // Only run mate search once
            // Check terminal state first
            let (is_mate, is_stalemate) = leaf_node.state.is_checkmate_or_stalemate(move_gen);
            if is_stalemate {
                leaf_node.terminal_or_mate_value = Some(0.5);
            } else if is_mate {
                 // If white is to move and it's mate, black won (0.0)
                 // If black is to move and it's mate, white won (1.0)
                leaf_node.terminal_or_mate_value = Some(if leaf_node.state.w_to_move { 0.0 } else { 1.0 });
            } else {
                // Not terminal, run mate search
                let (mate_score, _mate_move, _nodes) = mate_search(&mut BoardStack::with_board(leaf_node.state.clone()), move_gen, mate_search_depth, false);
                if mate_score == 1000000 { // Mate found for current player
                    leaf_node.terminal_or_mate_value = Some(if leaf_node.state.w_to_move { 1.0 } else { 0.0 });
                } else if mate_score == -1000000 { // Mated
                    leaf_node.terminal_or_mate_value = Some(if leaf_node.state.w_to_move { 0.0 } else { 1.0 });
                }
                // If no mate found by search, terminal_or_mate_value remains None for now
            }
        }

        // --- 3. Expansion & Evaluation (If no mate/terminal state found) ---
        let node_to_propagate_from: Rc<RefCell<MctsNode>>;

        if let Some(exact_value) = leaf_node.terminal_or_mate_value {
            // Mate found or terminal state reached
            value_to_propagate = Some(exact_value);
            node_to_propagate_from = leaf_node_rc.clone(); // Backpropagate from the leaf itself
            drop(leaf_node); // Release borrow
        } else {
            // No mate found, node is not terminal. Evaluate if not already done.
            if leaf_node.nn_value.is_none() {
                let (priors, value_current_player) = policy_network.evaluate(&leaf_node.state);
                let value_white_pov = if leaf_node.state.w_to_move { value_current_player } else { 1.0 - value_current_player };
                leaf_node.nn_value = Some(value_white_pov);
                leaf_node.store_priors_and_categorize_moves(priors, move_gen); // New method
                value_to_propagate = Some(value_white_pov);
                node_to_propagate_from = leaf_node_rc.clone(); // Backpropagate from evaluated leaf
                drop(leaf_node); // Release borrow
            } else {
                // Already evaluated, expand the highest priority available move
                let best_unexplored_move = leaf_node.get_best_unexplored_move(); // New method
                if let Some(action) = best_unexplored_move {
                    let next_state = leaf_node.state.apply_move_to_board(action);
                    let prior = leaf_node.priors.as_ref().unwrap().get(&action).cloned().unwrap_or(0.0);
                    let parent_weak = Rc::downgrade(&leaf_node_rc);
                    let new_child_rc = MctsNode::new_child(next_state, action, parent_weak, prior, move_gen);
                    leaf_node.children.push(new_child_rc.clone());
                    drop(leaf_node); // Release borrow before evaluating child

                    // Evaluate the newly expanded child node
                    let (child_priors, child_value_curr_player) = policy_network.evaluate(&new_child_rc.borrow().state);
                    let child_value_white_pov = if new_child_rc.borrow().state.w_to_move { child_value_curr_player } else { 1.0 - child_value_curr_player };
                    { // Borrow scope for child
                        let mut child_node = new_child_rc.borrow_mut();
                        child_node.nn_value = Some(child_value_white_pov);
                        child_node.store_priors_and_categorize_moves(child_priors, move_gen);
                    }
                    value_to_propagate = Some(child_value_white_pov);
                    node_to_propagate_from = new_child_rc; // Backpropagate from the new child

                } else {
                    // Node was evaluated but somehow fully explored? Error or edge case.
                    // Backpropagate stored value anyway.
                    value_to_propagate = leaf_node.nn_value; // Use stored value
                    node_to_propagate_from = leaf_node_rc.clone();
                    drop(leaf_node); // Release borrow
                }
            }
        }

        // --- 4. Backpropagation ---
        if let Some(value) = value_to_propagate {
            MctsNode::backpropagate(node_to_propagate_from, value);
        } // Else: Error? Or maybe mate search failed beyond depth?

        iteration_count += 1;
    }

    // --- Select Best Move --- (Based on visits, as before)
    // ...
}
Self-Correction: Refined the MCTS loop logic significantly to better match the AlphaZero flow combined with the mate search. Evaluation now happens on the leaf node before expansion, and the evaluated value is backpropagated. Expansion creates one child based on priority.

5.4. Prioritized Selection (select_leaf_for_expansion)
This function needs to traverse the tree using PUCT, but with a modification: if a node has unexplored moves according to the priority categories, it should be selected for expansion before descending further based purely on PUCT of existing children.

// (Location: mcts/mod.rs or mcts/node.rs)
fn select_leaf_for_expansion(
    node: Rc<RefCell<MctsNode>>,
    exploration_constant: f64,
) -> Rc<RefCell<MctsNode>> {
    let mut current_node_rc = node;
    loop {
        let node_borrow = current_node_rc.borrow();

        if node_borrow.terminal_or_mate_value.is_some() || node_borrow.is_terminal() {
            // Reached a terminal node or one decided by mate search - select this node
            drop(node_borrow);
            return current_node_rc;
        }

        // Check if there are unexplored moves according to the current priority category
        if !node_borrow.is_fully_explored() {
             // This node has high-priority unexplored moves, select it for expansion/evaluation
             drop(node_borrow);
             return current_node_rc;
        }

        // All moves in current priority (and higher) are explored.
        // If no children exist at all (shouldn't happen if not terminal), return self.
        if node_borrow.children.is_empty() {
             drop(node_borrow);
             return current_node_rc;
        }

        // Otherwise, select the best *existing* child using PUCT and descend.
        let best_child_rc = node_borrow.select_best_child(exploration_constant); // Uses PUCT
        drop(node_borrow);
        current_node_rc = best_child_rc; // Descend tree
    }
}
Self-Correction: Simplified selection. It uses PUCT on explored children. If it reaches a node that isn't fully explored according to the priority categories, that node is returned immediately for evaluation/expansion.

6. Interfaces
PolicyNetwork Trait: evaluate(board) -> (HashMap<Move, f64>, f64) - Provides priors and value [0,1] for the current player.
Board::apply_move_to_board(&self, Move) -> Board: Needs to be public and correctly implement move application, returning a new state.
mate_search(...) -> (score, Move, nodes): Needs to be accessible. Score indicates mate (e.g., +/- 1M) or no mate found (e.g., 0).
MoveGen: Needs gen_pseudo_legal_moves, is_capture, potentially helpers for categorization (like accessing SEE results if not done within categorization).
MctsNode: Needs methods like store_priors_and_categorize_moves, get_best_unexplored_move, is_fully_explored, advance_priority_category.
7. Trade-offs & Considerations
Performance: Mate search adds overhead. Categorization adds overhead during expansion/evaluation. Selection is closer to standard PUCT.
Tuning: Requires tuning mate_search_depth, MCTS exploration_constant, and move category definitions/priorities.
Complexity: Higher than standard MCTS or the previous policy-based MCTS due to the conditional logic involving mate search and prioritized expansion.
Mate Search Depth: Choosing an appropriate mate_search_depth is crucial.
Policy Priors: Still needed for PUCT calculation among explored children and potentially for tie-breaking within priority categories during expansion.
8. Future Work
Implement Board::apply_move_to_board.
Implement MctsNode helper methods (store_priors_and_categorize_moves, etc.).
Refine the select_leaf_for_expansion logic.
Refactor PestoEval for trainable weights.
Implement a non-uniform policy (heuristic or learned).
Tune parameters.
Add comprehensive tests.
