//! # Chess Engine Library
//!
//! This library implements a classical chess engine using bitboards, magic bitboards
//! for move generation, alpha-beta search with various enhancements (iterative deepening,
//! transposition tables, quiescence search, null move pruning), and a Pesto-style
//! tapered evaluation function.
//!
//! It provides modules for:
//! - Board representation (`board`, `board_utils`, `boardstack`)
//! - Move generation (`move_generation`, `magic_bitboard`, `magic_constants`)
//! - Evaluation (`eval`, `eval_constants`)
//! - Search algorithms (`search`, `transposition`)
//! - UCI protocol handling (`uci`)
//! - Core types and utilities (`move_types`, `piece_types`, `bits`, `hash`, `utils`)
//! - Agent interaction (`agent`)
//! - Memory management (`arena`)

pub mod agent;
pub mod arena;
pub mod board;
pub mod board_utils;
pub mod boardstack;
pub mod bits;
pub mod eval;
pub mod eval_constants;
pub mod hash;
pub mod magic_bitboard;
pub mod magic_constants;
pub mod make_move;
pub mod move_generation;
pub mod move_types;
pub mod piece_types;
pub mod search;
pub mod transposition;
pub mod uci;
pub mod utils;
pub mod mcts;
