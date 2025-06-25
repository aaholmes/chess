//! Tactical Move Detection and Prioritization
//!
//! This module implements tactical move identification and prioritization for the
//! tactical-first MCTS approach. It identifies captures, checks, and forks, then
//! prioritizes them using classical heuristics like MVV-LVA.

use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use crate::piece_types::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING};
use crate::search::see;
use std::collections::HashSet;

/// Represents a tactical move with its associated priority score
#[derive(Debug, Clone)]
pub enum TacticalMove {
    /// Capture move with MVV-LVA score
    Capture(Move, f64),
    /// Checking move with priority score
    Check(Move, f64),
    /// Fork move with value score
    Fork(Move, f64),
    /// Pin or skewer move
    Pin(Move, f64),
}

impl TacticalMove {
    /// Get the underlying move
    pub fn get_move(&self) -> Move {
        match self {
            TacticalMove::Capture(mv, _) => *mv,
            TacticalMove::Check(mv, _) => *mv,
            TacticalMove::Fork(mv, _) => *mv,
            TacticalMove::Pin(mv, _) => *mv,
        }
    }
    
    /// Get the priority score for this tactical move
    pub fn score(&self) -> f64 {
        match self {
            TacticalMove::Capture(_, score) => *score,
            TacticalMove::Check(_, score) => *score,
            TacticalMove::Fork(_, score) => *score,
            TacticalMove::Pin(_, score) => *score,
        }
    }
    
    /// Get the tactical move type as a string
    pub fn move_type(&self) -> &'static str {
        match self {
            TacticalMove::Capture(_, _) => "Capture",
            TacticalMove::Check(_, _) => "Check",
            TacticalMove::Fork(_, _) => "Fork",
            TacticalMove::Pin(_, _) => "Pin",
        }
    }
}

/// Identify all tactical moves from a given position
pub fn identify_tactical_moves(board: &Board, move_gen: &MoveGen) -> Vec<TacticalMove> {
    let mut tactical_moves = Vec::new();
    let (captures, non_captures) = move_gen.gen_pseudo_legal_moves(board);
    
    // Track moves we've already identified to avoid duplicates
    let mut identified_moves = HashSet::new();
    
    // 1. Analyze captures with MVV-LVA scoring
    for mv in &captures {
        if board.apply_move_to_board(*mv).is_legal(move_gen) {
            if !is_losing_capture(*mv, board) {
                let mvv_lva_score = calculate_mvv_lva(*mv, board);
                tactical_moves.push(TacticalMove::Capture(*mv, mvv_lva_score));
                identified_moves.insert(*mv);
            }
        }
    }
    
    // 2. Analyze checking moves (including non-captures)
    for mv in captures.iter().chain(non_captures.iter()) {
        if !identified_moves.contains(mv) {
            let new_board = board.apply_move_to_board(*mv);
            if new_board.is_legal(move_gen) && new_board.is_check(move_gen) {
                let check_score = calculate_check_priority(*mv, board);
                tactical_moves.push(TacticalMove::Check(*mv, check_score));
                identified_moves.insert(*mv);
            }
        }
    }
    
    // 3. Analyze fork moves (knight and pawn forks initially)
    for mv in captures.iter().chain(non_captures.iter()) {
        if !identified_moves.contains(mv) {
            let new_board = board.apply_move_to_board(*mv);
            if new_board.is_legal(move_gen) {
                if let Some(fork_score) = detect_fork_move(*mv, board, &new_board) {
                    tactical_moves.push(TacticalMove::Fork(*mv, fork_score));
                    identified_moves.insert(*mv);
                }
            }
        }
    }
    
    // Sort by priority score (highest first)
    tactical_moves.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap_or(std::cmp::Ordering::Equal));
    
    tactical_moves
}

/// Calculate MVV-LVA (Most Valuable Victim - Least Valuable Attacker) score
pub fn calculate_mvv_lva(mv: Move, board: &Board) -> f64 {
    let victim_value = get_piece_value_at_square(board, mv.to);
    let attacker_value = get_piece_value_at_square(board, mv.from);
    
    // MVV-LVA: prioritize valuable victims, deprioritize valuable attackers
    // Use 10x multiplier for victim to ensure victim value dominates
    (victim_value * 10.0) - attacker_value
}

