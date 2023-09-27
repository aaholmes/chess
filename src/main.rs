mod bitboard;
mod bits;
mod eval;

mod make_move;
mod gen_moves;
use gen_moves::MoveGen;
use crate::bitboard::{Bitboard, sq_ind_to_algebraic};
mod magic_constants;

mod utils;
mod search;
use crate::search::mate_search;

fn main() {
    // Demonstrates finding forced mate in 2 (from a famous game: Morphy vs. the Duke and the Count, 1858)
    let move_gen = MoveGen::new();
    let mut board = Bitboard::new_from_fen("4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 1 0");
    board.print();
    let (eval, best_move, nodes) = mate_search(&mut board, &move_gen, 3, true);
    println!("Eval: {} Best move: {}{}, Nodes: {}", eval, sq_ind_to_algebraic(best_move.from), sq_ind_to_algebraic(best_move.to), nodes);
}