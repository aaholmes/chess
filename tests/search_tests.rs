use kingfisher::board::Board;
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
fn test_capture_ordering() {
    let board = Board::new_from_fen("r1bqk2r/pp3Npp/2n1p1PP/1Pp5/3p4/3P1Q2/PP3PP1/R1B2RK1 w - - 0 1");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    board.print();
    let (mut captures, _) = move_gen.gen_pseudo_legal_moves_with_evals(&board, &pesto);

    let mut capture_vals: Vec<i32> = captures.iter().map(|m| move_gen.mvv_lva(&board, m.from, m.to)).collect();
    println!("{} Captures:", captures.len());
    for (i, m) in captures.iter().enumerate() {
        println!("{}. {} ({})", i+1, m, capture_vals[i]);
    }

    // Check that captures are ordered by MVV-LVA score in descending order
    for i in 1..captures.len() {
        assert!(capture_vals[i-1] >= capture_vals[i], "Moves not properly ordered at index {}", i);
    }
}

#[test]
fn test_alpha_beta_pruning() {
    let mut board = BoardStack::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    let depth = 4;
    let (score1, best_move1, nodes1) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, -1000000, 1000000, 0, false);

    // Now search with a narrow window that should cause more pruning
    let (score2, best_move2, nodes2) = alpha_beta_search(&mut board, &move_gen, &pesto, depth, score1 - 50, score1 + 50, 0, false);

    println!("Wide window - Score: {}, Best move: {}, Nodes: {}", score1, best_move1, nodes1);
    println!("Narrow window - Score: {}, Best move: {}, Nodes: {}", score2, best_move2, nodes2);

    // The scores should be the same, but nodes2 should be significantly less than nodes1
    assert_eq!(score1, score2, "Scores don't match");
    assert!(nodes2 < 2 * nodes1 / 3, "Not enough pruning. Nodes1: {}, Nodes2: {}", nodes1, nodes2);
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