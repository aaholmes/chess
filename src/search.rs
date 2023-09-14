// Alpha-beta negamax search

use crate::bitboard::Bitboard;
use crate::gen_moves::MoveGen;
use crate::eval::PestoEval;
use crate::utils;


pub(crate) fn negamax_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
    // Perform negamax search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
    // Also returns number of nodes searched
    let mut best_eval: i32 = -1000000;
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut n: i32 = 0;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let (mut eval, nodes) = negamax(&mut new_board, move_gen, pesto, depth - 1);
        eval = -eval;
        n += nodes;
        if eval > best_eval {
            best_eval = eval;
            best_move = m;
        }
    }
    (best_eval, best_move, n)

}

fn negamax(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, i32) {
    // Perform negamax search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position
    // Also returns number of nodes searched
    if depth == 0 {
        // Leaf node
        return (-pesto.eval(board), 1); // TODO: put quiescence search here
    }
    let mut best_eval: i32 = -1000000;
    let mut n: i32 = 0;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    captures.sort();
    captures.dedup();
    for m in captures {
        let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let (mut eval, nodes) = negamax(&mut new_board, move_gen, pesto, depth - 1);
        eval = -eval;
        n += nodes;
        if eval > best_eval {
            best_eval = eval;
        }
    }
    (best_eval, n)
}

pub(crate) fn alpha_beta_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32, alpha_init: i32, beta_init: i32) -> (i32, (usize, usize, Option<usize>), i32) {
    // Perform alpha-beta search from the given position
    // Exhaustive search to the given depth
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
    // Also returns number of nodes searched
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut alpha: i32 = alpha_init;
    let mut beta: i32 = beta_init;
    let mut n: i32 = 0;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    // captures.sort();
    // captures.dedup();
    for m in captures {
        let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let (mut eval, nodes) = alpha_beta(&mut new_board, move_gen, pesto, depth - 1, -beta, -alpha);
        eval = -eval;
        n += nodes;
        if eval > alpha {
            alpha = eval;
            best_move = m;
        }
        if alpha >= beta {
            break;
        }
    }
    (alpha, best_move, n)
}


fn alpha_beta(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32, mut alpha: i32, mut beta: i32) -> (i32, i32) {
    // Private recursive function used for alpha-beta search
    // External functions should call alpha_beta_search instead
    // Returns the eval (in centipawns) of the final position
    // Also returns number of nodes searched
    if depth == 0 {
        // Leaf node
        return (-pesto.eval(board), 1); // TODO: put quiescence search here
    }
    // Non-leaf node
    let mut n: i32 = 0;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    // captures.sort();
    // captures.dedup();
    for m in captures {
        let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        let (mut eval, nodes) = alpha_beta(&mut new_board, move_gen, pesto, depth - 1, -beta, -alpha);
        eval = -eval;
        n += nodes;
        if eval > alpha {
            alpha = eval;
        }
        if alpha >= beta {
            break;
        }
    }
    return (alpha, n);
}

pub(crate) fn iterative_deepening_ab_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, max_depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
    // Perform iterative deepening alpha-beta search from the given position
    // Searches to the given depth
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
    // Also returns number of nodes searched
    let mut eval: i32 = 0;
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut n: i32 = 0;
    let mut nodes = 0;
    for depth in 1..max_depth + 1 {
        (eval, best_move, nodes) = alpha_beta_search(board, move_gen, pesto, depth, -1000000, 1000000);
        n += nodes;
        println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, n, eval, utils::print_move(&best_move));
    }
    (eval, best_move, n)
}

pub(crate) fn aspiration_window_ab_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, max_depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
    // Perform aspiration window alpha-beta search from the given position
    let lower_bound_param: i32 = -25;
    let upper_bound_param: i32 = 25;

    let mut lower_bound: i32 = lower_bound_param;
    let mut upper_bound: i32 = upper_bound_param;
    let mut target_eval: i32 = board.eval;
    let mut eval: i32 = target_eval;
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut n: i32 = 0;
    let mut nodes = 0;
    for depth in 1..max_depth + 1 {
        let mut lower_window_scale: i32 = 1;
        let mut upper_window_scale: i32 = 1;
        while true {
            lower_bound = target_eval + lower_bound_param * lower_window_scale;
            upper_bound = target_eval + upper_bound_param * upper_window_scale;
            println!("Aspiration window search with window {} {}", lower_bound, upper_bound);
            (eval, best_move, nodes) = alpha_beta_search(board, move_gen, pesto, depth, lower_bound, upper_bound);
            n += nodes;
            println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, n, eval, utils::print_move(&best_move));
            if eval == lower_bound {
                println!("\nLower bound hit; retrying with larger window");
                lower_window_scale *= 9;
            } else if eval == upper_bound {
                println!("\nUpper bound hit; retrying with larger window");
                upper_window_scale *= 9;
            } else {
                println!("\nAspiration window search successful!");
                println!("Best move: {}", utils::print_move(&best_move));
                println!("Eval: {}\n", eval);
                target_eval = eval;
                break;
            }
        }
    }
    (eval, best_move, n)
}
