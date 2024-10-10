use kingfisher::bitboard::Bitboard;
use kingfisher::search::{alpha_beta, mate_search};
use kingfisher::eval::PestoEval;
use kingfisher::move_generation::MoveGen;

#[test]
fn test_mate_in_one_detection() {
    let board = Bitboard::new_from_fen("3qk3/3ppp2/8/8/8/8/3PPP2/3QK2R w K - 0 1");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();
    let (score, best_move, _) = mate_search(&board, &move_gen, 3, false);
    assert!(score > 900000); // Should detect mate
    assert_eq!(best_move.to, 63); // Rh8# (assuming 0-63 board representation)
}