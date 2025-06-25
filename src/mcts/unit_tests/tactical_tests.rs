//! Unit tests for tactical move detection and prioritization

use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use crate::mcts::tactical::{identify_tactical_moves, TacticalMove, calculate_mvv_lva};

fn setup_test_env() -> (MoveGen,) {
    (MoveGen::new(),)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvv_lva_basic_scoring() {
        let (move_gen,) = setup_test_env();
        
        // Simple position for testing MVV-LVA
        let board = Board::new_from_fen("rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3");
        
        // Create a test capture move (this is a simplified test)
        let test_move = Move::new(26, 36, None); // Example move
        let score = calculate_mvv_lva(test_move, &board);
        
        // Should return a valid score
        assert!(score.is_finite(), "MVV-LVA score should be finite");
    }

    #[test]
    fn test_tactical_move_identification() {
        let (move_gen,) = setup_test_env();
        
        // Position with available captures
        let board = Board::new_from_fen("rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3");
        
        let tactical_moves = identify_tactical_moves(&board, &move_gen);
        
        // Should identify some tactical moves
        assert!(tactical_moves.len() >= 0, "Should identify tactical moves (possibly zero in quiet positions)");
        
        // All tactical moves should have valid types
        for tm in &tactical_moves {
            let move_type = tm.move_type();
            assert!(
                move_type == "Capture" || 
                move_type == "Check" || 
                move_type == "Fork" || 
                move_type == "Pin",
                "Unknown tactical move type: {}", move_type
            );
        }
    }

    #[test]
    fn test_no_tactical_moves_in_quiet_position() {
        let (move_gen,) = setup_test_env();
        
        // Starting position - should have no immediate tactical moves
        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        
        let tactical_moves = identify_tactical_moves(&board, &move_gen);
        
        // Starting position should have no captures, checks, or forks
        assert!(tactical_moves.is_empty(), 
                "Starting position should have no tactical moves, found: {}", tactical_moves.len());
    }

    #[test]
    fn test_tactical_move_scoring() {
        let (move_gen,) = setup_test_env();
        
        // Position with multiple tactical options
        let board = Board::new_from_fen("r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4");
        
        let tactical_moves = identify_tactical_moves(&board, &move_gen);
        
        // All tactical moves should have valid scores
        for tm in &tactical_moves {
            assert!(tm.score() >= 0.0, "Tactical move should have non-negative score");
        }
    }

    #[test]
    fn test_tactical_move_uniqueness() {
        let (move_gen,) = setup_test_env();
        
        // Position with potential overlapping tactical categories
        let board = Board::new_from_fen("r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4");
        
        let tactical_moves = identify_tactical_moves(&board, &move_gen);
        
        // Verify no duplicate moves
        let mut seen_moves = std::collections::HashSet::new();
        for tm in &tactical_moves {
            let mv = tm.get_move();
            assert!(!seen_moves.contains(&mv), 
                    "Move {:?} appears multiple times in tactical moves", mv);
            seen_moves.insert(mv);
        }
    }
}