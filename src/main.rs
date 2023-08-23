mod bitboard;

mod eval;
use eval::PestoEval;

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
    // Move d2d4, g8f6, c1f4
    board.pieces[0] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("d2"));
    board.pieces[0] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("d4"));
    board.pieces[3] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("g8"));
    board.pieces[3] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("f6"));
    board.pieces[4] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("c1"));
    board.pieces[4] ^= bitboard::sq_ind_to_bit(bitboard::algebraic_to_sq_ind("f4"));
    assert_eq!(pesto.eval(board), 25);
}