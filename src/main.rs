mod bitboard;
mod bits;
mod eval;

use eval::PestoEval;
mod make_move;
mod gen_moves;
use gen_moves::MoveGen;
use crate::bitboard::{Bitboard, sq_ind_to_algebraic};
mod magic_constants;

mod utils;
mod search;
use crate::search::mate_search;

fn main() {
    for i in 0..64 {
        assert_eq!(i, bitboard::algebraic_to_sq_ind(&bitboard::sq_ind_to_algebraic(i)));
    }
    let board = bitboard::Bitboard::new();
    assert_eq!(board.pieces[14], board.pieces[0] | board.pieces[1] | board.pieces[2] | board.pieces[3] | board.pieces[4] | board.pieces[5] | board.pieces[6] | board.pieces[7] | board.pieces[8] | board.pieces[9] | board.pieces[10] | board.pieces[11]);

    let move_gen = MoveGen::new();

    // Demonstrates finding forced mate in 2 (from a famous game: Morphy vs. the Duke and the Count, 1858)
    let mut board = Bitboard::new_from_fen("4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 1 0");
    println!("___");
    board.print();
    let (eval, best_move, nodes) = mate_search(&mut board, &move_gen, 2);
    println!("Eval: {} Best move: {}{}, Nodes: {}", eval, sq_ind_to_algebraic(best_move.0), sq_ind_to_algebraic(best_move.1), nodes);

}