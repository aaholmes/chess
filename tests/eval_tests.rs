use kingfisher::bitboard::Bitboard;
use kingfisher::eval::PestoEval;

#[test]
fn test_initial_position_eval() {
    let board = Bitboard::new();
    let evaluator = PestoEval::new();
    let score = evaluator.eval(&board);
    assert_eq!(score, 0); // Initial position should be equal
}

#[test]
fn test_material_advantage() {
    let board = Bitboard::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w KQkq - 0 1"); // White is missing a rook
    let evaluator = PestoEval::new();
    let score = evaluator.eval(&board);
    assert!(score < 0); // Black should have an advantage
}

#[test]
fn test_positional_evaluation() {
    let initial_board = Bitboard::new();
    let developed_board = Bitboard::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
    let evaluator = PestoEval::new();
    let initial_score = evaluator.eval(&initial_board);
    let developed_score = evaluator.eval(&developed_board);
    println!("Initial score: {} Developed score: {}", initial_score, developed_score);
    assert!(developed_score > initial_score); // Developed position should be better for White
}