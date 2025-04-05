//! Transposition table for storing previously seen positions.
//!
//! This module implements a transposition table, which is used to cache and retrieve
//! information about previously analyzed chess positions, improving search efficiency.

use crate::board::Board;
use crate::move_types::Move;
use std::collections::HashMap;

/// Represents an entry in the transposition table.
#[derive(PartialEq)]
pub struct TranspositionEntry {
    /// The depth at which this position was searched.
    pub(crate) depth: i32,
    /// The evaluation score for this position.
    pub(crate) score: i32,
    /// The best move found for this position.
    pub(crate) best_move: Move,
}

/// A transposition table for caching chess positions and their evaluations.
pub struct TranspositionTable {
    /// The underlying hash map storing positions and their corresponding entries.
    table: HashMap<u64, TranspositionEntry>,
}

impl TranspositionTable {
    /// Creates a new transposition table.
    pub fn new() -> Self {
        TranspositionTable {
            table: HashMap::new(),
        }
    }

    /// Checks the table for a given board position and search depth.
    ///
    /// # Arguments
    ///
    /// * `board` - A reference to the `Bitboard` position to look up.
    /// * `depth` - The current search depth.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `TranspositionEntry` if found and the stored depth
    /// is greater than or equal to the current depth, otherwise `None`.
    pub fn probe(&self, board: &Board, depth: i32) -> Option<&TranspositionEntry> {
        // Check the table for a given board position and search depth
        // If it exists, return a reference to the entry
        // Else, return None
        let out = self.table.get(&board.zobrist_hash);
        if out == None {
            return None;
        }
        let entry = out.unwrap();
        if entry.depth >= depth {
            Some(entry)
        } else {
            None
        }
    }

    /// Adds a position to the transposition table or updates an existing entry.
    ///
    /// # Arguments
    ///
    /// * `board` - A reference to the `Bitboard` position to store.
    /// * `depth` - The depth at which this position was searched.
    /// * `score` - The evaluation score for this position.
    /// * `best_move` - The best move found for this position, if any.
    pub fn store(&mut self, board: &Board, depth: i32, score: i32, best_move: Move) {
        // Add a position to the table
        // If the position already exists, update it if the depth is greater
        let entry = self.table.get(&board.zobrist_hash);
        if entry == None {
            self.table.insert(
                board.zobrist_hash,
                TranspositionEntry {
                    depth,
                    score,
                    best_move,
                },
            );
        } else {
            let entry = entry.unwrap();
            if depth > entry.depth {
                self.table.insert(
                    board.zobrist_hash,
                    TranspositionEntry {
                        depth,
                        score,
                        best_move,
                    },
                );
            }
        }
    }

    /// Clears the transposition table.
    pub fn clear(&mut self) {
        self.table.clear();
    }
}
