//! This module defines the Bitboard structure and associated functions for chess board representation.

use std::collections::HashMap;
use crate::move_generation::MoveGen;
use crate::piece_types::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};

/// Represents the chess board using bitboards.
///
/// Each piece type and color has its own 64-bit unsigned integer,
/// where each bit represents a square on the chess board.
#[derive(Clone, Eq, PartialEq)]
pub struct Bitboard {
    /// Game result: None = in progress, Some(1) = white wins, Some(-1) = black wins, Some(0) = draw
    pub game_result: Option<i32>,
    /// True if it's White's turn to move
    pub w_to_move: bool,
    /// White kingside castling rights
    pub w_castle_k: bool,
    /// White queenside castling rights
    pub w_castle_q: bool,
    /// Black kingside castling rights
    pub b_castle_k: bool,
    /// Black queenside castling rights
    pub b_castle_q: bool,
    /// Index of square where en passant is possible
    pub en_passant: Option<usize>,
    /// Number of halfmoves since last capture or pawn advance
    pub halfmove_clock: u8,
    /// Number of fullmoves since start of game
    pub fullmove_clock: u8,
    /// Bitboards for each piece type and occupancy
    pub pieces: [[u64; 6]; 2],  // [Color as usize][PieceType as usize]
    /// Bitboards of total occupancies for each color
    pub pieces_occ: [u64; 2],
    /// Current evaluation of the position
    pub eval: i32,
    /// Current game phase
    pub game_phase: Option<i32>,
    /// Position history, for checking for repetitions
    pub position_history: HashMap<u64, u8>,
}

// Little Endian Rank Mapping
// Least Significant File
// ind = 8 * rank + file

/// Converts file and rank coordinates to a square index.
///
/// # Arguments
///
/// * `file` - The file (0-7, where 0 is the a-file)
/// * `rank` - The rank (0-7, where 0 is the first rank)
///
/// # Returns
///
/// The square index (0-63)
pub fn coords_to_sq_ind(file: usize, rank: usize) -> usize {
    8 * rank + file
}

/// Converts a square index to file and rank coordinates.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A tuple (file, rank) where file and rank are 0-7
pub fn sq_ind_to_coords(sq_ind: usize) -> (usize, usize) {
    (sq_ind % 8, sq_ind / 8)
}

/// Converts a square index to a bitboard representation.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A 64-bit integer with only the bit at the given square index set
pub fn sq_ind_to_bit(sq_ind: usize) -> u64 {
    1 << sq_ind
}

/// Converts a bitboard with a single bit set to its square index.
///
/// # Arguments
///
/// * `bit` - A 64-bit integer with only one bit set
///
/// # Returns
///
/// The index of the set bit (0-63)
pub fn bit_to_sq_ind(bit: u64) -> usize {
    bit.trailing_zeros() as usize
}

/// Converts a square index to algebraic notation.
///
/// # Arguments
///
/// * `sq_ind` - The square index (0-63)
///
/// # Returns
///
/// A string representing the square in algebraic notation (e.g., "e4")
pub fn sq_ind_to_algebraic(sq_ind: usize) -> String {
    let (file, rank) = sq_ind_to_coords(sq_ind);
    let file = (file + 97) as u8 as char;
    let rank = (rank + 49) as u8 as char;
    format!("{}{}", file, rank)
}

/// Converts algebraic notation to a square index.
///
/// # Arguments
///
/// * `algebraic` - A string representing a square in algebraic notation (e.g., "e4")
///
/// # Returns
///
/// The corresponding square index (0-63)
pub fn algebraic_to_sq_ind(algebraic: &str) -> usize {
    let mut chars = algebraic.chars();
    let file = chars.next().unwrap() as usize - 97;
    let rank = chars.next().unwrap() as usize - 49;
    coords_to_sq_ind(file, rank)
}

/// Converts algebraic notation to a bitboard representation.
///
/// # Arguments
///
/// * `algebraic` - A string representing a square in algebraic notation (e.g., "e4")
///
/// # Returns
///
/// A 64-bit integer with only the bit at the given square set
pub fn algebraic_to_bit(algebraic: &str) -> u64 {
    let sq_ind = algebraic_to_sq_ind(algebraic);
    sq_ind_to_bit(sq_ind)
}

