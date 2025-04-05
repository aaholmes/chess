//! Defines the Node structure for the MCTS tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering}; // For potential thread-safe node IDs if needed later
use crate::board::Board;
use crate::move_types::Move;
use crate::move_generation::MoveGen;

// Simple unique ID generator for debugging/visualization (optional)
static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(0);

/// Represents a node in the Monte Carlo Tree Search tree.
#[derive(Debug)]
pub struct MctsNode {
    pub id: usize, // Unique ID for debugging
    pub state: Board, // The game state this node represents
    pub parent: Option<Weak<RefCell<MctsNode>>>, // Weak reference to parent to avoid cycles
    pub children: Vec<Rc<RefCell<MctsNode>>>, // Children nodes
    pub action: Option<Move>, // The action (move) taken from parent to reach this node

    // MCTS statistics
    pub visits: u32,
    pub total_value: f64, // Accumulated value (from evaluations or simulations)
    pub policy_prior: f64, // Prior probability of selecting this node (from parent)
    pub value: Option<f64>, // Stored evaluation value (0-1 for current player) when node is first evaluated

    // Expansion control
    pub untried_actions: Vec<Move>, // Moves not yet expanded from this node
    pub is_terminal: bool, // Flag indicating if the state is terminal
}

impl MctsNode {
    /// Creates a new root node.
    pub fn new_root(state: Board, move_gen: &MoveGen) -> Rc<RefCell<MctsNode>> {
        let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
        let (is_terminal, _) = state.is_checkmate_or_stalemate(move_gen); // Check if root is already terminal
        let untried_actions = if is_terminal {
            Vec::new()
        } else {
            // Generate legal moves for the initial state
            let legal_moves = MctsNode::get_legal_moves(&state, move_gen);
            legal_moves
        };

        Rc::new(RefCell::new(MctsNode {
            id,
            state,
            parent: None,
            children: Vec::new(),
            action: None,
            visits: 0,
            total_value: 0.0,
            policy_prior: 0.0, // Root prior doesn't matter for selection from it
            value: None, // Root value is not typically stored/used directly
            untried_actions,
            is_terminal,
        }))
    }

    /// Creates a new child node.
    pub fn new_child(
        state: Board, // This state should be the result of applying 'action' to parent's state
        action: Move,
        parent: Weak<RefCell<MctsNode>>,
        policy_prior: f64, // Add policy prior
        move_gen: &MoveGen,
    ) -> Rc<RefCell<MctsNode>> {
         let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
         // Determine terminal status and legal moves for the *new* state
         let (is_terminal, _) = state.is_checkmate_or_stalemate(move_gen);
         let untried_actions = if is_terminal {
            Vec::new()
         } else {
            MctsNode::get_legal_moves(&state, move_gen)
         };

        Rc::new(RefCell::new(MctsNode {
            id,
            state,
            parent: Some(parent),
            children: Vec::new(),
            action: Some(action),
            visits: 0,
            total_value: 0.0,
            policy_prior, // Store the prior
            value: None, // Value is determined during expansion/evaluation
            untried_actions,
            is_terminal,
        }))
    }

    /// Checks if the node is fully expanded (all possible actions have led to child nodes).
    pub fn is_fully_expanded(&self) -> bool {
        self.untried_actions.is_empty()
    }

    /// Checks if the node represents a terminal game state.
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Calculates the PUCT (Polynomial Upper Confidence Trees) value for this node.
    /// Used during the selection phase.
    pub fn puct_value(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        // Q value: Average value obtained from this node (exploitation term)
        // Value is stored relative to the player *whose turn it is* in this node's state.
        let q_value = if self.visits == 0 {
            0.0 // Default value for unvisited nodes (or could use parent's value)
        } else {
            self.total_value / self.visits as f64
        };

        // U value: Exploration term, biased by policy prior
        let u_value = exploration_constant
            * self.policy_prior
            * (parent_visits as f64).sqrt() / (1.0 + self.visits as f64);

        q_value + u_value
    }
    
