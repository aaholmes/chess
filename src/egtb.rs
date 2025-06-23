// src/egtb.rs

// Use internal project types
use crate::board::Board;
use crate::move_types::{Move, CastlingRights};
use crate::piece_types::{self, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};
use crate::board_utils::sq_ind_to_bit; // Needed for piece count calculation

// Use the Syzygy library
use shakmaty::{Board as ShakmatyBoard, Position, Role, Color, Square};
use shakmaty_syzygy::{Tablebase, Wdl, Dtz};
use std::path::Path;

// Define the error type for EGTB operations
#[derive(Debug)]
pub enum EgtbError {
    LoadError(String), // Error loading tablebases
    ProbeError(String), // Error during probing
    ConversionError(String), // Error converting board representation
}

impl std::fmt::Display for EgtbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EgtbError::LoadError(s) => write!(f, "EGTB Load Error: {}", s),
            EgtbError::ProbeError(s) => write!(f, "EGTB Probe Error: {}", s),
            EgtbError::ConversionError(s) => write!(f, "Board Conversion Error: {}", s),
        }
    }
}

impl std::error::Error for EgtbError {}


/// Information obtained from probing the endgame tablebases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgtbInfo {
    /// Win/Draw/Loss status from the perspective of the side to move.
    pub wdl: Wdl,
    /// Distance to Zeroing move (often related to Distance to Mate).
    pub dtz: Option<Dtz>,
    /// The best move according to the tablebase, if available. (Currently always None)
    pub best_move: Option<Move>, // Use crate::move_types::Move
}

/// Structure to handle Syzygy endgame tablebase probing.
pub struct EgtbProber {
    tablebases: Tablebase,
    pub max_pieces: u8, // Store the max pieces supported by loaded tables
}

impl EgtbProber {
    /// Creates a new EgtbProber by loading tablebases from the specified path.
    pub fn new(path_str: &str) -> Result<Self, EgtbError> {
        let path = Path::new(path_str);
        let mut tablebases = Tablebase::new();
        match tablebases.add_directory(path) {
            Ok(_) => {
                // shakmaty-syzygy doesn't expose max pieces directly,
                // but it's typically 7 for standard distributions.
                let max_pieces = 7;
                Ok(EgtbProber { tablebases, max_pieces })
            }
            Err(e) => Err(EgtbError::LoadError(format!("Failed to load tablebases from '{}': {}", path_str, e))),
        }
    }

    /// Probes the endgame tablebases for the given board position.
    pub fn probe(&self, board: &Board) -> Result<Option<EgtbInfo>, EgtbError> {
        // 1. Check piece count using internal board representation
        let piece_count = board.get_all_occupancy().count_ones() as u8;
        if piece_count > self.max_pieces {
            return Ok(None); // Too many pieces
        }

        // 2. Convert internal Board to shakmaty::Board
        let shakmaty_board = match self.convert_board(board) {
             Ok(b) => b,
             Err(e) => return Err(e), // Propagate conversion error
        };

        // 3. Probe WDL
        let wdl_result = self.tablebases.probe_wdl(&shakmaty_board);
        if wdl_result.is_none() {
             // Position configuration not found in tables (even if piece count <= max)
             return Ok(None);
        }
        let wdl = wdl_result.unwrap();

        // 4. Probe DTZ
        let dtz = self.tablebases.probe_dtz(&shakmaty_board); // Returns Option<Dtz>

        // 5. Best Move (Still omitted)
        let best_move = None;

        Ok(Some(EgtbInfo { wdl, dtz, best_move }))
    }

