//! This module specifies various agents, which can use any combination of search and eval routines.

use crate::boardstack::BoardStack;
use crate::eval::PestoEval;
use crate::move_types::Move;
use crate::move_generation::MoveGen;
use crate::search::{aspiration_window_ab_search, iterative_deepening_ab_search, mate_search};

/// Trait defining the interface for chess agents.
pub trait Agent {
    /// Get the best move for the current board position.
    ///
    /// # Arguments
    ///
    /// * `board` - A mutable reference to the current `Bitboard` position.
    ///
    /// # Returns
    ///
    /// The best `Move` as determined by the agent.
    fn get_move(&self, board: &mut BoardStack) -> Move;
}

/// A simple agent that uses mate search followed by aspiration window quiescence search.
pub struct SimpleAgent<'a> {
    /// The depth to search for mate.
    pub mate_search_depth: i32,
    /// The depth for alpha-beta search.
    pub ab_search_depth: i32,
    /// The maximum depth for the quiescence search.
    pub q_search_max_depth: i32,
    /// Whether to print verbose output during search.
    pub verbose: bool,
    /// Reference to the move generator.
    pub move_gen: &'a MoveGen,
    /// Reference to the Pesto evaluation function.
    pub pesto: &'a PestoEval
}

impl SimpleAgent<'_> {
    /// Creates a new `SimpleAgent` with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `mate_search_depth` - The depth to search for mate.
    /// * `ab_search_depth` - The depth for alpha-beta search.
    /// * `verbose` - Whether to print verbose output during search.
    /// * `move_gen` - Reference to the move generator.
    /// * `pesto` - Reference to the Pesto evaluation function.
    ///
    /// # Returns
    ///
    /// A new `SimpleAgent` instance.
    pub fn new<'a>(mate_search_depth: i32, ab_search_depth: i32, q_search_max_depth: i32, verbose: bool, move_gen: &'a MoveGen, pesto: &'a PestoEval) -> SimpleAgent<'a> {
        SimpleAgent {
            mate_search_depth,
            ab_search_depth,
            q_search_max_depth,
            verbose,
            move_gen,
            pesto
        }
    }
}

impl Agent for SimpleAgent<'_> {
    fn get_move(&self, board: &mut BoardStack) -> Move {
        // First, perform mate search
        let (eval, m, nodes) = mate_search(board, self.move_gen, self.mate_search_depth, self.verbose);
        if eval == 1000000 {
            println!("Found checkmate after searching {} nodes!", nodes);
            return m;
        }

        // If no mate found, perform iterative deepening search
        let (depth, eval, m, n) = iterative_deepening_ab_search(board, self.move_gen, self.pesto, self.ab_search_depth, self.q_search_max_depth, None, self.verbose);
        println!("Mate search searched {} nodes, iterative deepening search searched another {} nodes at a depth of {} ({} total nodes). Eval: {}", nodes, n, depth, nodes + n, eval);
        m
    }
}