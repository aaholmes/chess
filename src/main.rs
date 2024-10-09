//! Main entry point for the Kingfisher chess engine.
//!
//! This module sets up the chess engine components and runs a sample game
//! between two simple agents.

mod agent;
use agent::{Agent, SimpleAgent};
mod arena;
use arena::Arena;
mod bitboard;
mod bits;
mod eval;
use eval::PestoEval;
mod move_types;
mod move_generation;
use move_generation::MoveGen;
mod magic_bitboard;
mod magic_constants;
mod make_move;
mod search;
mod transposition;
mod utils;

/// The main function that sets up and runs a sample chess game.
///
/// This function initializes the necessary components of the chess engine,
/// creates two simple agents, and runs a game between them in an arena.
fn main() {
    // Initialize the move generator
    let move_gen = MoveGen::new();

    // Initialize the Pesto evaluation function
    let pesto = PestoEval::new();

    // Create a simple agent for White
    let white = SimpleAgent::new(5, 2, false, &move_gen, &pesto);

    // Create a simple agent for Black
    let black = SimpleAgent::new(5, 2, false, &move_gen, &pesto);

    // Create an arena for the game with a maximum of 10 moves
    let mut arena = Arena::new(&white, &black, 10);

    // Play the game
    arena.play_game();

    // Print the final board state
    arena.board.print();
}