/// Helper function to check if a move is a capture
fn is_capture(board: &Board, mv: &Move) -> bool {
    let target_square_bb = 1u64 << mv.to;
    let opponent_color = !board.w_to_move as usize;
    
    // Check if the target square is occupied by an opponent's piece
    (board.pieces_occ[opponent_color] & target_square_bb) != 0
} 