// Alpha-beta negamax search

use std::process::abort;
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
    let beta: i32 = beta_init;
    let mut n: i32 = 0;
    println!("Checking for checkmate and stalemate");
    let (checkmate, stalemate) = board.is_checkmate_or_stalemate(move_gen);
    println!("Checkmate and stalemate checked");
    println!("Checkmate: {} Stalemate: {}", checkmate, stalemate);
    if checkmate {
        println!("AB search: Quiescence: Checkmate!");
        return (1000000, best_move, 1);
    } else if stalemate {
        println!("Quiescence: Stalemate!");
        return (0, best_move, 1);
    }
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    // captures.sort();
    // captures.dedup();
    for m in captures {
        println!("Considering move {} at root of search tree", utils::print_move(&m));
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
    println!("Alpha beta search at depth {} searched {} nodes. Best eval and move are {} {}", depth, n, alpha, utils::print_move(&best_move));
    (alpha, best_move, n)
}

fn alpha_beta(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, depth: i32, mut alpha: i32, beta: i32) -> (i32, i32) {
    // Private recursive function used for alpha-beta search
    // External functions should call alpha_beta_search instead
    // Returns the eval (in centipawns) of the final position
    // Also returns number of nodes searched
    println!("Alpha beta search at depth {} with alpha {} and beta {}", depth, alpha, beta);
    if depth == 0 {
        // Leaf node
        let (eval, nodes) = q_search_consistent_side_to_move_for_final_eval(board, move_gen, pesto, alpha, beta, true);
        // println!("Outcome of Q search consistent side to move: {} {}", eval, nodes);
        return (eval, nodes);
        // return (-pesto.eval(board), 1);
    }
    // Non-leaf node
    let mut n: i32 = 1;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
    captures.extend(moves);
    // Remove duplicates in captures. This is necessary because shifting piece moves to the edge of the board may or may not be captures.
    // captures.sort();
    // captures.dedup();
    for m in captures {
        // println!("Considering move {}", utils::print_move(&m));
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
            // println!("Inner Alpha beta search at depth {} searched {} nodes. Best eval and move are {} {}", depth, n, alpha, utils::print_move(&m));
            break;
        }
    }
    (alpha, n)
}

// pub(crate) fn iterative_deepening_ab_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, max_depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
//     // Perform iterative deepening alpha-beta search from the given position
//     // Searches to the given depth
//     // Returns the eval (in centipawns) of the final position, as well as the first move
//     // to play from the current position
//     // Also returns number of nodes searched
//     let mut eval: i32 = 0;
//     let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
//     let mut n: i32 = 0;
//     let nodes: i32 = 0;
//     for d in 1..max_depth / 2 + 1 {
//         let depth = 2 * d;
//         (eval, best_move, nodes) = alpha_beta_search(board, move_gen, pesto, depth, -1000000, 1000000);
//         n += nodes;
//         println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, n, eval, utils::print_move(&best_move));
//     }
//     (eval, best_move, n)
// }

// pub(crate) fn aspiration_window_ab_search(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, max_depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
//     // Perform aspiration window alpha-beta search from the given position
//     // Also uses iterative deepening
//     let lower_bound_param: i32 = -25;
//     let upper_bound_param: i32 = 25;
//
//     let mut target_eval: i32 = board.eval;
//     let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
//     let mut nodes: i32 = 0;
//
//     // First perform a quiescence search at a depth of 0
//     let mut lower_bound: i32 = -1000000;
//     let mut upper_bound: i32 = 1000000;
//     let (mut eval, mut n) = q_search_consistent_side_to_move_for_final_eval(board, move_gen, pesto, lower_bound, upper_bound, true);
//
//     // Now perform an iterative deepening search with aspiration windows
//     for d in 1..max_depth / 2 + 1 {
//         let depth = 2 * d;
//         let mut lower_window_scale: i32 = 1;
//         let mut upper_window_scale: i32 = 1;
//         loop {
//             lower_bound = target_eval + lower_bound_param * lower_window_scale;
//             upper_bound = target_eval + upper_bound_param * upper_window_scale;
//             println!("Aspiration window search with window {} {}", lower_bound, upper_bound);
//             (eval, best_move, nodes) = alpha_beta_search(board, move_gen, pesto, depth, lower_bound, upper_bound);
//             n += nodes;
//             println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, n, eval, utils::print_move(&best_move));
//             if eval == lower_bound {
//                 println!("\nLower bound hit; retrying with larger window");
//                 lower_window_scale *= 2;
//             } else if eval == upper_bound {
//                 println!("\nUpper bound hit; retrying with larger window");
//                 upper_window_scale *= 2;
//             } else {
//                 println!("\nAspiration window search successful!");
//                 println!("Best move: {}", utils::print_move(&best_move));
//                 println!("Eval: {}\n", eval);
//                 target_eval = eval;
//                 break;
//             }
//         }
//     }
//     (eval, best_move, n)
// }