/// Converts a bitboard with a single bit set to algebraic notation.
///
/// # Arguments
///
/// * `bit` - A 64-bit integer with only one bit set
///
/// # Returns
///
/// A string representing the square in algebraic notation (e.g., "e4")
pub fn bit_to_algebraic(bit: u64) -> String {
    let sq_ind = bit_to_sq_ind(bit);
    sq_ind_to_algebraic(sq_ind)
}

/// Flips a square index vertically on the board.
///
/// # Arguments
///
/// * `sq_ind` - The square index to flip (0-63)
///
/// # Returns
///
/// The vertically flipped square index (0-63)
pub fn flip_sq_ind_vertically(sq_ind: usize) -> usize {
    8 * (7 - sq_ind / 8) + sq_ind % 8
}

/// Flips a bitboard vertically.
///
/// # Arguments
///
/// * `bit` - The bitboard to flip
///
/// # Returns
///
/// The vertically flipped bitboard
pub fn flip_vertically(bit: u64) -> u64 {
    ( (bit << 56)                    ) |
        ( (bit << 40) & (0x00ff000000000000) ) |
        ( (bit << 24) & (0x0000ff0000000000) ) |
        ( (bit <<  8) & (0x000000ff00000000) ) |
        ( (bit >>  8) & (0x00000000ff000000) ) |
        ( (bit >> 24) & (0x0000000000ff0000) ) |
        ( (bit >> 40) & (0x000000000000ff00) ) |
        ( (bit >> 56) )
}

impl Bitboard {
    /// Creates a new Bitboard with the initial chess position.
    ///
    /// # Returns
    ///
    /// A new Bitboard struct representing the starting position of a chess game.
    pub fn new() -> Bitboard {
        let mut bitboard = Bitboard {
            game_result: None,
            w_to_move: true,
            w_castle_k: true,
            w_castle_q: true,
            b_castle_k: true,
            b_castle_q: true,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
            pieces: [[0; 6]; 2],  // Initialize all bitboards to 0
            pieces_occ: [0; 2],
            eval: 0,
            game_phase: None,
            position_history: Default::default(),
        };

        // Set up pieces in the starting position
        bitboard.pieces[WHITE][PAWN] = 0x000000000000FF00;
        bitboard.pieces[BLACK][PAWN] = 0x00FF000000000000;
        bitboard.pieces[WHITE][KNIGHT] = 0x0000000000000042;
        bitboard.pieces[BLACK][KNIGHT] = 0x4200000000000000;
        bitboard.pieces[WHITE][BISHOP] = 0x0000000000000024;
        bitboard.pieces[BLACK][BISHOP] = 0x2400000000000000;
        bitboard.pieces[WHITE][ROOK] = 0x0000000000000081;
        bitboard.pieces[BLACK][ROOK] = 0x8100000000000000;
        bitboard.pieces[WHITE][QUEEN] = 0x0000000000000008;
        bitboard.pieces[BLACK][QUEEN] = 0x0800000000000000;
        bitboard.pieces[WHITE][KING] = 0x0000000000000010;
        bitboard.pieces[BLACK][KING] = 0x1000000000000000;
        bitboard.pieces_occ[WHITE] = 0x000000000000FFFF;
        bitboard.pieces_occ[BLACK] = 0xFFFF000000000000;
        bitboard
    }

