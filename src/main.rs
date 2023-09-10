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

mod utils;

fn main() {
    for i in 0..64 {
        // println!("{} {}", i, bitboards::sq_ind_to_algebraic(i));
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
    // let pesto: PestoEval = PestoEval::new();
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
    board.print();
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_moves(&board);
    println!("Captures:");
    for c in captures {
        println!("{} {}", sq_ind_to_algebraic(c.0), sq_ind_to_algebraic(c.1));
    }
    println!("Moves:");
    for m in moves {
        println!("{} {}", sq_ind_to_algebraic(m.0), sq_ind_to_algebraic(m.1));
    }
    // let mut rbits: Vec<i32> = vec![];
    // let mut bbits: Vec<i32> = vec![];
    // for i in 0 .. 64 {
    //     unsafe { rbits.push(_popcnt64(R_MAGICS[i] as i64)); }
    //     unsafe { println!("{} {}", i, _popcnt64(R_MAGICS[i] as i64)); }
    // }
    // println!("___");
    // for i in 0 .. 64 {
    //     unsafe { bbits.push(_popcnt64(B_MAGICS[i] as i64)); }
    //     unsafe { println!("{} {}", i, _popcnt64(B_MAGICS[i] as i64)); }
    // }
    // println!("___");
    // println!("Min, max={} {}", rbits.iter().min().unwrap(), rbits.iter().max().unwrap());
    // println!("Min, max={} {}", bbits.iter().min().unwrap(), bbits.iter().max().unwrap());
//    for i in 0 .. 64 {
//        unsafe { println!("{} {} {} {}", i, _popcnt64(R_MAGIC[i] as i64), _popcnt64(R_MAGIC2[i] as i64), _popcnt64(R_MAGIC3[i] as i64)); }
//    }
//    println!("___");
//    for i in 0 .. 64 {
//        unsafe { println!("{} {} {} {}", i, _popcnt64(B_MAGIC[i] as i64), _popcnt64(B_MAGIC2[i] as i64), _popcnt64(B_MAGIC3[i] as i64)); }
//    }
    let board = Bitboard::new();
    println!("___");
    println!("perft");
    for i in 1..4 {
        println!("{} {}", i, utils::perft(board.clone(), &move_gen, i));
    }
}