/// Check if a capture is losing using Static Exchange Evaluation
fn is_losing_capture(mv: Move, board: &Board) -> bool {
    // Use SEE to determine if the capture loses material
    // For now, just accept all captures - proper SEE integration needs move_gen context
    // TODO: Implement proper SEE evaluation with correct parameters
    false
}

/// Get the value of a piece at a given square
fn get_piece_value_at_square(board: &Board, square: usize) -> f64 {
    // Extract piece information from board
    // This is a simplified implementation - you may need to adjust based on your Board structure
    for color in 0..2 {
        for piece_type in 0..6 {
            if board.pieces[color][piece_type] & (1u64 << square) != 0 {
                return get_piece_type_value(piece_type);
            }
        }
    }
    0.0 // Empty square
}

/// Get the standard value of a piece type
fn get_piece_type_value(piece_type: usize) -> f64 {
    match piece_type {
        PAWN => 1.0,
        KNIGHT => 3.0,
        BISHOP => 3.0,
        ROOK => 5.0,
        QUEEN => 9.0,
        KING => 0.0, // King "captures" are handled separately (illegal anyway)
        _ => 0.0,
    }
}

/// Calculate priority score for checking moves
fn calculate_check_priority(mv: Move, board: &Board) -> f64 {
    let piece_value = get_piece_value_at_square(board, mv.from);
    let centrality_bonus = calculate_centrality_bonus(mv.to);
    
    // Base score for checks, with bonus for piece value and centrality
    let base_check_score = 2.0; // Checks are generally valuable
    base_check_score + piece_value * 0.1 + centrality_bonus
}

/// Calculate bonus for central squares (simple implementation)
fn calculate_centrality_bonus(square: usize) -> f64 {
    let file = square % 8;
    let rank = square / 8;
    
    // Central squares (d4, d5, e4, e5) get highest bonus
    let distance_from_center = ((file as i32 - 3).abs() + (rank as i32 - 3).abs()) as f64;
    (8.0 - distance_from_center) * 0.1
}

/// Detect fork moves and calculate their value
fn detect_fork_move(mv: Move, board: &Board, new_board: &Board) -> Option<f64> {
    let piece_type = get_piece_type_at_square(board, mv.from)?;
    
    match piece_type {
        KNIGHT => detect_knight_fork(mv, new_board),
        PAWN => detect_pawn_fork(mv, board, new_board),
        _ => None, // Other piece forks are more complex to detect accurately
    }
}

/// Get the piece type at a given square
fn get_piece_type_at_square(board: &Board, square: usize) -> Option<usize> {
    for color in 0..2 {
        for piece_type in 0..6 {
            if board.pieces[color][piece_type] & (1u64 << square) != 0 {
                return Some(piece_type);
            }
        }
    }
    None
}

/// Detect knight forks
fn detect_knight_fork(mv: Move, new_board: &Board) -> Option<f64> {
    let knight_attacks = get_knight_attacks(mv.to);
    let opponent_color = if new_board.w_to_move { 1 } else { 0 }; // Opponent's color after the move
    
    let mut valuable_targets = Vec::new();
    
    for attack_square in knight_attacks {
        if let Some(target_value) = get_piece_value_at_square_for_color(new_board, attack_square, opponent_color) {
            if target_value >= 3.0 { // Rook, Queen, or King
                valuable_targets.push(target_value);
            }
        }
    }
    
    if valuable_targets.len() >= 2 {
        // Fork detected - score based on target values
        let total_value: f64 = valuable_targets.iter().sum();
        Some(5.0 + total_value) // Base fork value + target values
    } else {
        None
    }
}

