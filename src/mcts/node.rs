//! Defines the Node structure for the MCTS tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::f64;
use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use crate::piece_types::{QUEEN}; // For promotion check in categorization
use std::collections::HashMap; // For priors and categorized moves
// use super::policy::PolicyNetwork; // Not needed directly in this file if PolicyNetwork passed to mcts_search
// use crate::search::see::see; // Import SEE if used for categorization

// Define Move Categories (Lower discriminant = higher priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MoveCategory {
    // ForcedMate,      // Not used directly here, handled by terminal_or_mate_value
    // Simplified Categories for now:
    Capture = 1, // Any capture (could refine with SEE later)
    Quiet   = 2, // Any non-capture
    // PromotionToQueen = 1,
    // WinningCapture   = 2,  // SEE > threshold
    // KillerMove1      = 3,  // Placeholder category
    // KillerMove2      = 4,  // Placeholder category
    // EqualCapture     = 5,  // abs(SEE) <= threshold
    // Check            = 6,  // Non-capture, non-promo, non-killer check
    // OtherQuiet       = 8,
    // LosingCapture    = 9,  // SEE < -threshold
}

/// A node in the Monte Carlo Search Tree
#[derive(Debug)]
pub struct MctsNode {
    /// The chess position at this node
    pub state: Board,

    /// The move that led to this state (None for root)
    pub action: Option<Move>,

    /// Number of times this node has been visited
    pub visits: u32,

    /// Total value accumulated through this node (always from White's perspective, 0.0 to 1.0)
    pub total_value: f64,

    // --- Evaluation / Mate Status ---
    /// Stores the exact value if determined by terminal state check or mate search (0.0, 0.5, 1.0 for White).
    /// Also used as a flag to indicate if mate search has been performed. None = not checked, Some(-999.0) = checked, no mate.
    pub terminal_or_mate_value: Option<f64>,
    /// Stores the value from the policy/evaluation network (0.0 to 1.0 for White) when node is first evaluated.
    pub nn_value: Option<f64>,
    /// Stores the policy priors for all legal moves from this state, evaluated once.
    pub policy_priors: Option<HashMap<Move, f64>>,

    /// Reference to parent node (None for root)
    pub parent: Option<Weak<RefCell<MctsNode>>>,
    /// Child nodes (explored actions)
    pub children: Vec<Rc<RefCell<MctsNode>>>,

    // --- Expansion / Selection Control ---
    /// Stores unexplored legal moves, categorized by priority. Populated once after evaluation.
    pub unexplored_moves_by_cat: HashMap<MoveCategory, Vec<Move>>,
    /// Tracks the current highest-priority category being explored. Initialized after evaluation.
    pub current_priority_category: Option<MoveCategory>,
    // `untried_actions` Vec removed, replaced by the map above.

    /// Whether this is a terminal state (checkmate, stalemate) - based on initial check
    pub is_terminal: bool,
}

impl MctsNode {
    /// Creates a new root node for MCTS
    pub fn new_root(state: Board, move_gen: &MoveGen) -> Rc<RefCell<Self>> {
        // Check if the state is already terminal
        let (is_checkmate, is_stalemate) = state.is_checkmate_or_stalemate(move_gen);
        let is_terminal = is_checkmate || is_stalemate;

        // Determine initial terminal/mate value
        let initial_terminal_value = if is_stalemate {
            Some(0.5)
        } else if is_checkmate {
            Some(if state.w_to_move { 0.0 } else { 1.0 }) // Mate value from White's perspective
        } else {
            None // Not terminal initially
        };

        Rc::new(RefCell::new(Self {
            state,
            action: None,
            visits: 0,
            total_value: 0.0,
            parent: None,
            children: Vec::new(),
            terminal_or_mate_value: initial_terminal_value,
            nn_value: None, // Evaluated later
            policy_priors: None, // Evaluated later
            unexplored_moves_by_cat: HashMap::new(), // Populated later
            current_priority_category: None, // Set later
            is_terminal, // Store initial terminal status
        }))
    }

    /// Creates a new child node. Note: Priors/Categorization happen later.
    pub fn new_child( // Made public
        parent: Weak<RefCell<MctsNode>>,
        action: Move,
        new_state: Board, // Pass the already calculated new state
        move_gen: &MoveGen
    ) -> Rc<RefCell<Self>> {
        // Check if the new state is terminal
        let (is_checkmate, is_stalemate) = new_state.is_checkmate_or_stalemate(move_gen);
        let is_terminal = is_checkmate || is_stalemate;

        // Determine initial terminal/mate value
        let initial_terminal_value = if is_stalemate {
            Some(0.5)
        } else if is_checkmate {
            Some(if new_state.w_to_move { 0.0 } else { 1.0 }) // Mate value from White's perspective
        } else {
            None // Not terminal initially
        };

        Rc::new(RefCell::new(Self {
            state: new_state,
            action: Some(action),
            visits: 0,
            total_value: 0.0,
            parent: Some(parent),
            children: Vec::new(),
            terminal_or_mate_value: initial_terminal_value,
            nn_value: None,
            policy_priors: None,
            unexplored_moves_by_cat: HashMap::new(),
            current_priority_category: None,
            is_terminal,
        }))
    }

