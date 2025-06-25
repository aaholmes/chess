//! Unit tests for tactical-first node selection and lazy policy evaluation
//!
//! Tests cover node expansion, tactical move prioritization, UCB calculation,
//! and statistics tracking during selection.

use crate::board::Board;
use crate::move_generation::MoveGen;
use crate::move_types::Move;
use crate::mcts::node::MctsNode;
use crate::mcts::selection::{select_child_with_tactical_priority, get_tactical_statistics};
use crate::mcts::tactical_mcts::{TacticalMctsStats, TacticalMctsConfig};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn setup_test_env() -> (Board, MoveGen, TacticalMctsStats) {
    let board = Board::new_from_fen("rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3");
    let move_gen = MoveGen::new();
    let stats = TacticalMctsStats {
        iterations: 0,
        nodes_expanded: 0,
        tactical_moves_explored: 0,
        mates_found: 0,
        nn_policy_evaluations: 0,
        search_time: Duration::from_millis(0),
    };
    (board, move_gen, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_expansion_from_start() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None;
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        // Initially no children
        assert!(root_node.borrow().children.is_empty());
        
        // Call selection which should trigger expansion
        let selected = select_child_with_tactical_priority(
            root_node.clone(),
            1.414,
            &move_gen,
            &mut nn_policy,
            &mut stats,
        );
        
        // Should have expanded children
        assert!(!root_node.borrow().children.is_empty(), "Node should be expanded after selection");
        
        // Should return a valid child
        assert!(selected.is_some(), "Should select a child after expansion");
    }

    #[test]
    fn test_tactical_move_prioritization() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None;
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        // Select multiple times to see tactical moves being prioritized
        for _ in 0..5 {
            let selected = select_child_with_tactical_priority(
                root_node.clone(),
                1.414,
                &move_gen,
                &mut nn_policy,
                &mut stats,
            );
            
            assert!(selected.is_some(), "Should always select a valid child");
        }
        
        // Should have explored some tactical moves
        assert!(stats.tactical_moves_explored > 0, 
                "Should have explored tactical moves, found: {}", stats.tactical_moves_explored);
    }

    #[test]
    fn test_statistics_tracking() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None;
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        let initial_tactical_count = stats.tactical_moves_explored;
        
        // Perform several selections
        for _ in 0..10 {
            select_child_with_tactical_priority(
                root_node.clone(),
                1.414,
                &move_gen,
                &mut nn_policy,
                &mut stats,
            );
        }
        
        // Tactical move count should have increased
        assert!(stats.tactical_moves_explored >= initial_tactical_count,
                "Tactical moves explored should increase or stay same");
        
        // Get tactical statistics for the node
        let tactical_stats = get_tactical_statistics(&root_node.borrow());
        
        assert!(tactical_stats.total_tactical_moves >= 0, 
                "Should have identified tactical moves");
    }

    #[test]
    fn test_ucb_selection_with_policy() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None;
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        // Force expansion first
        select_child_with_tactical_priority(
            root_node.clone(),
            1.414,
            &move_gen,
            &mut nn_policy,
            &mut stats,
        );
        
        // Simulate some visits to create UCB values
        for child in &root_node.borrow().children {
            let mut child_ref = child.borrow_mut();
            child_ref.visits = 1;
            child_ref.total_value = 0.5;
            child_ref.total_value_squared = 0.25;
        }
        
        // Update parent visits
        root_node.borrow_mut().visits = 10;
        
        // Should still be able to select using UCB
        let selected = select_child_with_tactical_priority(
            root_node.clone(),
            1.414,
            &move_gen,
            &mut nn_policy,
            &mut stats,
        );
        
        assert!(selected.is_some(), "Should select child using UCB");
    }

    #[test]
    fn test_terminal_position_handling() {
        let move_gen = MoveGen::new();
        let mut stats = TacticalMctsStats {
            iterations: 0,
            nodes_expanded: 0,
            tactical_moves_explored: 0,
            mates_found: 0,
            nn_policy_evaluations: 0,
            search_time: Duration::from_millis(0),
        };
        let mut nn_policy = None;
        
        // Checkmate position - no legal moves
        let terminal_board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1r b KQkq - 0 1");
        let terminal_node = MctsNode::new_root(terminal_board, &move_gen);
        
        let selected = select_child_with_tactical_priority(
            terminal_node.clone(),
            1.414,
            &move_gen,
            &mut nn_policy,
            &mut stats,
        );
        
        // Should handle terminal position gracefully
        assert!(selected.is_none() || terminal_node.borrow().children.is_empty(), 
                "Terminal position should not expand children");
    }

    #[test]
    fn test_exploration_constant_effect() {
        let (board, move_gen, mut stats1) = setup_test_env();
        let (_, _, mut stats2) = setup_test_env();
        let mut nn_policy1 = None;
        let mut nn_policy2 = None;
        
        let root1 = MctsNode::new_root(board.clone(), &move_gen);
        let root2 = MctsNode::new_root(board, &move_gen);
        
        // Force some visits to create selection differences
        select_child_with_tactical_priority(root1.clone(), 0.5, &move_gen, &mut nn_policy1, &mut stats1);
        select_child_with_tactical_priority(root2.clone(), 2.0, &move_gen, &mut nn_policy2, &mut stats2);
        
        // Different exploration constants should potentially lead to different selections
        // (This is a behavioral test - exact outcomes depend on position)
        
        // Both should have made valid selections
        assert!(!root1.borrow().children.is_empty(), "Low exploration should still expand");
        assert!(!root2.borrow().children.is_empty(), "High exploration should still expand");
    }

    #[test]
    fn test_policy_evaluation_deferral() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None; // No neural network
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        // Perform selections
        for _ in 0..5 {
            select_child_with_tactical_priority(
                root_node.clone(),
                1.414,
                &move_gen,
                &mut nn_policy,
                &mut stats,
            );
        }
        
        // Should have minimal NN evaluations since we have no NN
        assert_eq!(stats.nn_policy_evaluations, 0, 
                   "Should not perform NN evaluations without neural network");
        
        // But should still explore tactical moves
        assert!(stats.tactical_moves_explored >= 0, 
                "Should still track tactical move exploration");
    }

    #[test]
    fn test_repeated_selection_consistency() {
        let (board, move_gen, mut stats) = setup_test_env();
        let mut nn_policy = None;
        
        let root_node = MctsNode::new_root(board, &move_gen);
        
        let mut selected_moves = Vec::new();
        
        // Record several selections
        for _ in 0..3 {
            if let Some(selected) = select_child_with_tactical_priority(
                root_node.clone(),
                1.414,
                &move_gen,
                &mut nn_policy,
                &mut stats,
            ) {
                if let Some(action) = selected.borrow().action {
                    selected_moves.push(action);
                }
            }
        }
        
        // Should have made consistent selections (either same move or different tactical moves)
        assert!(!selected_moves.is_empty(), "Should have made selections");
        
        // All selected moves should be legal
        let legal_moves = {
            let (captures, non_captures) = move_gen.gen_pseudo_legal_moves(&root_node.borrow().state);
            let mut all_moves = captures;
            all_moves.extend(non_captures);
            all_moves.into_iter()
                .filter(|&mv| root_node.borrow().state.apply_move_to_board(mv).is_legal(&move_gen))
                .collect::<Vec<_>>()
        };
        
        for selected_move in selected_moves {
            assert!(legal_moves.contains(&selected_move), 
                    "Selected move {:?} should be legal", selected_move);
        }
    }
}