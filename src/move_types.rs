//! Defines the Move struct and related methods for representing chess moves.
//!
//! This module provides the core `Move` type used throughout the chess engine
//! to represent and manipulate chess moves.

use std::fmt;
use crate::board_utils::sq_ind_to_algebraic;
use crate::piece_types::{KNIGHT, BISHOP, ROOK, QUEEN};

/// Represents a chess move.
///
/// This struct contains information about the source square, destination square,
/// and any promotion that occurs as a result of the move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    /// The index of the square the piece is moving from (0-63).
    pub from: usize,
    /// The index of the square the piece is moving to (0-63).
    pub to: usize,
    /// The type of piece to promote to, if this move results in a promotion.
    /// `None` if the move does not result in a promotion.
    pub promotion: Option<usize>
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub(crate) fn default() -> CastlingRights {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

/// A constant representing a null move, used in search algorithms
pub const NULL_MOVE: Move = Move { from: 0, to: 0, promotion: None };

impl Move {
    /// Creates a new `Move` instance.
    ///
    /// # Arguments
    ///
    /// * `from` - The index of the source square (0-63).
    /// * `to` - The index of the destination square (0-63).
    /// * `promotion` - The type of piece to promote to, if applicable. Use `None` if not a promotion.
    ///
    /// # Returns
    ///
    /// A new `Move` instance with the specified parameters.
    pub fn new(from: usize, to: usize, promotion: Option<usize>) -> Move {
        // New move
        Move {
            from,
            to,
            promotion
        }
    }

    /// Creates a new `Move` instance from a UCI string.
    ///
    /// # Arguments
    ///
    /// * `uci` - A string representing the move in UCI format (e.g., "e2e4", "e7e8q").
    ///
    /// # Returns
    ///
    /// An `Option<Move>` which is `Some(Move)` if the UCI string is valid, or `None` if it's invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use kingfisher::move_types::Move;
    /// let move1 = Move::from_uci("e2e4").unwrap();
    /// let move2 = Move::from_uci("e7e8q").unwrap(); // Promotion to queen
    /// assert!(Move::from_uci("invalid").is_none());
    /// ```
    pub fn from_uci(uci: &str) -> Option<Move> {
        if uci.len() < 4 || uci.len() > 5 {
            return None;
        }

        let from_file = (uci.chars().nth(0)? as u8).wrapping_sub(b'a');
        let from_rank = (uci.chars().nth(1)? as u8).wrapping_sub(b'1');
        let to_file = (uci.chars().nth(2)? as u8).wrapping_sub(b'a');
        let to_rank = (uci.chars().nth(3)? as u8).wrapping_sub(b'1');

        if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
            return None;
        }

        let from: usize = (from_rank * 8 + from_file) as usize;
        let to: usize = (to_rank * 8 + to_file) as usize;

        let promotion = if uci.len() == 5 {
            // Make sure the promotion is on the last rank
            if to_rank != 7 && to_rank != 0 {
                return None;
            }
            match uci.chars().nth(4)? {
                'n' => Some(KNIGHT),
                'b' => Some(BISHOP),
                'r' => Some(ROOK),
                'q' => Some(QUEEN),
                _ => return None,
            }
        } else {
            None
        };

        Some(Move { from, to, promotion })
    }

    /// Creates a null move.
    ///
    /// A null move is a special move used in chess engines to pass the turn
    /// without making an actual move on the board. It's typically used in
    /// null move pruning and other search techniques.
    ///
    /// # Returns
    ///
    /// A `Move` instance representing a null move, with `from` and `to` set to 0
    /// and `promotion` set to `None`.
    pub fn null() -> Move {
        // Null move
        Move {
            from: 0,
            to: 0,
            promotion: None
        }
    }

    /// Change the way a move is printed so that it uses algebraic notation
    pub fn print_algebraic(&self) -> String {
        let from = sq_ind_to_algebraic(self.from);
        let to = sq_ind_to_algebraic(self.to);
        let mut promotion = String::from("");
        if self.promotion.is_some() {
            promotion = String::from("=");
            match self.promotion.unwrap() {
                n if n == KNIGHT => promotion.push('N'),
                n if n == BISHOP => promotion.push('B'),
                n if n == ROOK => promotion.push('R'),
                n if n == QUEEN => promotion.push('Q'),
                _ => panic!("Invalid promotion piece")
            }
        }
        format!("{}{}{}", from, to, promotion)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_from_uci() {
        assert_eq!(Move::from_uci("e2e4"), Some(Move { from: 12, to: 28, promotion: None }));
        assert_eq!(Move::from_uci("a7a8q"), Some(Move { from: 48, to: 56, promotion: Some(QUEEN) }));
        assert_eq!(Move::from_uci("h2h1n"), Some(Move { from: 15, to: 7, promotion: Some(KNIGHT) }));
        assert_eq!(Move::from_uci("e1g1"), Some(Move { from: 4, to: 6, promotion: None })); // Castling
        assert_eq!(Move::from_uci("invalid"), None);
        assert_eq!(Move::from_uci("e2e9"), None); // Invalid square
        assert_eq!(Move::from_uci("e2e4q"), None); // Invalid promotion (not on last rank)
    }
}