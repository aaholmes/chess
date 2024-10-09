//! This module defines the Bitboard structure and associated functions for chess board representation.

use crate::gen_moves::MoveGen;

/// Piece labels for indexing the bitboard vector
pub const WP: usize = 0;  /// White Pawn
pub const BP: usize = 1;  /// Black Pawn
pub const WN: usize = 2;  /// White Knight
pub const BN: usize = 3;  /// Black Knight
pub const WB: usize = 4;  /// White Bishop
pub const BB: usize = 5;  /// Black Bishop
pub const WR: usize = 6;  /// White Rook
pub const BR: usize = 7;  /// Black Rook
pub const WQ: usize = 8;  /// White Queen
pub const BQ: usize = 9;  /// Black Queen
pub const WK: usize = 10; /// White King
pub const BK: usize = 11; /// Black King
pub const WOCC: usize = 12; /// White occupied squares
pub const BOCC: usize = 13; /// Black occupied squares
pub const OCC: usize = 14;  /// All occupied squares

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
    /// Vector of bitboards for each piece type and occupancy
    pub pieces: Vec<u64>,
    /// Current evaluation of the position
    pub eval: i32,
    /// Current game phase
    pub game_phase: Option<i32>
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
        Bitboard {
            game_result: None,
            w_to_move: true,
            w_castle_k: true,
            w_castle_q: true,
            b_castle_k: true,
            b_castle_q: true,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
            pieces: [
                0x000000000000FF00,
                0x00FF000000000000,
                0x0000000000000042,
                0x4200000000000000,
                0x0000000000000024,
                0x2400000000000000,
                0x0000000000000081,
                0x8100000000000000,
                0x0000000000000008,
                0x0800000000000000,
                0x0000000000000010,
                0x1000000000000000,
                0x000000000000FFFF,
                0xFFFF000000000000,
                0xFFFF00000000FFFF
            ].try_into().unwrap(),
            eval: 0,
            game_phase: None
        }
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
        board.pieces = [0; 15].try_into().unwrap();
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
                    'P' => board.pieces[WP] ^= bit,
                    'p' => board.pieces[BP] ^= bit,
                    'N' => board.pieces[WN] ^= bit,
                    'n' => board.pieces[BN] ^= bit,
                    'B' => board.pieces[WB] ^= bit,
                    'b' => board.pieces[BB] ^= bit,
                    'R' => board.pieces[WR] ^= bit,
                    'r' => board.pieces[BR] ^= bit,
                    'Q' => board.pieces[WQ] ^= bit,
                    'q' => board.pieces[BQ] ^= bit,
                    'K' => board.pieces[WK] ^= bit,
                    'k' => board.pieces[BK] ^= bit,
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
        board.pieces[WOCC] = board.pieces[WP] | board.pieces[WN] | board.pieces[WB] | board.pieces[WR] | board.pieces[WQ] | board.pieces[WK];
        board.pieces[BOCC] = board.pieces[BP] | board.pieces[BN] | board.pieces[BB] | board.pieces[BR] | board.pieces[BQ] | board.pieces[BK];
        board.pieces[OCC] = board.pieces[WOCC] | board.pieces[BOCC];
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
                if bit & self.pieces[WK] != 0 {
                    print!("K ");
                } else if bit & self.pieces[WQ] != 0 {
                    print!("Q ");
                } else if bit & self.pieces[WR] != 0 {
                    print!("R ");
                } else if bit & self.pieces[WB] != 0 {
                    print!("B ");
                } else if bit & self.pieces[WN] != 0 {
                    print!("N ");
                } else if bit & self.pieces[WP] != 0 {
                    print!("P ");
                } else if bit & self.pieces[BK] != 0 {
                    print!("k ");
                } else if bit & self.pieces[BQ] != 0 {
                    print!("q ");
                } else if bit & self.pieces[BR] != 0 {
                    print!("r ");
                } else if bit & self.pieces[BB] != 0 {
                    print!("b ");
                } else if bit & self.pieces[BN] != 0 {
                    print!("n ");
                } else if bit & self.pieces[BP] != 0 {
                    print!("p ");
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }

    /// Flips the board vertically, returning a new board.
    ///
    /// This method creates a new Bitboard with the position flipped vertically,
    /// effectively switching the perspectives of White and Black.
    ///
    /// # Returns
    ///
    /// A new Bitboard struct representing the vertically flipped position.
    pub fn flip_vertically(&self) -> Bitboard {
        Bitboard {
            game_result: self.game_result,
            w_to_move: !self.w_to_move,
            w_castle_k: self.b_castle_k,
            w_castle_q: self.b_castle_q,
            b_castle_k: self.w_castle_k,
            b_castle_q: self.w_castle_q,
            en_passant: { if self.en_passant.is_none() { None } else { Some(flip_sq_ind_vertically(self.en_passant.unwrap())) } },
            halfmove_clock: self.halfmove_clock,
            fullmove_clock: self.fullmove_clock,
            pieces: self.pieces.iter().map(|&x| flip_vertically(x)).collect(),
            eval: -self.eval,
            game_phase: self.game_phase
        }
    }

    /// Gets the piece type at a given square index.
    ///
    /// # Arguments
    ///
    /// * `sq_ind` - The square index to check (0-63)
    ///
    /// # Returns
    ///
    /// An Option containing the piece type (0-11) if a piece is present, or None if the square is empty.
    pub fn get_piece(&self, sq_ind: usize) -> Option<usize> {
        let bit = sq_ind_to_bit(sq_ind);
        (0..12).find(|&i| bit & self.pieces[i] != 0)
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
        let king_sq_ind: usize;
        if self.w_to_move {
            king_sq_ind = bit_to_sq_ind(self.pieces[BK]);
            if king_sq_ind == 64 {
                println!("No black king");
                self.print();
            }
        } else {
            king_sq_ind = bit_to_sq_ind(self.pieces[WK]);
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
            king_sq_ind = bit_to_sq_ind(self.pieces[WK]);
        } else {
            king_sq_ind = bit_to_sq_ind(self.pieces[BK]);
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
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[WB] | self.pieces[WQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[WR] | self.pieces[WQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[WN]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.bp_capture_bitboard[sq_ind] & self.pieces[WP]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[WK]) != 0 {
                return true;
            }
            false
        } else {
            // Can the king reach an enemy bishop or queen by a bishop move?
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[BB] | self.pieces[BQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[BR] | self.pieces[BQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[BN]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.wp_capture_bitboard[sq_ind] & self.pieces[BP]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[BK]) != 0 {
                return true;
            }
            false
        }
    }

}
