// Specifies various agents, which can use any combination of search and eval routines

use crate::bitboard::Bitboard;
use crate::eval::PestoEval;
use crate::gen_moves::{Move, MoveGen};
use crate::search::{aspiration_window_ab_search, mate_search};

pub trait Agent {
    fn get_move(&self, board: &mut Bitboard) -> Move;
}

// Simple agent that uses mate search of a given depth followed by aspiration window quiescence search of a given depth
pub struct SimpleAgent<'a> {
    pub mate_search_depth: i32,
    pub ab_search_depth: i32,
    pub verbose: bool,
    pub move_gen: &'a MoveGen,
    pub pesto: &'a PestoEval
}

impl SimpleAgent<'_> {
    pub fn new<'a>(mate_search_depth: i32, ab_search_depth: i32, verbose: bool, move_gen: &'a MoveGen, pesto: &'a PestoEval) -> SimpleAgent<'a> {
        SimpleAgent {
            mate_search_depth,
            ab_search_depth,
            verbose,
            move_gen,
            pesto
        }
    }
}

impl Agent for SimpleAgent<'_> {
    fn get_move(&self, board: &mut Bitboard) -> Move {
        let (eval, m, nodes) = mate_search(board, self.move_gen, self.mate_search_depth, self.verbose);
        if eval == 1000000 {
            if self.verbose {
                println!("Found checkmate after searching {} nodes!", nodes);
            }
            return m;
        }
        let (eval, m, n) = aspiration_window_ab_search(board, self.move_gen, self.pesto, self.ab_search_depth, self.verbose);
        if self.verbose {
            println!("Aspiration window search searched another {} nodes ({} total)! Eval: {}", n, nodes + n, eval);
        }
        m
    }
}