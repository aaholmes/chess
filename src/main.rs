mod board;
use board::bitboards;
//mod eval;
//use eval::pesto;
fn main() {
    for i in 0..64 {
        // println!("{} {}", i, bitboards::sq_ind_to_algebraic(i));
        assert_eq!(i, bitboards::algebraic_to_sq_ind(&bitboards::sq_ind_to_algebraic(i)));
    }
    let mut board = bitboards::Bitboard::new();
    board.print();
}