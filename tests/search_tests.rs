use kingfisher::boardstack::BoardStack;
use kingfisher::search::mate_search;
use kingfisher::move_generation::MoveGen;
use kingfisher::search::{alpha_beta_search, iterative_deepening_ab_search};
use kingfisher::eval::PestoEval;

#[test]
fn test_mate_in_one_detection() {
    let mut board = BoardStack::new_from_fen("3qk3/3ppp2/8/8/8/8/3PPP2/3QK2R w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, best_move, _) = mate_search(&mut board, &move_gen, 1, false);
    assert!(score > 900000); // Should detect mate
    assert_eq!(best_move.to, 63); // Rh8# (assuming 0-63 board representation)
}

#[test]
fn test_mate_in_two_detection() {
    let mut board = BoardStack::new_from_fen("3qk3/3ppp2/5n2/8/8/8/3PPP2/3QK2R w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, _, _) = mate_search(&mut board, &move_gen, 1, false);
    assert!(score < 900000); // Should not detect mate in 1
    let (score, best_move, _) = mate_search(&mut board, &move_gen, 2, false);
    assert!(score > 900000); // Should detect mate in 2
    assert_eq!(best_move.to, 63); // Rh8+ (assuming 0-63 board representation)
}

#[test]
fn test_mate_in_three_detection() {
    let mut board = BoardStack::new_from_fen("3qk3/3pppr1/5n2/8/8/8/3PPP2/3QK1RR w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, _, _) = mate_search(&mut board, &move_gen, 1, false);
    assert!(score < 900000); // Should not detect mate in 1
    let (score, _, _) = mate_search(&mut board, &move_gen, 2, false);
    assert!(score < 900000); // Should not detect mate in 2
    let (score, best_move, _) = mate_search(&mut board, &move_gen, 3, false);
    assert!(score > 900000); // Should detect mate in 3
    assert_eq!(best_move.to, 63); // Rh8+ (assuming 0-63 board representation)
}

#[test]
fn test_alpha_beta_pruning_effectiveness() {
    let mut board = BoardStack::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    let depth = 4;
    let INFINITY = 1000000;
    let (score_full, _, nodes_full) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, -INFINITY, INFINITY, 0, true);

    // Now search with a narrow window
    let (score_narrow, _, nodes_narrow) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, score_full - 50, score_full + 50, 0, true);

    println!("Full window (White) - Score: {}, Nodes: {}", score_full, nodes_full);
    println!("Narrow window (White) - Score: {}, Nodes: {}", score_narrow, nodes_narrow);

    assert_eq!(score_full, score_narrow, "Scores don't match for White");
    assert!(nodes_narrow < nodes_full * 2 / 3, "Not enough pruning for White. Full: {}, Narrow: {}", nodes_full, nodes_narrow);

    // Test for black
    board = BoardStack::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4");
    let (score_full_black, _, nodes_full_black) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, -INFINITY, INFINITY, 0, false);
    let (score_narrow_black, _, nodes_narrow_black) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, score_full_black - 50, score_full_black + 50, 0, false);

    println!("Full window (Black) - Score: {}, Nodes: {}", score_full_black, nodes_full_black);
    println!("Narrow window (Black) - Score: {}, Nodes: {}", score_narrow_black, nodes_narrow_black);

    assert_eq!(score_full_black, score_narrow_black, "Scores don't match for Black");
    assert!(nodes_narrow_black < nodes_full_black * 2 / 3, "Not enough pruning for Black. Full: {}, Narrow: {}", nodes_full_black, nodes_narrow_black);
}

#[test]
fn test_search_stability() {
    let mut board = BoardStack::new();
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    let max_depth = 3;
    let q_search_max_depth = 6;
    let (score1, best_move1, _) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, false);
    let (score2, best_move2, _) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, false);

    // The scores and best moves should be the same across multiple runs
    assert_eq!(score1, score2);
    assert_eq!(best_move1, best_move2);
}