// Quiescence search with consistent side to move
// The idea is to compare apples to apples by only comparing evals of positions where the same side is to move
fn q_search_consistent_side_to_move_for_final_eval(board: &mut Bitboard, move_gen: &MoveGen, pesto: &PestoEval, mut alpha: i32, beta: i32, eval_after_even_moves: bool) -> (i32, i32) {
    let (checkmate, stalemate) = board.is_checkmate_or_stalemate(move_gen);
    if checkmate {
        println!("Qsearch: Quiescence: Checkmate!");
        return (-1000000, 1);
    } else if stalemate {
        println!("Quiescence: Stalemate!");
        return (0, 1);
    }
    // board.print();
    if eval_after_even_moves {
        // The problem here is that we are currently only comparing the eval at the end of the tactics, but
        // sometimes the player to move might not want to play a capture, so we need to consider the stand pat eval too
        // This side can either play a capture, or evaluate the position, whichever is better
        let eval = -pesto.eval(board);
        let captures = move_gen.gen_pseudo_legal_captures(board);
        if captures.len() == 0 {
            // println!("Quiescence: No captures left! Eval: {}", eval);
            (eval, 1)
        } else {
            // println!("Stand pat eval: {}", eval);
            if eval > alpha {
                if eval >= beta {
                    // println!("Quiescence: Stand pat eval is better than beta! Eval: {}", eval);
                    return (beta, 1); // Return beta here because player to move has reached a position that is better than beta, so opponent will never play this position
                }
                alpha = eval;
            }
            let mut nodes: i32 = 1;
            for c in captures {
                let mut new_board = board.make_move(c.0, c.1, c.2);
                if !new_board.is_legal(move_gen) {
                    continue;
                }
                let (mut score, nn) = q_search_consistent_side_to_move_for_final_eval(&mut new_board, move_gen, pesto, -beta, -alpha, !eval_after_even_moves);
                score = -score;
                println!("Capture eval: {}", score);
                nodes += nn;
                if score > alpha {
                    alpha = score;
                    if score >= beta {
                        return (beta, nodes);
                    }
                }
            }
            (alpha, nodes)
        }
    } else {
        // Other side simply plays best move
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(board, &pesto);
        let mut nodes: i32 = 1;
        captures.extend(moves);
        for c in captures {
            let mut new_board = board.make_move(c.0, c.1, c.2);
            if !new_board.is_legal(move_gen) {
                continue;
            }
            let (mut score, nn) = q_search_consistent_side_to_move_for_final_eval(&mut new_board, move_gen, pesto, -beta, -alpha, !eval_after_even_moves);
            score = -score;
            println!("Other side eval: {}", score);
            nodes += nn;
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        (alpha, nodes)
    }
}

// Search for forced wins by performing an iteratively deepening search, where the side to move always gives check
// Finds checkmates, and will in the future also find won endgames in the tablebases when we implement them
// Does not find forced checkmates in which the side to move does not give check
// Does not find forced stalemates or threefold repetitions, since it won't always be clear a priori whether playing for a draw is the best option
// The plan is to perform a mate search at the beginning of every move, followed by a standard quiescence search
// Actually eventually we will want every move in the search tree to be a mate search, but for now we will just do it at the beginning of every move
// Returns the eval and number of nodes searched
// Eval can be 1000000 for checkmate, -1000000 for checkmate for the other side, or 0 for neither
// This is an iterative deepening, alpha-beta search, where alpha-beta is just used for the side to move to avoid getting mated,
// and the iterative deepening stops as soon as a forced mate is found
// That way, we first find mate in 1, then mate in 2, etc
pub(crate) fn mate_search(board: &mut Bitboard, move_gen: &MoveGen, max_depth: i32) -> (i32, (usize, usize, Option<usize>), i32) {
    let mut eval: i32 = 0;
    let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
    let mut n: i32 = 0;
    let mut alpha = -1000000;
    let beta = 1000000;
    for d in 1..max_depth * 2 {
        println!("Performing mate search at depth {}", d);
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(board);
        captures.extend(moves);
        for m in captures {
            let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
            if !new_board.is_legal(move_gen) {
                continue;
            }
            if !new_board.is_check(move_gen) {
                continue;
            }
            println!("Considering move {}", utils::print_move(&m));
            let (score, nodes) = mate_search_recursive(&mut new_board, move_gen, d - 1, -beta, -alpha, false);
            eval = -score;
            println!("Results of mate search recursive: {} {}", eval, nodes);
            n += nodes;
            if eval > alpha {
                alpha = eval;
            }
            if alpha >= beta {
                best_move = m;
                break;
            }
        }
        println!("At depth {}, searched {} nodes. best eval {}", d, n, eval);
        // If checkmate found, stop searching
        if eval == 1000000 {
            println!("Mate search: Checkmate! No need to go deeper");
            break;
        }
    }
    (eval, best_move, n)
}


fn mate_search_recursive(board: &mut Bitboard, move_gen: &MoveGen, depth: i32, mut alpha: i32, beta: i32, side_to_move: bool) -> (i32, i32) {
    // Private recursive function used for mate search
    // External functions should call mate_search instead
    // Returns the eval (in centipawns) of the final position
    // Also returns number of nodes searched
    println!("Mate search at depth {} with alpha {} and beta {}", depth, alpha, beta);
    if depth == 0 {
        // Leaf node
        // Check whether this is checkmate (could be either side)
        let (checkmate, stalemate) = board.is_checkmate_or_stalemate(move_gen);
        if checkmate {
            println!("Mate search: Checkmate!");
            return (-1000000, 1);
        } else if stalemate {
            println!("Mate search: Stalemate! (This should never happen!)");
            return (0, 1);
        } else {
            return (0, 1);
        }
    }
    // Non-leaf node
    let mut n: i32 = 1;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(board);
    captures.extend(moves);
    for m in captures {
        let mut new_board: Bitboard = board.make_move(m.0, m.1, m.2);
        if !new_board.is_legal(move_gen) {
            continue;
        }
        if side_to_move && !new_board.is_check(move_gen) {
            continue;
        }
        println!("Considering move {}", utils::print_move(&m));
        let (mut eval, nodes) = mate_search_recursive(&mut new_board, move_gen, depth - 1, -beta, -alpha, !side_to_move);
        eval = -eval;
        n += nodes;
        if eval > alpha {
            alpha = eval;
        }
        if alpha >= beta {
            break;
        }
    }
    (alpha, n)
}

// // Quiescence search
// pub(crate) fn q_search(board: &Bitboard, move_gen: &MoveGen, pesto: &PestoEval, mut alpha: i32, mut beta: i32) -> (i32, i32) {
//     // board.print();
//     // println!("eval: {}", eval);
//     let mut nodes: i32 = 1;
//     let mut captures = move_gen.gen_pseudo_legal_captures(board);
//     for c in captures {
//         let mut new_board = board.make_move(c.0, c.1, c.2);
//         if !new_board.is_legal(move_gen) {
//             continue;
//         }
//         let (mut score, nn) = q_search(&new_board, move_gen, pesto, -beta, -alpha);
//         score = -score;
//         nodes += nn;
//         if score > alpha {
//             alpha = score;
//         }
//         if alpha >= beta {
//             break;
//         }
//     }
//     // println!("alpha, beta: {} {}", alpha, beta);
//     (alpha, nodes)
// }