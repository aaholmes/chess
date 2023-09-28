mod agent;
use crate::agent::{Agent, SimpleAgent};
mod bitboard;
use crate::bitboard::{Bitboard, sq_ind_to_algebraic};
mod bits;
mod eval;
use crate::eval::PestoEval;
mod gen_moves;
use gen_moves::MoveGen;
mod magic_constants;
mod make_move;
mod search;
mod utils;

fn main() {
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();
    let simple_agent = SimpleAgent::new(5, 2, true, &move_gen, &pesto);

    // Play a move from starting position
    let mut board = Bitboard::new();
    board.print();
    let m = simple_agent.get_move(&mut board);
    println!("Move: {}{}", sq_ind_to_algebraic(m.from), sq_ind_to_algebraic(m.to));
    println!("\n---\n");

    // Find forced mate in 2 (from a famous game: Morphy vs. the Duke and the Count, 1858)
    let mut board = Bitboard::new_from_fen("4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 1 0");
    board.print();
    let m = simple_agent.get_move(&mut board);
    println!("Move: {}{}", sq_ind_to_algebraic(m.from), sq_ind_to_algebraic(m.to));
}