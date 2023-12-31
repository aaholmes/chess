// Utility functions

use crate::bitboard::{Bitboard, coords_to_sq_ind, sq_ind_to_algebraic, sq_ind_to_bit};
use crate::gen_moves::{Move, MoveGen};

// Print u64 as an 8x8 board
pub fn print_bits(bits: u64) {
    println!("  +-----------------+");
    for rank in (0..8).rev() {
        print!("{} | ", rank + 1);
        for file in 0..8 {
            let sq_ind = coords_to_sq_ind(file, rank);
            let bit = sq_ind_to_bit(sq_ind);
            if bit & bits != 0 {
                print!("X ");
            } else {
                print!(". ");
            }
        }
        println!("|");
    }
    println!("  +-----------------+");
    println!("    a b c d e f g h");
}

// Print a move in algebraic notation
pub fn print_move(the_move: &Move) -> String {
    let from = sq_ind_to_algebraic(the_move.from);
    let to = sq_ind_to_algebraic(the_move.to);
    let mut promotion = String::from("");
    if the_move.promotion.is_some() {
        promotion = String::from("=");
        match the_move.promotion.unwrap() / 2 {
            1 => promotion.push('N'),
            2 => promotion.push('B'),
            3 => promotion.push('R'),
            4 => promotion.push('Q'),
            _ => panic!("Invalid promotion piece")
        }
    }
    format!("{}{}{}", from, to, promotion)
}

// Perft - performance test
// Count the number of nodes in a tree of depth n
// For debugging only
pub fn perft(board: Bitboard, move_gen: &MoveGen, depth: u8, verbose: bool) -> u64 {
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    let mut nodes = 0;
    if depth == 1 {
        if verbose {
            println!("Moves: {:?}", captures.iter().map(print_move).collect::<Vec<String>>());
        }
        let mut test_board: Bitboard;
        for i in captures {
            test_board = board.make_move(i);
            if test_board.is_legal(move_gen) {
                nodes += 1;
            }
        }
        return nodes;
    }
    for c in captures {
        if verbose {
            println!("{} {}", print_move(&c), depth);
        }
        let new_board = board.make_move(c);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        nodes += perft(new_board, move_gen, depth - 1, verbose);
    }
    nodes
}


// 33 perft tests from https://www.chessprogramming.org/Perft_Results
// These are all the tests available on that page with up to 1 billion nodes each
// More optimization is needed to handle more nodes in a reasonable amount of time
// Current tests run in about 90 seconds on Apple M2
#[test]
fn test_start_pos_perft1() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 20);
}
#[test]
fn test_start_pos_perft2() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 400);
}
#[test]
fn test_start_pos_perft3() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 8902);
}
#[test]
fn test_start_pos_perft4() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 197281);
}
#[test]
fn test_start_pos_perft5() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 4865609);
}
#[test]
fn test_start_pos_perft6() {
    let board = Bitboard::new();
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 6, false), 119060324);
}

#[test]
fn test_pos1_perft1() {
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 48);
}
#[test]
fn test_pos1_perft2() {
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 2039);
}
#[test]
fn test_pos1_perft3() {
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 97862);
}
#[test]
fn test_pos1_perft4() {
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 4085603);
}
#[test]
fn test_pos1_perft5() {
    let board = Bitboard::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 193690690);
}

#[test]
fn test_pos2_perft1() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 14);
}
#[test]
fn test_pos2_perft2() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 191);
}
#[test]
fn test_pos2_perft3() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 2812);
}
#[test]
fn test_pos2_perft4() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 43238);
}
#[test]
fn test_pos2_perft5() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 674624);
}
#[test]
fn test_pos2_perft6() {
    let board = Bitboard::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 6, false), 11030083);
}

#[test]
fn test_pos3_perft1() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 6);
}
#[test]
fn test_pos3_perft2() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 264);
}
#[test]
fn test_pos3_perft3() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 9467);
}
#[test]
fn test_pos3_perft4() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 422333);
}
#[test]
fn test_pos3_perft5() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 15833292);
}
#[test]
fn test_pos3_perft6() {
    let board = Bitboard::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 6, false), 706045033);
}

#[test]
fn test_pos4_perft1() {
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 44);
}
#[test]
fn test_pos4_perft2() {
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 1486);
}
#[test]
fn test_pos4_perft3() {
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 62379);
}
#[test]
fn test_pos4_perft4() {
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 2103487);
}
#[test]
fn test_pos4_perft5() {
    let board = Bitboard::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 89941194);
}

#[test]
fn test_pos5_perft1() {
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 1, false), 46);
}
#[test]
fn test_pos5_perft2() {
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 2, false), 2079);
}
#[test]
fn test_pos5_perft3() {
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 3, false), 89890);
}
#[test]
fn test_pos5_perft4() {
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 4, false), 3894594);
}
#[test]
fn test_pos5_perft5() {
    let board = Bitboard::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let move_gen = MoveGen::new();
    assert_eq!(perft(board, &move_gen, 5, false), 164075551);
}