    /// Backpropagates the simulation result up the tree.
    ///
    /// # Arguments
    /// * `node_rc` - The node from which the evaluation/simulation started (the expanded node or a terminal leaf).
    /// * `value` - The evaluation result (e.g., 1.0 for win, 0.5 draw, 0.0 loss for White).
    pub fn backpropagate(mut node_rc: Rc<RefCell<MctsNode>>, value: f64) {
        loop {
            let mut node_borrow = node_rc.borrow_mut();
            node_borrow.visits += 1;
            // Value is always from White's perspective (0.0 to 1.0)
            // Add the value corresponding to the player whose turn it *was* when moving *into* this node.
            let reward_to_add = if node_borrow.state.w_to_move {
                 1.0 - value // Black just moved to get here, add Black's score (1 - White's score)
            } else {
                 value // White just moved to get here, add White's score
            };
            node_borrow.total_value += reward_to_add;

    
            // (Reward logic moved up)
    
    
            // Move up to the parent
            let parent_weak_opt = node_borrow.parent.clone(); // Clone Weak ref
            drop(node_borrow); // Release borrow before potentially upgrading Weak ref
    
            if let Some(parent_weak) = parent_weak_opt {
                if let Some(parent_rc) = parent_weak.upgrade() {
                    node_rc = parent_rc; // Move to parent
                } else {
                    break; // Parent has been dropped (shouldn't happen in standard MCTS)
                }
            } else {
                break; // Reached the root node
            }
        }
    }
    /// Selects the best child node according to the UCT formula.
    /// Panics if called on a terminal node or a node with no children.
    /// Assumes children list is non-empty.
    /// Selects the best child node according to the PUCT formula.
    fn select_best_child(&self, exploration_constant: f64) -> Rc<RefCell<MctsNode>> {
        let parent_visits = self.visits;
        self.children
            .iter()
            .max_by(|a, b| {
                // Use puct_value now
                let puct_a = a.borrow().puct_value(parent_visits, exploration_constant);
                let puct_b = b.borrow().puct_value(parent_visits, exploration_constant);
                puct_a.partial_cmp(&puct_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned() // Clone the Rc<RefCell<MctsNode>>
            .expect("max_by should return a value for non-empty children")
    }

    /// Helper function to generate legal moves for a given state.
    /// Filters pseudo-legal moves by checking legality using apply_move_to_board.
    pub fn get_legal_moves(state: &Board, move_gen: &MoveGen) -> Vec<Move> {
        let (captures, moves) = move_gen.gen_pseudo_legal_moves(state);
        let mut legal_moves = Vec::with_capacity(captures.len() + moves.len());
        for m in captures.into_iter().chain(moves.into_iter()) {
            // Use the existing apply_move_to_board method which returns a new Board state.
            // Ensure apply_move_to_board is public in board.rs or its defining module.
            // This check is inefficient if done repeatedly; ideally MoveGen provides legal moves.
            let next_state = state.apply_move_to_board(m); // Use actual method
            if next_state.is_legal(move_gen) { // Check legality of the resulting state
                legal_moves.push(m);
            }
        }
        legal_moves
    }

    /// Expands the current node by adding one child node for an untried action.
    /// Returns the newly created child node.
    /// Panics if called on a fully expanded or terminal node.
    /// Expands the current node using policy network priors.
    /// Adds one child node for an untried action and returns it.
    /// Stores the evaluation value obtained from the policy network in the *parent* node.
    /// Panics if called on a fully expanded or terminal node.
    pub fn expand_with_policy<P: PolicyNetwork>(
        node_rc: Rc<RefCell<MctsNode>>, // The node to expand
        move_gen: &MoveGen,
        policy_network: &P,
    ) -> (Rc<RefCell<MctsNode>>, f64) { // Return new child and the evaluated value
        let mut node_borrow = node_rc.borrow_mut(); // Get mutable borrow

        if node_borrow.is_fully_expanded() || node_borrow.is_terminal() {
            panic!("expand called on fully expanded or terminal node");
        }

        // Get policy priors and value for the current node's state *before* choosing action
        // This evaluation happens only once when expanding the first child.
        let (priors, value) = if node_borrow.value.is_none() {
             let (p, v) = policy_network.evaluate(&node_borrow.state);
             // Store the value (relative to current player)
             // Convert value from policy network (0-1 for current player) to be relative to White (0-1)
             let value_white_pov = if node_borrow.state.w_to_move { v } else { 1.0 - v };
             node_borrow.value = Some(value_white_pov);
             (p, value_white_pov)
        } else {
             // Should not happen if called correctly, but return empty priors and stored value
             (HashMap::new(), node_borrow.value.unwrap())
        };


        // Choose the next untried action
        // TODO: Consider sampling based on priors instead of just popping? Standard MCTS pops.
        let action = node_borrow.untried_actions.pop().expect("Untried actions should not be empty");

        // Get the prior for this specific action
        let prior = *priors.get(&action).unwrap_or(&0.0); // Default to 0 if move not in policy map (shouldn't happen for legal moves)

        // Create the next state by applying the action
        let next_state = node_borrow.state.apply_move_to_board(action);

        // Create the new child node, passing a Weak reference to the parent (self)
        let parent_weak = Rc::downgrade(&node_rc);
        let new_child = MctsNode::new_child(next_state, action, parent_weak, prior, move_gen); // Pass prior

        // Add the new child to the current node's children
        node_borrow.children.push(new_child.clone()); // Clone Rc for ownership

        drop(node_borrow); // Release mutable borrow

        (new_child, value) // Return the new child and the evaluated value
    }
}

// --- MCTS Core Logic Helpers --- (Can be moved to a separate file/struct later)

/// Selects a leaf node starting from the given node using UCT.
/// Traverses the tree, choosing the child with the highest UCT value at each step,
/// until a node is reached that is either terminal or not fully expanded.
/// Selects a leaf node starting from the given node using PUCT.
pub fn select_leaf(node: Rc<RefCell<MctsNode>>, exploration_constant: f64) -> Rc<RefCell<MctsNode>> {
    let mut current_node = node;
    loop {
        let node_borrow = current_node.borrow();
        if node_borrow.is_terminal() || !node_borrow.is_fully_expanded() {
            // Reached a terminal node or a node that can be expanded
            drop(node_borrow); // Release borrow before returning
            break;
        }
        // Node is fully expanded and non-terminal, so select the best child
        let best_child = node_borrow.select_best_child(exploration_constant); // Uses PUCT now
        drop(node_borrow); // Release borrow before moving ownership in next iteration
        current_node = best_child;
    }
    current_node // Return the leaf node (Rc<RefCell<MctsNode>>)
}

// Note: Expansion is now handled via MctsNode::expand, called from the main loop after selection.
// The `expand_leaf` helper function is removed as its logic is better placed in the main loop.