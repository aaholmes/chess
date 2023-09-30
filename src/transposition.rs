// Transposition table for storing previously seen positions

use std::collections::HashMap;
use std::hash::Hash;
use crate::bitboard::Bitboard;
use crate::gen_moves::Move;

#[derive(PartialEq)]
struct TranspositionEntry {
    depth: i32,
    score: i32,
    best_move: Option<Move>,
}

struct TranspositionTable {
    table: HashMap<Bitboard, TranspositionEntry>,
}

impl TranspositionTable {
    pub fn check_table(&self, board: &Bitboard, depth: i32) -> Option<&TranspositionEntry> {
        // Checks the table for a given board position and search depth
        // If it exists, returns a reference to the entry
        // Else, returns None
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
    // For the positions, we care about side to move and castling and en passant ability, but not halfmove clock or fullmove clock
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