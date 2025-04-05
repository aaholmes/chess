use super::alpha_beta::alpha_beta_search;
use super::history::HistoryTable;
use super::history::MAX_PLY;
use crate::boardstack::BoardStack;
use crate::eval::PestoEval;
use crate::move_generation::MoveGen;
use crate::move_types::{Move, NULL_MOVE};
use crate::transposition::TranspositionTable;
use crate::utils::print_move;
use std::time::{Duration, Instant};

/// Perform iterative deepening alpha-beta search from the given position
pub fn iterative_deepening_ab_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    tt: &mut TranspositionTable,
    max_depth: i32,
    q_search_max_depth: i32,
    time_limit: Option<Duration>,
    verbose: bool,
) -> (i32, i32, Move, i32) {
    let mut eval: i32 = 0;
    let mut best_move: Move = NULL_MOVE;
    let mut nodes: i32 = 0;
    let mut last_fully_searched_depth: i32 = 0;

    let start_time = Instant::now();
    let mut killers = [[NULL_MOVE; 2]; MAX_PLY];
    let mut history = HistoryTable::new();

    // Iterate over increasing depths
    for depth in 1..=max_depth {
        if verbose {
            println!("info depth {}", depth);
        }

        // Perform alpha-beta search for the current depth
        let (new_eval, new_best_move, new_nodes, terminated) = alpha_beta_search(
            board,
            move_gen,
            pesto,
            tt,
            &mut killers,
            &mut history,
            depth,
            -1000000,
            1000000,
            q_search_max_depth,
            false,
            Some(start_time),
            time_limit,
        );

        nodes += new_nodes;

        if terminated {
            if verbose {
                println!("info string Search terminated at depth {}", depth);
            }
            break;
        }

        eval = new_eval;
        if new_best_move != NULL_MOVE {
            best_move = new_best_move;
        }
        last_fully_searched_depth = depth;

        if verbose {
            let elapsed = start_time.elapsed().as_millis();
            let nps = if elapsed > 0 {
                (nodes as u128 * 1000) / elapsed
            } else {
                0
            };
            println!(
                "info depth {} score cp {} time {} nodes {} nps {} pv {}",
                depth,
                eval,
                elapsed,
                nodes,
                nps,
                print_move(&best_move)
            );
        }

        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                if verbose {
                    println!("info string Time limit reached after depth {}", depth);
                }
                break;
            }
        }

        if eval.abs() > 900000 {
            if verbose {
                println!("info string Mate found at depth {}", depth);
            }
            break;
        }
    }

    (last_fully_searched_depth, eval, best_move, nodes)
}

/// Perform aspiration window alpha-beta search from the given position
pub fn aspiration_window_ab_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    tt: &mut TranspositionTable,
    pesto: &PestoEval,
    max_depth: i32,
    q_search_max_depth: i32,
    verbose: bool,
) -> (i32, Move, i32) {
    let initial_delta = 25;
    let mut delta = initial_delta;

    let mut eval: i32 = 0;
    let mut best_move: Move = NULL_MOVE;
    let mut total_nodes: i32 = 0;

    let mut killers = [[NULL_MOVE; 2]; MAX_PLY];
    let mut history = HistoryTable::new();

    for depth in 1..=max_depth {
        let mut alpha = eval - delta;
        let mut beta = eval + delta;

        loop {
            if verbose {
                println!(
                    "info depth {} searching window [{}, {}]",
                    depth, alpha, beta
                );
            }

            let (current_eval, current_best_move, nodes, terminated) = alpha_beta_search(
                board,
                move_gen,
                pesto,
                tt,
                &mut killers,
                &mut history,
                depth,
                alpha,
                beta,
                q_search_max_depth,
                false,
                None,
                None,
            );
            total_nodes += nodes;

            if terminated {
                println!("Warning: Search terminated unexpectedly within aspiration window");
                return (eval, best_move, total_nodes);
            }

            eval = current_eval;
            if current_best_move != NULL_MOVE {
                best_move = current_best_move;
            }

            if eval <= alpha {
                if verbose {
                    println!(
                        "info string Fail low at depth {}, widening window [{}, {}]",
                        depth,
                        -1000000,
                        eval + 1
                    );
                }
                alpha = -1000000;
                beta = eval + 1;
                delta += delta / 2;
            } else if eval >= beta {
                if verbose {
                    println!(
                        "info string Fail high at depth {}, widening window [{}, {}]",
                        depth,
                        eval - 1,
                        1000000
                    );
                }
                alpha = eval - 1;
                beta = 1000000;
                delta += delta / 2;
            } else {
                if verbose {
                    println!(
                        "info depth {} score cp {} nodes {} pv {}",
                        depth,
                        eval,
                        total_nodes,
                        print_move(&best_move)
                    );
                }
                delta = initial_delta + delta / 4;
                break;
            }
        }

        if eval.abs() > 900000 {
            break;
        }
    }

    (eval, best_move, total_nodes)
}
