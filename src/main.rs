mod board;
use board::bitboards;
use crate::board::bitboards::bit_to_sq_ind;

//mod eval;
//use eval::pesto;
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
}