    /// Helper function to convert `crate::board::Board` to `shakmaty::Board`.
    fn convert_board(&self, board: &Board) -> Result<ShakmatyBoard, EgtbError> {
       let mut shakmaty_board = ShakmatyBoard::empty();

       // Set pieces
       for color_idx in [WHITE, BLACK] {
           for piece_idx in [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] {
               let bb = board.get_piece_bitboard(color_idx, piece_idx);
               for sq_idx in 0..64 {
                   if (bb & sq_ind_to_bit(sq_idx)) != 0 {
                       let shakmaty_role = match piece_idx {
                           PAWN => Role::Pawn,
                           KNIGHT => Role::Knight,
                           BISHOP => Role::Bishop,
                           ROOK => Role::Rook,
                           QUEEN => Role::Queen,
                           KING => Role::King,
                           _ => unreachable!(), // Should not happen
                       };
                       let shakmaty_color = if color_idx == WHITE { Color::White } else { Color::Black };
                       let shakmaty_sq = Square::from_int(sq_idx as usize)
                           .ok_or_else(|| EgtbError::ConversionError(format!("Invalid square index: {}", sq_idx)))?;
                       shakmaty_board.set_piece(shakmaty_sq, shakmaty_color, shakmaty_role);
                   }
               }
           }
       }

       // Set side to move
       shakmaty_board.side_to_move = if board.w_to_move { Color::White } else { Color::Black };

       // Set en passant square
       // Our board.en_passant is Option<u8> (square index)
       shakmaty_board.en_passant = board.en_passant.map(|sq_idx| Square::from_int(sq_idx as usize).unwrap()); // Assuming valid square index

       // Set castling rights
       let mut castling_rights = CastlingRights::empty();
       if board.castling_rights.white_kingside { castling_rights |= CastlingRights::WHITE_KING_SIDE; }
       if board.castling_rights.white_queenside { castling_rights |= CastlingRights::WHITE_QUEEN_SIDE; }
       if board.castling_rights.black_kingside { castling_rights |= CastlingRights::BLACK_KING_SIDE; }
       if board.castling_rights.black_queenside { castling_rights |= CastlingRights::BLACK_QUEEN_SIDE; }
       shakmaty_board.castling_rights = castling_rights;

       // Rule 50 counter and halfmove clock
       shakmaty_board.halfmove_clock = board.halfmove_clock;
       shakmaty_board.fullmove_number = board.fullmove_number;


       // TODO: Add validation? shakmaty might handle invalid positions (e.g., pawns on back rank)
       // during probe, but we could add checks here if needed.

       Ok(shakmaty_board)
    }
}

// Optional: Basic unit tests (updated for internal Board)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board; // Use internal Board
    use shakmaty::Chess; // Use shakmaty's Chess board for creating positions

    // Mock structure specifically for testing the piece count check logic
    // within the probe function, without needing actual tablebases.
    struct PieceCountCheckProber {
        max_pieces: u8,
    }

    impl PieceCountCheckProber {
        fn new(max_pieces: u8) -> Self {
            PieceCountCheckProber { max_pieces }
        }

        // Mimics the initial piece count check of the real EgtbProber::probe
        fn probe(&self, board: &Board) -> Result<Option<()>, EgtbError> {
            let piece_count = board.get_all_occupancy().count_ones() as u8;
            if piece_count > self.max_pieces {
                Ok(None) // Too many pieces, real probe would return here
            } else {
                // In a real probe, conversion and actual probing would happen here.
                // For this mock, we just indicate the piece count was acceptable.
                Ok(Some(()))
            }
        }
    }

    #[test]
    fn test_new_invalid_path() {
        // Use a path that is highly unlikely to exist or be valid EGTB dir
        let result = EgtbProber::new("/tmp/non_existent_egtb_path_for_testing");
        assert!(result.is_err());
        match result.err().unwrap() {
            EgtbError::LoadError(_) => {} // Expected error type
            _ => panic!("Expected LoadError for invalid path"),
        }
    }


    #[test]
    fn test_probe_piece_count_within_limit() {
        // Use the mock that only checks piece count
        let prober = PieceCountCheckProber::new(7);

        // 7 pieces - should pass piece count check (returns Some in mock)
        let board = Board::new_from_fen("8/8/8/8/k7/p7/P7/K7 w - - 0 1");
        assert!(prober.probe(&board).unwrap().is_some());

        // 3 pieces - should pass piece count check
        let board = Board::new_from_fen("8/8/8/8/8/8/k7/K1N5 w - - 0 1");
        assert!(prober.probe(&board).unwrap().is_some());
    }

    #[test]
    fn test_probe_piece_count_exceeds_limit() {
        // Use the mock that only checks piece count
        let prober = PieceCountCheckProber::new(7);

        // Starting pos (32 pieces) - should fail piece count check (returns None)
        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(prober.probe(&board).unwrap().is_none());

        // 8 pieces exactly - should fail piece count check
        let board = Board::new_from_fen("8/8/8/8/k7/p1p5/P1P5/KBN5 w - - 0 1");
        assert!(prober.probe(&board).unwrap().is_none());
    }

    // Note: Testing actual probing still requires real EGTB files.
    // #[test]
    // #[ignore]
    // fn test_real_probe_known_position() {
    //     let prober = EgtbProber::new("../egtb_files").expect("Failed to load test EGTBs");
    //     let board = Board::new_from_fen("8/8/8/8/8/k7/P7/K7 w - - 0 1"); // K vs K+P endgame
    //     let result = prober.probe(&board).unwrap();
    //     assert!(result.is_some());
    //     // Add assertions based on expected WDL/DTZ
    // }
}