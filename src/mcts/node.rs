//! Defines the Node structure for the MCTS tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::f64;
use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;

/// A node in the Monte Carlo Search Tree
#[derive(Debug)]
pub struct MctsNode {
    /// The chess position at this node
    pub state: Board,
    
    /// The move that led to this state (None for root)
    pub action: Option<Move>,
    
    /// Number of times this node has been visited
    pub visits: u32,
    
    /// Total reward accumulated through this node (from white's perspective)
    pub total_value: f64,
    
    /// Reference to parent node (None for root)
    pub parent: Option<Weak<RefCell<MctsNode>>>,
    
    /// Child nodes
    pub children: Vec<Rc<RefCell<MctsNode>>>,
    
    /// Actions not yet expanded into child nodes
    pub untried_actions: Vec<Move>,
    
    /// Whether this is a terminal state (checkmate, stalemate)
    pub is_terminal: bool,
}

impl MctsNode {
    /// Creates a new root node for MCTS
    pub fn new_root(state: Board, move_gen: &MoveGen) -> Rc<RefCell<Self>> {
        // Check if the state is already terminal
        let (is_checkmate, is_stalemate) = state.is_checkmate_or_stalemate(move_gen);
        let is_terminal = is_checkmate || is_stalemate;
        
        // Generate legal moves if not terminal
        let mut untried_actions = Vec::new();
        if !is_terminal {
            // Get pseudo-legal moves
            let (captures, moves) = move_gen.gen_pseudo_legal_moves(&state);
            
            // Combine and filter for legal moves
            for m in captures {
                let new_board = state.apply_move_to_board(m);
                if new_board.is_legal(move_gen) {
                    untried_actions.push(m);
                }
            }
            
            for m in moves {
                let new_board = state.apply_move_to_board(m);
                if new_board.is_legal(move_gen) {
                    untried_actions.push(m);
                }
            }
        }
        
        Rc::new(RefCell::new(Self {
            state,
            action: None,
            visits: 0,
            total_value: 0.0,
            parent: None,
            children: Vec::new(),
            untried_actions,
            is_terminal,
        }))
    }
    
    /// Creates a new child node from a parent node and a move
    fn new_child(
        parent: Weak<RefCell<MctsNode>>, 
        parent_state: &Board, 
        action: Move, 
        move_gen: &MoveGen
    ) -> Rc<RefCell<Self>> {
        // Create new board by applying the move
        let new_state = parent_state.apply_move_to_board(action);
        
        // Check if the state is terminal
        let (is_checkmate, is_stalemate) = new_state.is_checkmate_or_stalemate(move_gen);
        let is_terminal = is_checkmate || is_stalemate;
        
        // Generate legal moves if not terminal
        let mut untried_actions = Vec::new();
        if !is_terminal {
            // Get pseudo-legal moves
            let (captures, moves) = move_gen.gen_pseudo_legal_moves(&new_state);
            
            // Combine and filter for legal moves
            for m in captures {
                let next_board = new_state.apply_move_to_board(m);
                if next_board.is_legal(move_gen) {
                    untried_actions.push(m);
                }
            }
            
            for m in moves {
                let next_board = new_state.apply_move_to_board(m);
                if next_board.is_legal(move_gen) {
                    untried_actions.push(m);
                }
            }
        }
        
        Rc::new(RefCell::new(Self {
            state: new_state,
            action: Some(action),
            visits: 0,
            total_value: 0.0,
            parent: Some(parent),
            children: Vec::new(),
            untried_actions,
            is_terminal,
        }))
    }
    
    /// Returns whether this node represents a terminal game state
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }
    
    /// UCT score for a child node (used for selection)
    pub fn uct_value(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY; // Unexplored nodes have highest priority
        }
        
        // Convert total value to average value (from current player's perspective)
        let exploitation_term = if self.state.w_to_move {
            self.total_value / self.visits as f64 // White is maximizing
        } else {
            1.0 - (self.total_value / self.visits as f64) // Black is minimizing (from white's perspective)
        };
        
        // Exploration term
        let exploration_term = exploration_constant * 
            ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        
        exploitation_term + exploration_term
    }
    
    /// Expands the tree by adding a new child node
    pub fn expand(
        node: Rc<RefCell<MctsNode>>, 
        move_gen: &MoveGen
    ) -> Rc<RefCell<MctsNode>> {
        // Borrow the node to get access to its fields
        let mut node_ref = node.borrow_mut();
        
        // If no untried actions or terminal node, return the node itself
        if node_ref.untried_actions.is_empty() || node_ref.is_terminal {
            return node.clone();
        }
        
        // Pick a random untried action using thread_rng
        let idx = rand::random::<usize>() % node_ref.untried_actions.len();
        let action = node_ref.untried_actions.remove(idx);
        
        // Create the current state
        let state = node_ref.state.clone();
        
        // Create weak ref to parent for the child
        let parent_ref = Rc::downgrade(&node);
        
        // Create child node (drop borrow before creating to avoid nested borrow)
        drop(node_ref);
        
        let child = Self::new_child(parent_ref, &state, action, move_gen);
        
        // Borrow again to add child
        node.borrow_mut().children.push(child.clone());
        
        child
    }
    
    /// Backpropagate simulation results up the tree
    pub fn backpropagate(node: Rc<RefCell<MctsNode>>, result: f64) {
        let mut current = Some(node);
        
        while let Some(n) = current {
            let mut node_ref = n.borrow_mut();
            node_ref.visits += 1;
            node_ref.total_value += result;
            
            current = match &node_ref.parent {
                Some(weak_parent) => weak_parent.upgrade(),
                None => None,
            };
        }
    }
}

/// Select a leaf node from the tree using UCT selection policy
pub fn select_leaf(root: Rc<RefCell<MctsNode>>, exploration_constant: f64) -> Rc<RefCell<MctsNode>> {
    let mut current = root.clone();
    
    loop {
        // If the current node has untried actions, it's a leaf
        if !current.borrow().untried_actions.is_empty() {
            break;
        }
        
        // If the current node has no children or is terminal, it's a leaf
        if current.borrow().children.is_empty() || current.borrow().is_terminal() {
            break;
        }
        
        // Find the child with the highest UCT score
        let parent_visits = current.borrow().visits;
        
        // Using a separate scope to drop borrow before recursive call
        let best_child = {
            let children = &current.borrow().children;
            
            // Find child with highest UCT score
            children
                .iter()
                .max_by(|a, b| {
                    let a_score = a.borrow().uct_value(parent_visits, exploration_constant);
                    let b_score = b.borrow().uct_value(parent_visits, exploration_constant);
                    a_score.partial_cmp(&b_score).unwrap()
                })
                .unwrap()
                .clone()
        };
        
        current = best_child;
    }
    
    current
}