    /// Returns true if this node's state is terminal (checkmate/stalemate).
    pub fn is_game_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Returns true if the node has been evaluated (by NN or mate search) OR is terminal.
    pub fn is_evaluated_or_terminal(&self) -> bool {
        self.nn_value.is_some() || self.terminal_or_mate_value.is_some()
    }

    /// Checks if all move categories have been fully explored.
    pub fn is_fully_explored(&self) -> bool {
        // Considered fully explored if categorization has happened and the map is empty.
        // If policy_priors is None, it hasn't been evaluated/categorized yet.
        self.policy_priors.is_some() && self.unexplored_moves_by_cat.is_empty()
    }

    /// UCT value calculation - used for selecting among *explored* children.
    /// Note: This uses standard UCT. PUCT logic is implicitly handled by the
    /// prioritized expansion strategy, which determines *which* nodes get expanded,
    /// rather than modifying the selection value formula itself.
    pub fn uct_value(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY; // Should not happen if selecting among explored children
        }

        // Q value: Average value from White's perspective
        let avg_value_white = self.total_value / self.visits as f64;

        // Adjust Q value based on whose turn it is at the *parent* node (who is selecting)
        let q_value_for_selector = if let Some(parent_weak) = &self.parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                 // If parent was White, use Q directly. If parent was Black, use 1-Q.
                 if parent_rc.borrow().state.w_to_move { avg_value_white } else { 1.0 - avg_value_white }
            } else { 0.5 } // Parent dropped? Default to draw value
        } else { 0.5 }; // Root node? Should not be selected via UCT. Default draw.


        // U value: Exploration term (standard UCT)
        let exploration_term = exploration_constant * ((parent_visits as f64).ln() / self.visits as f64).sqrt();

        q_value_for_selector + exploration_term
    }

    /// Selects the best *explored* child node according to the UCT formula.
    /// Panics if called on a terminal node or a node with no children.
    pub fn select_best_explored_child(&self, exploration_constant: f64) -> Rc<RefCell<MctsNode>> {
        let parent_visits = self.visits;
        self.children
            .iter()
            .max_by(|a, b| {
                let uct_a = a.borrow().uct_value(parent_visits, exploration_constant);
                let uct_b = b.borrow().uct_value(parent_visits, exploration_constant);
                uct_a.partial_cmp(&uct_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .expect("select_best_explored_child called on node with no children")
    }

    /// Stores policy priors and categorizes unexplored moves after evaluation.
    /// Called once when a node is first evaluated by the policy network.
    pub fn store_priors_and_categorize_moves(
        &mut self,
        priors: HashMap<Move, f64>,
        move_gen: &MoveGen,
        // Pass killers if needed for categorization
        // killers: &[[Move; 2]; MAX_PLY],
        // ply: usize,
    ) {
        if self.policy_priors.is_some() { return; } // Already done

        self.policy_priors = Some(priors);
        let legal_moves = MctsNode::get_legal_moves(&self.state, move_gen); // Regenerate legal moves
        let mut categorized_moves: HashMap<MoveCategory, Vec<Move>> = HashMap::new();

        // TODO: Implement move categorization logic here more thoroughly
        for mv in legal_moves {
            let category = self.categorize_move(&mv, move_gen); // Implement this helper
            categorized_moves.entry(category).or_default().push(mv);
        }

        // Sort categories by priority to determine the starting category
        let mut sorted_categories: Vec<_> = categorized_moves.keys().cloned().collect();
        sorted_categories.sort(); // Sorts by enum discriminant (priority)

        self.unexplored_moves_by_cat = categorized_moves;
        self.current_priority_category = sorted_categories.first().cloned();
    }

    /// Helper to categorize a single move (implement heuristic checks here).
    /// TODO: Needs access to killers, history, SEE results for proper categorization.
    fn categorize_move(&self, mv: &Move, _move_gen: &MoveGen) -> MoveCategory {
        // Basic categorization example
        if mv.promotion.is_some() && mv.promotion == Some(QUEEN) {
            return MoveCategory::PromotionToQueen;
        }
        
        // Check if the move is a capture by checking the destination square
        let to_square = mv.to;
        let opponent_color = !self.state.w_to_move as usize;
        
        // Check if there's an opponent piece at the destination square
        // We'll check each piece type
        let mut opponent_pieces = 0u64;
        for piece in 0..6 {
            opponent_pieces |= self.state.get_piece_bitboard(opponent_color, piece);
        }
        
        if (1u64 << to_square) & opponent_pieces != 0 {
            return MoveCategory::EqualCapture; // Default capture category for now
        }
        
        // Default for quiet moves
        MoveCategory::OtherQuiet // Default
    }

    /// Gets the highest-priority unexplored move and removes it from the map.
    /// Advances the current priority category if necessary.
    pub fn get_best_unexplored_move(&mut self) -> Option<Move> {
        while let Some(current_cat) = self.current_priority_category {
            if let Some(moves_in_cat) = self.unexplored_moves_by_cat.get_mut(&current_cat) {
                // TODO: Could sort moves within category by policy prior before popping
                if let Some(mv) = moves_in_cat.pop() {
                    // If category becomes empty after pop, remove it and advance category
                    if moves_in_cat.is_empty() {
                        self.unexplored_moves_by_cat.remove(&current_cat);
                        self.advance_priority_category();
                    }
                    return Some(mv);
                } else {
                    // Category was already empty, remove it and advance
                    self.unexplored_moves_by_cat.remove(&current_cat);
                    self.advance_priority_category();
                    // Continue loop to check next category
                }
            } else {
                 // Category key exists but no vector? Should not happen. Advance.
                 self.advance_priority_category();
                 // Continue loop
            }
        }
        None // No unexplored moves left in any category
    }

    /// Advances `current_priority_category` to the next non-empty one.
    fn advance_priority_category(&mut self) {
         if let Some(current_cat) = self.current_priority_category {
             // Find the next category in the sorted order
             let mut next_cat_found = false;
             // Iterate through simplified enum variants by discriminant value
             for cat_num in (current_cat as usize + 1)..=(MoveCategory::Quiet as usize) {
                  // This unsafe transmute assumes a standard C-like enum layout (safe for simple cases)
                  let next_possible_cat = unsafe { std::mem::transmute::<u8, MoveCategory>(cat_num as u8) };
                  if self.unexplored_moves_by_cat.contains_key(&next_possible_cat) {
                      self.current_priority_category = Some(next_possible_cat);
                      next_cat_found = true;
                      break;
                  }
             }
             if !next_cat_found {
                 self.current_priority_category = None; // All categories exhausted
             }
         }
    }

    /// Backpropagates a simulation result through the tree.
    /// Updates visits and values for each node from the given leaf to the root.
    pub fn backpropagate(node: Rc<RefCell<MctsNode>>, value: f64) {
        let mut current_node_opt = Some(node);
        
        while let Some(current_node_rc) = current_node_opt {
            // Update statistics for the current node
            {
                let mut current_node = current_node_rc.borrow_mut();
                current_node.visits += 1;
                current_node.total_value += value;
            }
            
            // Move to parent if there is one
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
}

/// Backtracks up the tree in the selection phase to find a node ready for expansion.
/// Checks each node on the path for expansion opportunity before selection continues.
fn should_expand_not_select(node: &MctsNode) -> bool {
    // If this node hasn't been categorized yet, it's not ready for expansion
    if node.policy_priors.is_none() {
        return false;
    }
    
    // If this node still has unexplored moves, it's ready for expansion
    if !node.unexplored_moves_by_cat.is_empty() {
        return true;
    }
    
    // If all moves have been explored, continue selection through this node
    false
}

/// Selects a leaf node for expansion in MCTS.
pub fn select_leaf_for_expansion(root: Rc<RefCell<MctsNode>>, exploration_constant: f64) -> Rc<RefCell<MctsNode>> {
    let mut current = root;
    
    loop {
        // Check if this node is terminal or ready for expansion
        let should_expand;
        {
            let node_borrow = current.borrow();
            if node_borrow.terminal_or_mate_value.is_some() || node_borrow.is_terminal {
                break;
            }
            should_expand = should_expand_not_select(&node_borrow);
        }
        
        if should_expand {
            break; // This node has unexplored moves, stop here for expansion
        }
        
        // If no children yet but we decided not to expand, there's likely an issue
        let has_children = !current.borrow().children.is_empty();
        if !has_children {
            break; // Leaf node reached
        }
        
        // Select child with highest UCT score
        let next = current.borrow().select_best_explored_child(exploration_constant);
        current = next;
    }
    
    current
}

// Note: Expansion logic is now split between MctsNode helpers and the main mcts_search loop.