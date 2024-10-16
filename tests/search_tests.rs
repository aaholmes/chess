use kingfisher::boardstack::BoardStack;
use kingfisher::search::{mate_search, negamax_search};
use kingfisher::move_generation::MoveGen;
use kingfisher::search::{alpha_beta_search, iterative_deepening_ab_search};
use kingfisher::eval::PestoEval;
use kingfisher::transposition::TranspositionTable;

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
    let mut tt = TranspositionTable::new();

    let depth = 4;
    let INFINITY = 1000000;
    let (score_full, _, nodes_full) = alpha_beta_search(&mut board, &move_gen, &pesto, &mut tt, depth, -INFINITY, INFINITY, 0, true);

    // Now search with a narrow window
    let (score_narrow, _, nodes_narrow) = alpha_beta_search(&mut board, &move_gen, &pesto, &mut tt, depth, score_full - 50, score_full + 50, 0, true);

    println!("Full window (White) - Score: {}, Nodes: {}", score_full, nodes_full);
    println!("Narrow window (White) - Score: {}, Nodes: {}", score_narrow, nodes_narrow);

    assert_eq!(score_full, score_narrow, "Scores don't match for White");
    assert!(nodes_narrow < nodes_full * 2 / 3, "Not enough pruning for White. Full: {}, Narrow: {}", nodes_full, nodes_narrow);

    // Test for black
    board = BoardStack::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4");
    let (score_full_black, _, nodes_full_black) = alpha_beta_search(&mut board, &move_gen, &pesto, &mut tt, depth, -INFINITY, INFINITY, 0, false);
    let (score_narrow_black, _, nodes_narrow_black) = alpha_beta_search(&mut board, &move_gen, &pesto, &mut tt, depth, score_full_black - 50, score_full_black + 50, 0, false);

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

    let max_depth = 6;
    let q_search_max_depth = 99;
    let (depth1, score1, best_move1, eval1) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, None, false);
    let (depth2, score2, best_move2, eval2) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, None, false);

    // The scores and best moves should be the same across multiple runs
    assert_eq!(depth1, depth2);
    assert_eq!(score1, score2);
    assert_eq!(best_move1, best_move2);
    assert_eq!(eval1, eval2);
}

// Count the number of nodes visited in the negamax search and the alpha-beeta search, for a depth of up to 4 ply.
// This is used to determine the effectiveness of the alpha-beta search.
#[test]
fn test_pruning() {
    let mut board = BoardStack::new();
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();
    let mut tt = TranspositionTable::new();
    for depth in 1..6 {
        let (negamax_eval, negamax_move, negamax_nodes) = negamax_search(&mut board, &move_gen, &pesto, depth);
        let (alpha_beta_eval, alpha_beta_move, alpha_beta_nodes) = alpha_beta_search(&mut board, &move_gen, &pesto, &mut tt, depth, -1000000, 1000000, 0, false);
        assert!(negamax_eval == alpha_beta_eval, "Evals don't match for depth {}, negamax eval: {}, alpha-beta eval: {}", depth, negamax_eval, alpha_beta_eval);
        assert!(negamax_move == alpha_beta_move, "Moves don't match for depth {}, negamax move: {}, alpha-beta move: {}", depth, negamax_move.print_algebraic(), alpha_beta_move.print_algebraic());
        println!("Move, eval = {}, {}", &negamax_move.print_algebraic(), negamax_eval);
        println!("Depth: {}, Negamax nodes: {}, Alpha-beta nodes: {}", depth, negamax_nodes, alpha_beta_nodes);
    }
}