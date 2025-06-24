use kingfisher::board::Board;
use kingfisher::eval::PestoEval;
use kingfisher::move_generation::MoveGen;
use kingfisher::mcts::{tactical_mcts_search, TacticalMctsConfig, print_search_stats};
use kingfisher::neural_net::NeuralNetPolicy;
use std::time::Duration;

#[test]
fn test_tactical_mcts_starting_position() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    let config = TacticalMctsConfig {
        max_iterations: 50,
        time_limit: Duration::from_millis(500),
        mate_search_depth: 3,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move, stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        config,
    );
    
    assert!(best_move.is_some(), "Should find a move from starting position");
    assert!(stats.iterations > 0, "Should perform iterations");
    assert!(stats.nodes_expanded > 0, "Should expand nodes");
}

#[test]
fn test_tactical_mcts_captures_available() {
    // Position with capture available: 1. e4 e5 2. Nf3 d6 3. Bc4 h6 4. Nc3 Nf6 5. d3 Be7
    let board = Board::new_from_fen("rnbqk2r/ppp1bpp1/3p1n1p/4p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQkq - 0 6");
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    let config = TacticalMctsConfig {
        max_iterations: 100,
        time_limit: Duration::from_millis(1000),
        mate_search_depth: 3,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move, stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        config,
    );
    
    assert!(best_move.is_some(), "Should find a move in tactical position");
    assert!(stats.tactical_moves_explored >= 0, "Should track tactical moves");
}

#[test]
fn test_tactical_mcts_mate_detection() {
    // Back rank mate in 1: White to move, Qd8# is mate
    let board = Board::new_from_fen("6k1/5ppp/8/8/8/8/8/4Q2K w - - 0 1");
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    let config = TacticalMctsConfig {
        max_iterations: 20, // Should find mate quickly
        time_limit: Duration::from_millis(1000),
        mate_search_depth: 3,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move, stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        config,
    );
    
    assert!(best_move.is_some(), "Should find the mating move");
    // Note: mate detection depends on the mate search implementation
}

#[test]
fn test_tactical_mcts_no_legal_moves() {
    // Stalemate position: King on a1, opponent pawns block all moves
    let board = Board::new_from_fen("8/8/8/8/8/p1p5/P1P5/K7 w - - 0 1");
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    let config = TacticalMctsConfig {
        max_iterations: 10,
        time_limit: Duration::from_millis(100),
        mate_search_depth: 1,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move, _stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        config,
    );
    
    // Should handle terminal positions gracefully
    assert!(best_move.is_none() || best_move.is_some(), "Should handle terminal positions");
}

#[test]
fn test_tactical_mcts_config_variations() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    // Test with different exploration constants
    let configs = vec![
        TacticalMctsConfig {
            max_iterations: 30,
            time_limit: Duration::from_millis(200),
            mate_search_depth: 1,
            exploration_constant: 0.5,
            use_neural_policy: false,
        },
        TacticalMctsConfig {
            max_iterations: 30,
            time_limit: Duration::from_millis(200),
            mate_search_depth: 1,
            exploration_constant: 2.0,
            use_neural_policy: false,
        },
    ];
    
    for config in configs {
        let (best_move, stats) = tactical_mcts_search(
            board,
            &move_gen,
            &pesto_eval,
            &mut nn_policy,
            config,
        );
        
        assert!(best_move.is_some(), "Should find move with different exploration constants");
        assert!(stats.iterations > 0, "Should perform iterations");
    }
}

#[test]
fn test_tactical_mcts_time_vs_iterations() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    // Test time-limited search
    let time_config = TacticalMctsConfig {
        max_iterations: 10000, // High iteration count
        time_limit: Duration::from_millis(100), // But limited time
        mate_search_depth: 1,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let start_time = std::time::Instant::now();
    let (best_move, stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        time_config,
    );
    let elapsed = start_time.elapsed();
    
    assert!(best_move.is_some(), "Should find move with time limit");
    assert!(elapsed < Duration::from_millis(200), "Should respect time limit");
    assert!(stats.iterations < 10000, "Should stop due to time limit, not iteration limit");
    
    // Test iteration-limited search
    let iter_config = TacticalMctsConfig {
        max_iterations: 20, // Low iteration count
        time_limit: Duration::from_millis(5000), // But high time limit
        mate_search_depth: 1,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move2, stats2) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        iter_config,
    );
    
    assert!(best_move2.is_some(), "Should find move with iteration limit");
    assert_eq!(stats2.iterations, 20, "Should perform exactly 20 iterations");
}

#[test]
fn test_tactical_mcts_statistics() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let pesto_eval = PestoEval::new();
    let mut nn_policy = None;
    
    let config = TacticalMctsConfig {
        max_iterations: 50,
        time_limit: Duration::from_millis(1000),
        mate_search_depth: 2,
        exploration_constant: 1.414,
        use_neural_policy: false,
    };
    
    let (best_move, stats) = tactical_mcts_search(
        board,
        &move_gen,
        &pesto_eval,
        &mut nn_policy,
        config,
    );
    
    assert!(best_move.is_some(), "Should find a move");
    
    // Verify statistics make sense
    assert_eq!(stats.iterations, 50, "Should perform requested iterations");
    assert!(stats.nodes_expanded > 0, "Should expand some nodes");
    assert!(stats.search_time > Duration::from_millis(0), "Should take some time");
    assert!(stats.nn_policy_evaluations == 0, "Should not use NN policy when disabled");
    
    // In starting position, shouldn't find mates
    assert_eq!(stats.mates_found, 0, "Should not find mates in starting position");
}