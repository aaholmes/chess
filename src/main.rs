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
use crate::utils::perft;

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
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
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
    // Confirm no moves allowed after fool's mate
    // let mut board = Bitboard::new_from_fen("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
    // board.print();
    // println!("perft");
    // for i in 1..5 {
    //     println!("{} {}", i, utils::perft(board.clone(), &move_gen, i, false));
    // }
    let board = Bitboard::new();
    println!("___");
    // let mut board = Bitboard::new();
    // board.print();
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e2"), bitboard::algebraic_to_sq_ind("e4"), None);
    // board.print();
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e7"), bitboard::algebraic_to_sq_ind("e6"), None);
    // board.print();
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e4"), bitboard::algebraic_to_sq_ind("e5"), None);
    // board.print();
    // board = board.make_move(bitboard::algebraic_to_sq_ind("d7"), bitboard::algebraic_to_sq_ind("d5"), None);
    // board.print();
    // board = board.make_move(bitboard::algebraic_to_sq_ind("e5"), bitboard::algebraic_to_sq_ind("d6"), None);
    // board.print();
    // let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    // for m in moves {
    //     let mut board = Bitboard::new();
    //     board = board.make_move(m.0, m.1, m.2);
    //     // if utils::print_move(&m) == "e2e4" {
    //     //     utils::perft(board.clone(), &move_gen, 2, true);
    //     // }
    //     println!("{} {}", utils::print_move(&m), utils::perft(board, &move_gen, 2, false));
    // }
    // 31 perft tests from https://www.chessprogramming.org/Perft_Results
    let board = Bitboard::new();
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 20);
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 400);
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 8902);
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 197281);
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 4865609);
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 6, false), 119060324);
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 48);
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 2039);
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 97862);
    // let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 4, false), 4085603);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 14);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 191);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 2812);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 43238);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 674624);
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 6, false), 11030083);
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 6);
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 264);
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 9467);
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 422333);
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 15833292);
    // let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 6, false), 706045033);
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 44);
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 1486);
    // let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 3, false), 62379);
    // let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 4, false), 2103487);
    // let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 5, false), 89941194);
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    board.print();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 46);
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 2079);
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 89890);
    // let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    // let move_gen = MoveGen::new();
    // assert_eq!(perft(board, &move_gen, 4, false), 3894594);
}