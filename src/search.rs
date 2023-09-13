// Alpha-beta negamax search

use crate::bitboard::Bitboard;
use crate::gen_moves::MoveGen;
use crate::eval::PestoEval;
use crate::utils;


pub(crate) fn negamax_search(board: &Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, (usize, usize, Option<usize>)) {
    // Perform negamax search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
    let mut best_eval: i32 = -1000000;
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let eval: i32 = -negamax(&new_board, move_gen, pesto, depth - 1);
        if eval > best_eval {
            best_eval = eval;
            best_move = m;
        }
    }
    (best_eval, best_move)

}

fn negamax(board: &Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> i32 {
    // Perform negamax search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position
    if depth == 0 {
        // Leaf node
        return -pesto.eval(board); // TODO: put quiescence search here
    }
    let mut best_eval: i32 = -1000000;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let eval: i32 = -negamax(&new_board, move_gen, pesto, depth - 1);
        if eval > best_eval {
            best_eval = eval;
        }
    }
    best_eval
}

pub(crate) fn alpha_beta_search(board: &Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, (usize, usize, Option<usize>)) {
    // Perform alpha-beta search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut alpha: i32 = -1000000;
    let mut beta: i32 = 1000000;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let eval: i32 = -alpha_beta(&new_board, move_gen, pesto, depth - 1, -beta, -alpha);
        if eval > alpha {
            alpha = eval;
            best_move = m;
        }
        if alpha >= beta {
            break;
        }
    }
    (alpha, best_move)
}


fn alpha_beta(board: &Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32, mut alpha: i32, mut beta: i32) -> i32 {
    // Private recursive function used for alpha-beta search
    // External functions should call alpha_beta_search instead
    // Returns the eval (in centipawns) of the final position
    if depth == 0 {
        // Leaf node
        return -pesto.eval(board); // TODO: put quiescence search here
    }
    // Non-leaf node
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let eval: i32 = -alpha_beta(&new_board, move_gen, pesto, depth - 1, -beta, -alpha);
        if eval > alpha {
            alpha = eval;
        }
        if alpha >= beta {
            break;
        }
    }
    return alpha;
}