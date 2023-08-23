mod board;
use board::bitboards;
// use crate::board::bitboards::bit_to_sq_ind;

mod eval;
use crate::eval::pesto::PestoEval;

fn main() {
    for i in 0..64 {
        // println!("{} {}", i, bitboards::sq_ind_to_algebraic(i));
        assert_eq!(i, bitboards::algebraic_to_sq_ind(&bitboards::sq_ind_to_algebraic(i)));
    }
    let mut board = bitboards::Bitboard::new();
    board.print();
    // for i in 0..64 {
    //     let bit = bitboards::sq_ind_to_bit(i);
    //     let algebraic = bitboards::sq_ind_to_algebraic(i);
    //     let flipped_bit = bitboards::flip_vertically(bit);
    //     let flipped_algebraic = bitboards::sq_ind_to_algebraic(bitboards::bit_to_sq_ind(flipped_bit));
    //     println!("{} {} {} {}", bit_to_sq_ind(bit), algebraic, bit_to_sq_ind(flipped_bit), flipped_algebraic);
    // }
    board.flip_vertically().print();
    for i in 0..64 {
        println!("{} {}", i, bitboards::flip_sq_ind_vertically(i));
    }
    let pesto: PestoEval = PestoEval::new();
    board.print();
    // Move d2d4, g8f6, c1f4
    board.pieces[0] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("d2"));
    board.pieces[0] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("d4"));
    board.pieces[3] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("g8"));
    board.pieces[3] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("f6"));
    board.pieces[4] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("c1"));
    board.pieces[4] ^= bitboards::sq_ind_to_bit(bitboards::algebraic_to_sq_ind("f4"));
    assert_eq!(pesto.eval(board), 25);
}