    /// Creates a new Bitboard from a FEN (Forsyth–Edwards Notation) string.
    ///
    /// # Arguments
    ///
    /// * `fen` - A string slice that holds the FEN representation of a chess position.
    ///
    /// # Returns
    ///
    /// A new Bitboard struct representing the chess position described by the FEN string.
    pub fn new_from_fen(fen: &str) -> Bitboard {
        let parts = fen.split(' ').collect::<Vec<&str>>();
        let mut board = Bitboard::new();
        board.pieces = [[0; 6]; 2];
        board.pieces_occ = [0; 2];
        board.w_castle_k = false;
        board.w_castle_q = false;
        board.b_castle_k = false;
        board.b_castle_q = false;
        let mut rank = 7;
        let mut file = 0;
        for c in parts[0].chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as usize;
            } else {
                let sq_ind = coords_to_sq_ind(file, rank);
                let bit = sq_ind_to_bit(sq_ind);
                match c {
                    'P' => board.pieces[WHITE][PAWN] ^= bit,
                    'p' => board.pieces[BLACK][PAWN] ^= bit,
                    'N' => board.pieces[WHITE][KNIGHT] ^= bit,
                    'n' => board.pieces[BLACK][KNIGHT] ^= bit,
                    'B' => board.pieces[WHITE][BISHOP] ^= bit,
                    'b' => board.pieces[BLACK][BISHOP] ^= bit,
                    'R' => board.pieces[WHITE][ROOK] ^= bit,
                    'r' => board.pieces[BLACK][ROOK] ^= bit,
                    'Q' => board.pieces[WHITE][QUEEN] ^= bit,
                    'q' => board.pieces[BLACK][QUEEN] ^= bit,
                    'K' => board.pieces[WHITE][KING] ^= bit,
                    'k' => board.pieces[BLACK][KING] ^= bit,
                    _ => panic!("Invalid FEN")
                }
                file += 1;
            }
        }
        match parts[1] {
            "w" => board.w_to_move = true,
            "b" => board.w_to_move = false,
            _ => panic!("Invalid FEN")
        }
        match parts[2] {
            "-" => (),
            _ => {
                for c in parts[2].chars() {
                    match c {
                        'K' => board.w_castle_k = true,
                        'Q' => board.w_castle_q = true,
                        'k' => board.b_castle_k = true,
                        'q' => board.b_castle_q = true,
                        _ => panic!("Invalid FEN")
                    }
                }
            }
        }
        match parts[3] {
            "-" => (),
            _ => {
                let sq_ind = algebraic_to_sq_ind(parts[3]);
                board.en_passant = Some(sq_ind);
            }
        }
        match parts[4] {
            "0" => (),
            _ => {
                board.halfmove_clock = parts[4].parse::<u8>().unwrap();
            }
        }
        match parts[5] {
            "1" => (),
            _ => {
                board.fullmove_clock = parts[5].parse::<u8>().unwrap();
            }
        }
        for color in 0..2 {
            board.pieces_occ[color] = board.pieces[color][PAWN];
            for piece in 1..6 {
                board.pieces_occ[color] |= board.pieces[color][piece];
            }
        }
        board
    }

    /// Prints a visual representation of the chess board to the console.
    pub fn print(&self) {
        println!("  +-----------------+");
        for rank in (0..8).rev() {
            print!("{} | ", rank + 1);
            for file in 0..8 {
                let sq_ind = coords_to_sq_ind(file, rank);
                let bit = sq_ind_to_bit(sq_ind);
                if bit & self.pieces[WHITE][PAWN] != 0 {
                    print!("P ");
                } else if bit & self.pieces[BLACK][PAWN] != 0 {
                    print!("p ");
                } else if bit & self.pieces[WHITE][KNIGHT] != 0 {
                    print!("N ");
                } else if bit & self.pieces[BLACK][KNIGHT] != 0 {
                    print!("n ");
                } else if bit & self.pieces[WHITE][BISHOP] != 0 {
                    print!("B ");
                } else if bit & self.pieces[BLACK][BISHOP] != 0 {
                    print!("b ");
                } else if bit & self.pieces[WHITE][ROOK] != 0 {
                    print!("R ");
                } else if bit & self.pieces[BLACK][ROOK] != 0 {
                    print!("r ");
                } else if bit & self.pieces[WHITE][QUEEN] != 0 {
                    print!("Q ");
                } else if bit & self.pieces[BLACK][QUEEN] != 0 {
                    print!("q ");
                } else if bit & self.pieces[WHITE][KING] != 0 {
                    print!("K ");
                } else if bit & self.pieces[BLACK][KING] != 0 {
                    print!("k ");
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }

    /// Gets the piece type at a given square index.
    ///
    /// # Arguments
    ///
    /// * `sq_ind` - The square index to check (0-63)
    ///
    /// # Returns
    ///
    /// An Option containing the piece type if a piece is present, or None if the square is empty.
    pub fn get_piece(&self, sq_ind: usize) -> Option<(usize, usize)> {
        let bit = sq_ind_to_bit(sq_ind);
        if bit & self.pieces[WHITE][PAWN] != 0 {
            Some((WHITE, PAWN))
        } else if bit & self.pieces[BLACK][PAWN] != 0 {
            Some((BLACK, PAWN))
        } else if bit & self.pieces[WHITE][KNIGHT] != 0 {
            Some((WHITE, KNIGHT))
        } else if bit & self.pieces[BLACK][KNIGHT] != 0 {
            Some((BLACK, KNIGHT))
        } else if bit & self.pieces[WHITE][BISHOP] != 0 {
            Some((WHITE, BISHOP))
        } else if bit & self.pieces[BLACK][BISHOP] != 0 {
            Some((BLACK, BISHOP))
        } else if bit & self.pieces[WHITE][ROOK] != 0 {
            Some((WHITE, ROOK))
        } else if bit & self.pieces[BLACK][ROOK] != 0 {
            Some((BLACK, ROOK))
        } else if bit & self.pieces[WHITE][QUEEN] != 0 {
            Some((WHITE, QUEEN))
        } else if bit & self.pieces[BLACK][QUEEN] != 0 {
            Some((BLACK, QUEEN))
        } else if bit & self.pieces[WHITE][KING] != 0 {
            Some((WHITE, KING))
        } else if bit & self.pieces[BLACK][KING] != 0 {
            Some((BLACK, KING))
        } else {
            None
        }
    }

    /// Returns the bitboard for a specific piece type and color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color of the piece (White or Black)
    /// * `piece_type` - The type of the piece (Pawn, Knight, Bishop, Rook, Queen, or King)
    ///
    /// # Returns
    ///
    /// A 64-bit unsigned integer representing the bitboard for the specified piece type and color.
    ///
    /// # Examples
    ///
    /// ```
    /// use kingfisher::bitboard::Bitboard;
    /// use kingfisher::piece_types::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};
    /// let board = Bitboard::new(); // Assume this creates a standard starting position
    /// let white_pawns = board.get_piece_bitboard(WHITE, PAWN);
    /// let black_pawns = board.get_piece_bitboard(BLACK, PAWN);
    /// assert_eq!(white_pawns, 0x000000000000FF00); // All white pawns on their starting squares
    /// assert_eq!(black_pawns, 0x00FF000000000000); // All black pawns on their starting squares
    /// ```
    pub fn get_piece_bitboard(&self, color: usize, piece_type: usize) -> u64 {
        self.pieces[color][piece_type]
    }

    /// Determines whether the current position is legal.
    ///
    /// A position is considered legal if the side to move cannot capture the opponent's king.
    ///
    /// # Arguments
    ///
    /// * `move_gen` - A reference to a MoveGen struct for generating potential moves.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the current position is legal.
    pub fn is_legal(&self, move_gen: &MoveGen) -> bool {
        self.print();
        let king_sq_ind: usize;
        if self.w_to_move {
            king_sq_ind = bit_to_sq_ind(self.pieces[BLACK][KING]);
            if king_sq_ind == 64 {
                println!("No black king");
                self.print();
            }
        } else {
            king_sq_ind = bit_to_sq_ind(self.pieces[WHITE][KING]);
            if king_sq_ind == 64 {
                println!("No white king");
                self.print();
            }
        }
        !self.is_square_attacked(king_sq_ind, self.w_to_move, move_gen)
    }

    /// Determines whether the current position is checkmate or stalemate.
    ///
    /// A position is considered checkmate if the side to move is in check and has no legal moves.
    /// A position is considered stalemate if the side to move is not in check but has no legal moves.
    ///
    /// # Arguments
    ///
    /// * `move_gen` - A reference to a MoveGen struct for generating potential moves.
    ///
    /// # Returns
    ///
    /// A tuple (bool, bool) where:
    /// - The first boolean is true if the position is checkmate, false otherwise.
    /// - The second boolean is true if the position is stalemate, false otherwise.
    pub fn is_checkmate_or_stalemate(&self, move_gen: &MoveGen) -> (bool, bool) {
        // Generate all pseudo-legal moves
        let (captures, moves) = move_gen.gen_pseudo_legal_moves(self);

        // Check if any of the captures are legal
        if !captures.is_empty() {
            for c in captures {
                let new_board = self.make_move(c);
                if new_board.is_legal(move_gen) {
                    return (false, false);
                }
            }
        }
        
        // Check if any of the non-capture moves are legal
        if !moves.is_empty() {
            for m in moves {
                let new_board = self.make_move(m);
                if new_board.is_legal(move_gen) {
                    return (false, false);
                }
            }
        }

        // If we get here, there are no legal moves.
        // Check if the current position is in check
        let is_check = self.is_check(move_gen);
        if is_check {
            (true, false)  // Checkmate
        } else {
            (false, true)  // Stalemate
        }
    }

    /// Checks if the king of the side to move is in check.
    ///
    /// # Arguments
    ///
    /// * `move_gen` - A reference to a MoveGen struct for generating potential moves.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the king of the side to move is in check.
    pub fn is_check(&self, move_gen: &MoveGen) -> bool {
        let king_sq_ind: usize;
        if self.w_to_move {
            king_sq_ind = bit_to_sq_ind(self.pieces[WHITE][KING]);
        } else {
            king_sq_ind = bit_to_sq_ind(self.pieces[BLACK][KING]);
        }
        self.is_square_attacked(king_sq_ind, !self.w_to_move, move_gen)
    }

    /// Checks if a square is attacked by a given side.
    ///
    /// # Arguments
    ///
    /// * `sq_ind`- The square index (0-63) to check.
    /// * `by_white` - If true, check if the square is attacked by white; if false, check if it's attacked by black.
    /// * `move_gen` - A reference to a MoveGen struct for generating potential moves.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the square is attacked by the specified side.
    pub fn is_square_attacked(&self, sq_ind: usize, by_white: bool, move_gen: &MoveGen) -> bool {
        // Find out if the square is attacked by a given side (white if by_white is true, black if by_white is false).
        if by_white {
            // Can the king reach an enemy bishop or queen by a bishop move?
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[WHITE][BISHOP] | self.pieces[WHITE][QUEEN])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[WHITE][ROOK] | self.pieces[WHITE][QUEEN])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[WHITE][KNIGHT]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.bp_capture_bitboard[sq_ind] & self.pieces[WHITE][PAWN]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[WHITE][KING]) != 0 {
                return true;
            }
            false
        } else {
            // Can the king reach an enemy bishop or queen by a bishop move?
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[BLACK][BISHOP] | self.pieces[BLACK][QUEEN])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[BLACK][ROOK] | self.pieces[BLACK][QUEEN])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[BLACK][KNIGHT]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.wp_capture_bitboard[sq_ind] & self.pieces[BLACK][PAWN]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[BLACK][KING]) != 0 {
                return true;
            }
            false
        }
    }

    /// Checks if the current position has occurred three times, resulting in a draw by repetition.
    ///
    /// This method considers a position to be repeated if:
    /// - The piece positions are identical
    /// - The side to move is the same
    /// - The castling rights are the same
    /// - The en passant possibilities are the same
    ///
    /// # Returns
    ///
    /// `true` if the current position has occurred three times, `false` otherwise.
    ///
    /// # Note
    ///
    /// This method relies on the Zobrist hash of the position, which includes all
    /// relevant aspects of the chess position.
    pub fn is_draw_by_repetition(&self) -> bool {
        // Compute hash of current position
        let hash = self.compute_zobrist_hash();

        // Check if there are 3 or more repetitions of the same hash
        *self.position_history.get(&hash).unwrap() >= 3
    }

}