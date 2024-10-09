//! Transposition table for storing previously seen positions.
//!
//! This module implements a transposition table, which is used to cache and retrieve
//! information about previously analyzed chess positions, improving search efficiency.

use std::collections::HashMap;
use std::hash::Hash;
use crate::bitboard::Bitboard;
use crate::gen_moves::Move;

/// Represents an entry in the transposition table.
#[derive(PartialEq)]
struct TranspositionEntry {
    /// The depth at which this position was searched.
    depth: i32,
    /// The evaluation score for this position.
    score: i32,
    /// The best move found for this position, if any.
    best_move: Option<Move>,
}

/// A transposition table for caching chess positions and their evaluations.
struct TranspositionTable {
    /// The underlying hash map storing positions and their corresponding entries.
    table: HashMap<Bitboard, TranspositionEntry>,
}

impl TranspositionTable {
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
    pub fn check_table(&self, board: &Bitboard, depth: i32) -> Option<&TranspositionEntry> {
        // Check the table for a given board position and search depth
        // If it exists, return a reference to the entry
        // Else, return None
        let out = self.table.get(board);
        if out == None {
            return None;
        }
        let entry = out.unwrap();
        if entry.depth >= depth {
            return Some(entry);
        } else {
            return None;
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
    pub fn add_position(&mut self, board: &Bitboard, depth: i32, score: i32, best_move: Option<Move>) {
        // Add a position to the table
        // If the position already exists, update it if the depth is greater
        let entry = self.table.get(board);
        if entry == None {
            self.table.insert(board.clone(), TranspositionEntry {depth, score, best_move});
        } else {
            let entry = entry.unwrap();
            if depth > entry.depth {
                self.table.insert(board.clone(), TranspositionEntry {depth, score, best_move});
            }
        }
    }
}

impl Hash for Bitboard {
    /// Implements the `Hash` trait for `Bitboard`.
    ///
    /// This function defines how a `Bitboard` is hashed for use in the transposition table.
    /// It considers the pieces, side to move, castling rights, and en passant ability,
    /// but not the halfmove or fullmove clocks.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pieces.hash(state);
        self.w_to_move.hash(state);
        self.w_castle_k.hash(state);
        self.w_castle_q.hash(state);
        self.b_castle_k.hash(state);
        self.b_castle_q.hash(state);
        self.en_passant.hash(state);
    }
}