// Arena for staging engine matches

use crate::agent::Agent;
use crate::bitboard::{Bitboard, sq_ind_to_algebraic};
use crate::utils::print_move;

pub(crate) struct Arena<'a> {
    white_player: &'a dyn Agent,
    black_player: &'a dyn Agent,
    max_moves: i32
}

impl Arena<'_> {
    pub fn new<'a>(white_player: &'a dyn Agent, black_player: &'a dyn Agent, max_moves: i32) -> Arena<'a> {
        Arena {
            white_player,
            black_player,
            max_moves
        }
    }
    pub fn play_game(&self) {
        println!("Playing game (max {} moves)", self.max_moves);
        let mut board = Bitboard::new();
        board.print();
        for i in 0..self.max_moves {
            println!("Move {}", i);
            if i % 2 == 0 {
                // White to move
                let m = self.white_player.get_move(&mut board);
                println!("Move: {}", print_move(&m));
                board = board.make_move(m);
            } else {
                // Black to move
                let m = self.black_player.get_move(&mut board);
                println!("Move: {}", print_move(&m));
                board = board.make_move(m);
            }
        }
    }
}