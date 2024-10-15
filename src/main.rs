//! Main entry point for the Kingfisher chess engine.
//!
//! This module sets up the chess engine components and runs a sample game
//! between two simple agents.

extern crate kingfisher;
use kingfisher::agent::SimpleAgent;
use kingfisher::arena::Arena;
use kingfisher::eval::PestoEval;
use kingfisher::move_generation::MoveGen;

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
    let white = SimpleAgent::new(3, 6, 3, false, &move_gen, &pesto);

    // Create a simple agent for Black
    let black = SimpleAgent::new(3, 6, 3, false, &move_gen, &pesto);

    // Create an arena for the game with a maximum of 10 moves
    let mut arena = Arena::new(&white, &black, 40);

    // Play the game
    arena.play_game();

    // Print the final board state
    arena.boardstack.current_state().print();
}
