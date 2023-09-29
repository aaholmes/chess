mod agent;
use agent::{Agent, SimpleAgent};
mod arena;
use arena::Arena;
mod bitboard;
mod bits;
mod eval;
use eval::PestoEval;
mod gen_moves;
use gen_moves::MoveGen;
mod magic_constants;
mod make_move;
mod search;
mod utils;

fn main() {
    // Play a game between two simple agents
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();
    let white = SimpleAgent::new(5, 2, false, &move_gen, &pesto);
    let black = SimpleAgent::new(5, 2, false, &move_gen, &pesto);
    let arena = Arena::new(&white, &black, 10);
    arena.play_game();
}