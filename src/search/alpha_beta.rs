use std::time::{Duration, Instant};
use crate::boardstack::BoardStack;
use crate::move_generation::MoveGen;
use crate::eval::PestoEval;
use crate::move_types::{Move, NULL_MOVE};
use crate::transposition::{TranspositionEntry, TranspositionTable};
use super::history::HistoryTable;
use super::history::MAX_PLY;
use super::quiescence::quiescence_search;

/// Perform alpha-beta search from the given position
///
/// This function performs an exhaustive search to the given depth, using alpha-beta pruning
/// to optimize the search process. Includes NMP, Killers, History Heuristic, LMR.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `tt` - Transposition Table
/// * `killers` - Killer move table
/// * `history` - History heuristic table
/// * `depth` - Remaining depth to search
/// * `alpha` - Alpha bound
/// * `beta` - Beta bound
/// * `q_search_max_depth` - Max depth for quiescence search
/// * `verbose` - Verbosity flag
/// * `start_time` - Start time for time limit checks
/// * `time_limit` - Optional time limit duration
///
/// # Returns
///
/// A tuple containing: (score, best_move, nodes_searched, terminated_early)
pub fn alpha_beta_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    tt: &mut TranspositionTable,
    killers: &mut [[Move; 2]; MAX_PLY],
    history: &mut HistoryTable,
    depth: i32,
    alpha_init: i32,
    beta_init: i32,
    q_search_max_depth: i32,
    verbose: bool,
    start_time: Option<Instant>,
    time_limit: Option<Duration>
) -> (i32, Move, i32, bool) {
    let mut best_move: Move = NULL_MOVE;
    let mut alpha: i32 = alpha_init;
    let beta: i32 = beta_init;
    let mut n: i32 = 0;
    let mut eval: i32 = 0;

    // Check for checkmate and stalemate
    if verbose {
        println!("Checking for checkmate and stalemate");
    }
    let (checkmate, stalemate) = board.current_state().is_checkmate_or_stalemate(move_gen);
    if verbose {
        println!("Checkmate and stalemate checked");
        println!("Checkmate: {} Stalemate: {}", checkmate, stalemate);
    }

    // Handle checkmate and stalemate cases
    if checkmate {
        if verbose {
            println!("AB search: Checkmate!");
        }
        return (1000000, best_move, 1, true);
    } else if stalemate {
        if verbose {
            println!("AB search: Stalemate!");
        }
        return (0, best_move, 1, true);
    }

    // Generate and combine captures and regular moves
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto, Some(history));
    captures.extend(moves);

    // Print the list of captures
    if verbose {
        println!("Generated {} moves", captures.len());
        for m in &captures {
            println!("Move: {}", m);
        }
    }

    for m in captures {
        if verbose {
            println!("Considering move {} at root of search tree", m);
        }
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        let (search_eval, nodes) = alpha_beta_recursive(
            board,
            move_gen,
            pesto,
            tt,
            killers,
            history,
            depth - 1,
            -beta,
            -alpha,
            q_search_max_depth,
            verbose
        );
        eval = -search_eval;
        n += nodes;
        if eval > alpha {
            alpha = eval;
            best_move = m;
        }

        if verbose {
            println!("Just checked move {}, current best move is {}", m, best_move);
            if let Some(start_time) = start_time {
                println!("Current time: {:?}, time limit: {:?}", start_time.elapsed(), time_limit);
            }
        }

        // Check time limit
        if let Some(start_time) = start_time {
            if let Some(time_limit) = time_limit {
                if start_time.elapsed() > time_limit {
                    if verbose {
                        println!("Time limit reached. Stopping search.");
                    }
                    return (alpha, best_move, nodes, true);
                }
            }
        }

        // Undo the move
        board.undo_move();

        // Prune if necessary
        if alpha >= beta {
            // Update history table for the cutoff move
            if !move_gen.is_capture(&board.current_state(), &best_move) {
                history.update(&best_move, depth);
            }
            break;
        }
    }

    if verbose {
        println!("Alpha beta search at depth {} searched {} nodes. Best eval and move are {} {}", depth, n, alpha, best_move);
    }

    // Store the result in the transposition table
    tt.store(board.current_state(), depth, eval, best_move);

    (alpha, best_move, n, false)
}

/// Recursive helper function for alpha-beta search
fn alpha_beta_recursive(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    tt: &mut TranspositionTable,
    killers: &mut [[Move; 2]; MAX_PLY],
    history: &mut HistoryTable,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    q_search_max_depth: i32,
    verbose: bool
) -> (i32, i32) {
    if depth <= 0 {
        return quiescence_search(board, move_gen, pesto, alpha, beta, q_search_max_depth, verbose);
    }

    let mut best_eval: i32 = -1000000;
    let mut n: i32 = 0;

    // Generate and combine captures and regular moves
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto, Some(history));
    captures.extend(moves);

    // Iterate through all moves
    for m in captures {
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        let (mut eval, nodes) = alpha_beta_recursive(board, move_gen, pesto, tt, killers, history, depth - 1, -beta, -alpha, q_search_max_depth, verbose);
        eval = -eval;
        n += nodes;

        // Update best evaluation
        best_eval = best_eval.max(eval);
        if eval > alpha {
            alpha = eval;
            if alpha >= beta {
                // Update history table for the cutoff move
                if !move_gen.is_capture(&board.current_state(), &m) {
                    history.update(&m, depth);
                }
                board.undo_move();
                return (beta, n);
            }
        }

        // Undo the move
        board.undo_move();
    }

    (best_eval, n)
} 