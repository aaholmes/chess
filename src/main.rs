mod bitboard;
use bitboard::{WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK};
mod eval;
use eval::PestoEval;
mod move_gen;

fn main() {
    for i in 0..64 {
        // println!("{} {}", i, bitboards::sq_ind_to_algebraic(i));
        assert_eq!(i, bitboard::algebraic_to_sq_ind(&bitboard::sq_ind_to_algebraic(i)));
    }
    let mut board = bitboard::Bitboard::new();
    board.print();
    // for i in 0..64 {
    //     let bit = bitboard::sq_ind_to_bit(i);
    //     let algebraic = bitboard::sq_ind_to_algebraic(i);
    //     let flipped_bit = bitboard::flip_vertically(bit);
    //     let flipped_algebraic = bitboard::sq_ind_to_algebraic(bitboard::bit_to_sq_ind(flipped_bit));
    //     println!("{} {} {} {}", bit_to_sq_ind(bit), algebraic, bit_to_sq_ind(flipped_bit), flipped_algebraic);
    // }
    board.flip_vertically().print();
    for i in 0..64 {
        println!("{} {}", i, bitboard::flip_sq_ind_vertically(i));
    }
    let pesto: PestoEval = PestoEval::new();
    board.print();
    // Move a2a4, g8f6, a1a2, d7d5
    println!("{}", pesto.eval(&board));
    board.pieces[WP] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("a2"));
    board.pieces[WP] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("a4"));
    board.w_to_move = !board.w_to_move;
    println!("{}", pesto.eval(&board));
    board.pieces[BN] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("g8"));
    board.pieces[BN] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("f6"));
    board.w_to_move = !board.w_to_move;
    println!("{}", pesto.eval(&board));
    board.pieces[WR] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("a1"));
    board.pieces[WR] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("a2"));
    board.w_to_move = !board.w_to_move;
    println!("{}", pesto.eval(&board));
    board.pieces[BP] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("d7"));
    board.pieces[BP] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("d5"));
    board.w_to_move = !board.w_to_move;
    println!("{}", pesto.eval(&board));
    assert_eq!(pesto.eval(&board), 88);
}