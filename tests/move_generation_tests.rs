use kingfisher::board::Board;
use kingfisher::move_generation::MoveGen;

#[test]
fn test_initial_move_count() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 20); // 16 pawn moves + 4 knight moves
}

#[test]
fn test_knight_moves() {
    let board = Board::new_from_fen("K7/8/k7/8/4N3/8/8/8 w - - 0 1");
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 11); // Knight should have 8 possible moves and king should have 3 (moving into check is OK for this function)
}

#[test]
fn test_pawn_promotion() {
    let board = Board::new_from_fen("1r6/P7/K7/8/k7/8/8/8 w - - 0 1");
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 12); // 4 promotions, 4 capture-promotions, 4 king moves (moving into check is OK for this function)
}