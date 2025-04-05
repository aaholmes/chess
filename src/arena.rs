//! This module provides an Arena for staging chess engine matches.

use crate::agent::Agent;
use crate::boardstack::BoardStack;
use crate::utils::print_move;

/// Struct representing an arena for chess engine matches.
pub struct Arena<'a> {
    /// The agent playing as White.
    white_player: &'a dyn Agent,
    /// The agent playing as Black.
    black_player: &'a dyn Agent,
    /// The maximum number of moves allowed in the game.
    max_moves: i32,
    /// The current state of the chess board.
    pub boardstack: BoardStack,
}

impl Arena<'_> {
    /// Creates a new Arena with the specified players and maximum number of moves.
    ///
    /// # Arguments
    ///
    /// * `white_player` - The agent playing as White.
    /// * `black_player` - The agent playing as Black.
    /// * `max_moves` - The maximum number of moves allowed in the game.
    ///
    /// # Returns
    ///
    /// A new `Arena` instance.
    pub fn new<'a>(
        white_player: &'a dyn Agent,
        black_player: &'a dyn Agent,
        max_moves: i32,
    ) -> Arena<'a> {
        Arena {
            white_player,
            black_player,
            max_moves,
            boardstack: BoardStack::new(),
        }
    }

    /// Plays a game between the two agents in the arena.
    ///
    /// This method alternates moves between White and Black players until the maximum
    /// number of moves is reached. It prints the game state after each move.
    pub fn play_game(&mut self) {
        println!("Playing game (max {} moves)", self.max_moves);
        self.boardstack.current_state().print();

        for i in 0..self.max_moves {
            println!("Move {}", i);

            let (current_player, color) = if i % 2 == 0 {
                (self.white_player, "White")
            } else {
                (self.black_player, "Black")
            };

            // Get and make the move for the current player
            let m = current_player.get_move(&mut self.boardstack);
            println!("{} to move: {}", color, print_move(&m));
            self.boardstack.make_move(m);

            // Print the updated board state
            self.boardstack.current_state().print();

            // TODO: Add game termination conditions (checkmate, stalemate, etc.)
        }

        // TODO: Determine and print the game result
    }
}
