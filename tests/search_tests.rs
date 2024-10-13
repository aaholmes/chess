use kingfisher::bitboard::Bitboard;
use kingfisher::search::mate_search;
use kingfisher::eval::PestoEval;
use kingfisher::move_generation::MoveGen;

#[test]
fn test_mate_in_one_detection() {
    let board = Bitboard::new_from_fen("3qk3/3ppp2/8/8/8/8/3PPP2/3QK2R w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, best_move, _) = mate_search(&board, &move_gen, 1, false);
    assert!(score > 900000); // Should detect mate
    assert_eq!(best_move.to, 63); // Rh8# (assuming 0-63 board representation)
}

#[test]
fn test_mate_in_two_detection() {
    let board = Bitboard::new_from_fen("3qk3/3ppp2/5n2/8/8/8/3PPP2/3QK2R w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, _, _) = mate_search(&board, &move_gen, 1, false);
    assert!(score < 900000); // Should not detect mate in 1
    let (score, best_move, _) = mate_search(&board, &move_gen, 2, false);
    assert!(score > 900000); // Should detect mate in 2
    assert_eq!(best_move.to, 63); // Rh8+ (assuming 0-63 board representation)
}

#[test]
fn test_mate_in_three_detection() {
    let board = Bitboard::new_from_fen("3qk3/3pppr1/5n2/8/8/8/3PPP2/3QK1RR w K - 0 1");
    let move_gen = MoveGen::new();
    let (score, _, _) = mate_search(&board, &move_gen, 1, false);
    assert!(score < 900000); // Should not detect mate in 1
    let (score, _, _) = mate_search(&board, &move_gen, 2, false);
    assert!(score < 900000); // Should not detect mate in 2
    let (score, best_move, _) = mate_search(&board, &move_gen, 3, false);
    assert!(score > 900000); // Should detect mate in 3
    assert_eq!(best_move.to, 63); // Rh8+ (assuming 0-63 board representation)
}