/// Get knight attack squares from a given position
fn get_knight_attacks(square: usize) -> Vec<usize> {
    let mut attacks = Vec::new();
    let file = (square % 8) as i8;
    let rank = (square / 8) as i8;
    
    let knight_offsets = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2), (1, 2), (2, -1), (2, 1),
    ];
    
    for (df, dr) in knight_offsets.iter() {
        let new_file = file + df;
        let new_rank = rank + dr;
        
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            attacks.push((new_rank * 8 + new_file) as usize);
        }
    }
    
    attacks
}

/// Get piece value at square for specific color
fn get_piece_value_at_square_for_color(board: &Board, square: usize, color: usize) -> Option<f64> {
    for piece_type in 0..6 {
        if board.pieces[color][piece_type] & (1u64 << square) != 0 {
            return Some(get_piece_type_value(piece_type));
        }
    }
    None
}

/// Detect pawn forks
fn detect_pawn_fork(mv: Move, board: &Board, new_board: &Board) -> Option<f64> {
    let pawn_attacks = get_pawn_attacks(mv.to, board.w_to_move);
    let opponent_color = if new_board.w_to_move { 1 } else { 0 };
    
    let mut targets = 0;
    let mut total_value = 0.0;
    
    for attack_square in pawn_attacks {
        if let Some(target_value) = get_piece_value_at_square_for_color(new_board, attack_square, opponent_color) {
            if target_value >= 3.0 { // Knight, Bishop, Rook, or Queen
                targets += 1;
                total_value += target_value;
            }
        }
    }
    
    if targets >= 2 {
        Some(3.0 + total_value) // Pawn forks are generally less valuable than knight forks
    } else {
        None
    }
}

/// Get pawn attack squares
fn get_pawn_attacks(square: usize, is_white: bool) -> Vec<usize> {
    let mut attacks = Vec::new();
    let file = (square % 8) as i8;
    let rank = (square / 8) as i8;
    
    let direction = if is_white { 1 } else { -1 };
    
    // Left diagonal
    if file > 0 {
        let new_rank = rank + direction;
        if new_rank >= 0 && new_rank < 8 {
            attacks.push((new_rank * 8 + file - 1) as usize);
        }
    }
    
    // Right diagonal
    if file < 7 {
        let new_rank = rank + direction;
        if new_rank >= 0 && new_rank < 8 {
            attacks.push((new_rank * 8 + file + 1) as usize);
        }
    }
    
    attacks
}

/// Filter tactical moves to remove obviously bad ones
pub fn filter_tactical_moves(tactical_moves: Vec<TacticalMove>, board: &Board) -> Vec<TacticalMove> {
    tactical_moves
        .into_iter()
        .filter(|tactical_move| {
            let mv = tactical_move.get_move();
            
            // Basic legality check (should already be done, but double-check)
            let new_board = board.apply_move_to_board(mv);
            if !new_board.is_legal(&MoveGen::new()) {
                return false;
            }
            
            // For captures, ensure they're not obviously losing
            if let TacticalMove::Capture(_, _) = tactical_move {
                return !is_losing_capture(mv, board);
            }
            
            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    
    #[test]
    fn test_tactical_move_identification() {
        let board = Board::new(); // Starting position
        let move_gen = MoveGen::new();
        
        let tactical_moves = identify_tactical_moves(&board, &move_gen);
        
        // Starting position should have no tactical moves
        assert!(tactical_moves.is_empty());
    }
    
    #[test]
    fn test_mvv_lva_calculation() {
        let board = Board::new_from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        
        // Test capturing pawn with pawn (if such move exists)
        let mv = Move::new(28, 36, None); // e4 takes e5 (example)
        let score = calculate_mvv_lva(mv, &board);
        
        // Pawn takes pawn should give score: (1.0 * 10) - 1.0 = 9.0
        assert!((score - 9.0).abs() < 0.1);
    }
    
    #[test]
    fn test_knight_attack_generation() {
        // Test knight on e4 (square 28)
        let attacks = get_knight_attacks(28);
        
        // Knight on e4 should attack 8 squares (if all are on board)
        assert_eq!(attacks.len(), 8);
        
        // Check some specific squares
        assert!(attacks.contains(&19)); // d6
        assert!(attacks.contains(&21)); // f6
        assert!(attacks.contains(&11)); // d2
    }
}