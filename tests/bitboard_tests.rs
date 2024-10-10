use kingfisher::bitboard::Bitboard;
use kingfisher::move_generation::MoveGen;

#[test]
fn test_initial_position() {
    let board = Bitboard::new();
    assert_eq!(board.get_piece(0), Some(6)); // White Rook
    assert_eq!(board.get_piece(4), Some(10)); // White King
    assert_eq!(board.get_piece(63), Some(7)); // Black Rook
    assert!(board.w_to_move);
    assert!(board.w_castle_k && board.w_castle_q && board.b_castle_k && board.b_castle_q);
}

#[test]
fn test_make_move() {
    let mut board = Bitboard::new();
    let move_gen = MoveGen::new();
    let moves = move_gen.gen_pseudo_legal_moves(&board);
    let e4_move = moves.1.iter().find(|&m| m.from == 12 && m.to == 28).unwrap();

    board = board.make_move(*e4_move);
    assert_eq!(board.get_piece(28), Some(0)); // White Pawn on e4
    assert_eq!(board.get_piece(12), None); // e2 is empty
    assert!(!board.w_to_move);
}

#[test]
fn test_is_legal() {
    // Legal position
    let board = Bitboard::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4");
    let move_gen = MoveGen::new();
    assert!(board.is_legal(&move_gen));
}

#[test]
fn test_is_illegal() {
    // Illegal position (black king in check)
    let illegal_board = Bitboard::new_from_fen("r1bq1bnr/pppp1kpp/2n2p2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R w KQkq - 0 5");
    let move_gen = MoveGen::new();
    assert!(!illegal_board.is_legal(&move_gen));
}