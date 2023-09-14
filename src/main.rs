mod bitboard;
mod bits;
mod eval;

use std::arch::x86_64::_popcnt64;
use eval::PestoEval;
mod make_move;
mod gen_moves;
use gen_moves::MoveGen;
use crate::bitboard::{Bitboard, sq_ind_to_algebraic};
mod magic_constants;
use magic_constants::{R_MAGICS, B_MAGICS};
use crate::search::{alpha_beta_search, iterative_deepening_ab_search, negamax_search};
use crate::utils::perft;

mod utils;
mod search;
// mod search;

fn main() {
    for i in 0..64 {
        assert_eq!(i, bitboard::algebraic_to_sq_ind(&bitboard::sq_ind_to_algebraic(i)));
    }
    let mut board = bitboard::Bitboard::new();
    assert_eq!(board.pieces[14], board.pieces[0] | board.pieces[1] | board.pieces[2] | board.pieces[3] | board.pieces[4] | board.pieces[5] | board.pieces[6] | board.pieces[7] | board.pieces[8] | board.pieces[9] | board.pieces[10] | board.pieces[11]);
    // board.print();
    // // for i in 0..64 {
    // //     let bit = bitboard::sq_ind_to_bit(i);
    // //     let algebraic = bitboard::sq_ind_to_algebraic(i);
    // //     let flipped_bit = bitboard::flip_vertically(bit);
    // //     let flipped_algebraic = bitboard::sq_ind_to_algebraic(bitboard::bit_to_sq_ind(flipped_bit));
    // //     println!("{} {} {} {}", bit_to_sq_ind(bit), algebraic, bit_to_sq_ind(flipped_bit), flipped_algebraic);
    // // }
    // board.flip_vertically().print();
    // for i in 0..64 {
    //     println!("{} {}", i, bitboard::flip_sq_ind_vertically(i));
    // }
    let pesto: PestoEval = PestoEval::new();
    // board.print();
    // // Move e2e4, e7e5, Ng1f3, Nb8c6, Bf1c4, Bc8c5, O-O
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("e2"), bitboard::algebraic_to_sq_ind("e4"), None);
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("e7"), bitboard::algebraic_to_sq_ind("e5"), None);
    board = board.make_move(bitboard::algebraic_to_sq_ind("d1"), bitboard::algebraic_to_sq_ind("h5"), None);
    board = board.make_move(bitboard::algebraic_to_sq_ind("d7"), bitboard::algebraic_to_sq_ind("d5"), None);
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("g1"), bitboard::algebraic_to_sq_ind("f3"), None);
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("b8"), bitboard::algebraic_to_sq_ind("c6"), None);
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("f1"), bitboard::algebraic_to_sq_ind("c4"), None);
    // println!("{}", pesto.eval(&board));
    board = board.make_move(bitboard::algebraic_to_sq_ind("f8"), bitboard::algebraic_to_sq_ind("c5"), None);
    // println!("{}", pesto.eval(&board));
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e1"), bitboard::algebraic_to_sq_ind("g1"), None);
    // println!("{}", pesto.eval(&board));
    // assert_eq!(pesto.eval(&board), 52);
    board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e2"), bitboard::algebraic_to_sq_ind("e4"), None);
    board.print();
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board, &pesto);
    println!("Captures:");
    for c in captures {
        println!("{} {}", sq_ind_to_algebraic(c.0), sq_ind_to_algebraic(c.1));
    }
    println!("Moves:");
    for m in moves {
        let mut new_board = board.make_move(m.0, m.1, m.2);
        println!("{} {} {}", sq_ind_to_algebraic(m.0), sq_ind_to_algebraic(m.1), pesto.eval(&new_board));
    }
    let mut board = Bitboard::new();
    println!("___");
    board.print();
    let use_ab: bool = true;
    iterative_deepening_ab_search(&mut board, &move_gen, &PestoEval::new(), 8);
    // for i in 1..8 {
    //     let (eval, m, n) = {
    //         if use_ab {
    //             alpha_beta_search(&board, &move_gen, &PestoEval::new(), i)
    //         } else {
    //             negamax_search(&board, &move_gen, &PestoEval::new(), i)
    //         }
    //     };
    //     println!("At depth {}, searched {} nodes. best eval and move are {} {}", i, n, eval, utils::print_move(&m));
    // }
    // for _i in 0..10 {
    //     let (eval, m, n) = {
    //         if use_ab {
    //             alpha_beta_search(&board, &move_gen, &PestoEval::new(), 3)
    //         } else {
    //             negamax_search(&board, &move_gen, &PestoEval::new(), 3)
    //         }
    //     };
    //     board = board.make_move(m.0, m.1, m.2);
    //     println!("At depth {}, searched {} nodes. best eval and move are {} {}", 3, n, eval, utils::print_move(&m));
    //     board.print();
    // }
}