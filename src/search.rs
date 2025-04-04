/c/! Alpha-beta negamax search module
//!
//! This module implements the negamax search algorithm for chess position evaluation.

use std::cmp::max;
use std::time::{Duration, Instant};
use crate::boardstack::BoardStack;
use crate::move_types::Move;
use crate::move_generation::MoveGen;
use crate::eval::PestoEval;
use crate::utils::print_move;
use crate::transposition::TranspositionTable;

/// Perform negamax search from the given position
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `depth` - The depth to search to
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation (in centipawns) of the best move
/// * The best move to play from the current position
/// * The number of nodes searched
pub fn negamax_search(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, Move, i32) {
    let mut best_eval: i32 = -1000000;
    let mut best_move: Move = Move::null();
    let mut n: i32 = 0;
    
    // Generate and combine captures and regular moves
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto);
    captures.extend(moves);
    
    // Iterate through all moves
    for m in captures {
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        let (mut eval, nodes) = negamax(board, move_gen, pesto, depth - 1);
        eval = -eval;
        n += nodes;
        
        // Update best move if a better evaluation is found
        if eval > best_eval {
            best_eval = eval;
            best_move = m;
        }

        // Undo the move
        board.undo_move();
    }
    (best_eval, best_move, n)
}

/// Recursive helper function for negamax search
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `depth` - The current depth in the search tree
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation (in centipawns) of the best move
/// * The number of nodes searched
fn negamax(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, i32) {
    if depth == 0 {
        // Leaf node: return the board evaluation
        return (pesto.eval(&board.current_state(), move_gen), 1);
    }
    
    let mut best_eval: i32 = -1000000;
    let mut n: i32 = 0;
    
    // Generate and combine captures and regular moves
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto);
    captures.extend(moves);
    
    // Iterate through all moves
    for m in captures {
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        let (mut eval, nodes) = negamax(board, move_gen, pesto, depth - 1);
        eval = -eval;
        n += nodes;
        
        // Update best evaluation
        best_eval = best_eval.max(eval);

        // Undo the move
        board.undo_move();
    }
    
    (best_eval, n)
}

/// Perform alpha-beta search from the given position
///
/// This function performs an exhaustive search to the given depth, using alpha-beta pruning
/// to optimize the search process.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `depth` - The depth to search to
/// * `alpha_init` - The initial alpha value for alpha-beta pruning
/// * `beta_init` - The initial beta value for alpha-beta pruning
/// * `q_search_max_depth` - The maximum depth for the quiescence search
/// * `verbose` - A flag indicating whether to print verbose output
/// * `start_time` - Current time if time limit is enabled
/// * `time_limit` - Time limit for the search if time limit is enabled
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation (in centipawns) of the final position
/// * The best move to play from the current position
/// * The number of nodes searched
/// * Whether the search was terminated
pub fn alpha_beta_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    tt: &mut TranspositionTable,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    q_search_max_depth: i32,
    verbose: bool,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
) -> (i32, Move, i32, bool) {

    let mut nodes = 1;
    let mut best_move = Move::null();
    let is_pv_node = beta - alpha > 1;

    // Transposition table lookup: If this position has already been searched at the target depth
    // then return the transposition table entry.
    // Note: this also grabs the old answer even if the old answer was for a greater depth,
    // and due to the fact that the value of having the move is not taken into account,
    // this can lead to an eval that differs from similar positions by the value of one tempo
    // (in either direction).
    if let Some(entry) = tt.probe(board.current_state(), depth) {
        return (entry.score, entry.best_move, entry.depth, false);
    }

    // Check for time limit
    if let Some(time_limit) = time_limit {
        let elapsed_time = start_time.unwrap().elapsed();
        if elapsed_time >= time_limit {
            return (beta, best_move, nodes, true); // Time limit exceeded
        }
    }

    // Quiescence search
    if depth <= 0 {
        let (eval, nodes) = quiescence_search(board, move_gen, pesto, alpha, beta, q_search_max_depth, verbose);
        return (eval, Move::null(), nodes, false);
    }

    // Null move pruning
    // TODO: Add `can_null` parameter or logic to control when NMP is allowed (e.g., not in zugzwang positions)
    // TODO: Verify `board.is_check()` is efficient or cache the check status
    let can_null = true; // Placeholder: NMP is generally safe except in potential zugzwang
    if can_null && depth >= 3 && !board.is_check(move_gen) { // Assuming `is_check` checks the current side
        board.make_null_move();
        // Note: The reduction factor (R=3 here) is standard, but can be tuned.
        // Search with a null window (-beta, -beta + 1)
        let (score, _, child_nodes, terminated) = alpha_beta_search(board, move_gen, pesto, tt, depth - 1 - 3, -beta, -beta + 1, q_search_max_depth, false, start_time, time_limit);
        board.undo_null_move();
        nodes += child_nodes;

        if terminated { // Check if the sub-search was terminated due to time limit
            return (alpha, best_move, nodes, true); // Propagate termination
        }

        if -score >= beta {
            // Null move cutoff: The opponent has a move that forces a beta cutoff even after we pass the turn.
            // Store beta cutoff in TT? Some engines do, some don't for NMP cutoffs.
            return (beta, Move::null(), nodes, false); // Return beta, indicating a cutoff
        }
    }

    let mut moves_searched = 0;

    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto);
    captures.extend(moves);

    // Improve alpha-beta pruning by searching the best move from the transposition table first
    // The move must have a depth of at least 1
    if let Some(entry) = tt.probe(board.current_state(), 1) {
        if let Some(tt_best_move) = captures.iter().find(|&m| *m == entry.best_move) {
            // Move the transposition table's best move to the front
            let index = captures.iter().position(|m| *m == *tt_best_move).unwrap();
            let best_move = captures.remove(index);
            captures.insert(0, best_move);
        }
    }

    for mov in captures {
        // Increment the number of moves searched
        moves_searched += 1;

        // Make the move
        board.make_move(mov);

        // If the move is illegal, undo the move and continue
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }

        let mut score;

        // Late Move Reduction
        let FULL_DEPTH_MOVES = 2;
        let REDUCTION_LIMIT = 2;
        let REDUCTION_AMOUNT = 2;
        if moves_searched > FULL_DEPTH_MOVES
           && depth >= REDUCTION_LIMIT + REDUCTION_AMOUNT + 1
           && !is_pv_node
        {
            // Reduced depth search
            let (reduced_score, _, child_nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, depth - REDUCTION_AMOUNT - 1,
                -beta, -alpha, q_search_max_depth, verbose, start_time, time_limit
            );
            score = -reduced_score;
            nodes += child_nodes;

            if terminated {
                board.undo_move();
                return (alpha, best_move, nodes, true);
            }

            // Re-search at full depth if the reduced search was promising
            if score > alpha {
                let (full_score, _, child_nodes, terminated) = alpha_beta_search(
                    board, move_gen, pesto, tt, depth - 1,
                    -beta, -alpha, q_search_max_depth, verbose, start_time, time_limit
                );
                score = -full_score;
                nodes += child_nodes;

                if terminated {
                    board.undo_move();
                    return (alpha, best_move, nodes, true);
                }
            }
        } else {
            // Full depth search
            let (child_score, _, child_nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, depth - 1,
                -beta, -alpha, q_search_max_depth, verbose, start_time, time_limit
            );
            score = -child_score;
            nodes += child_nodes;

            if terminated {
                board.undo_move();
                return (alpha, best_move, nodes, true);
            }
        }

        board.undo_move();

        if score > alpha {
            alpha = score;
            best_move = mov;

            if alpha >= beta {
                // Store in transposition table
                tt.store(&board.current_state(), depth, beta, best_move);
                return (beta, best_move, nodes, false);
            }
        }
    }

    // Store in transposition table
    tt.store(&board.current_state(), depth, alpha, best_move);

    (alpha, best_move, nodes, false)
}

/// Perform iterative deepening alpha-beta search from the given position
///
/// This function performs an iterative deepening search, where the search depth is gradually increased
/// until the maximum depth is reached. At each iteration, the alpha-beta search algorithm is used to
/// search for the best move.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `max_depth` - The maximum depth to search to
/// * `q_search_max_depth` - The maximum depth for the quiescence search
/// * `time_limit` - An optional duration for the search time limit
/// * `verbose` - A flag indicating whether to print verbose output
///
/// # Returns
///
/// A tuple containing:
/// * The depth at which the search was stopped
/// * The evaluation (in centipawns) of the final position
/// * The best move to play from the current position
/// * The number of nodes searched
pub fn iterative_deepening_ab_search(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, tt: &mut TranspositionTable, max_depth: i32, q_search_max_depth: i32, time_limit: Option<Duration>, verbose: bool) -> (i32, i32, Move, i32) {

    let mut eval: i32 = 0;
    let mut best_move: Move = Move::null();
    let mut nodes: i32 = 0;
    let mut last_fully_searched_depth: i32 = 0;

    let start_time = Instant::now();

    // Check the transposition table to see if this node has already been searched at the target depth
    if let Some(entry) = tt.probe(&board.current_state(), max_depth) {
        return (entry.depth, entry.score, entry.best_move, nodes);
    }

    // Iterate over increasing depths
    let mut depth = 1;
    while depth <= max_depth {

        if verbose {
            println!("Starting search at depth {}", depth);
        }
        // Skip odd depths in iterative deepening (other than max_depth if it is odd)
        if depth < max_depth && depth % 2 == 1 {
            depth += 1;
            continue;
        }

        // Perform alpha-beta search
        let (new_eval, new_best_move, new_nodes, terminated) = alpha_beta_search(board, move_gen, pesto, &mut tt, depth, -1000000, 1000000, q_search_max_depth, verbose, Some(start_time), time_limit);

        if !terminated {
            eval = new_eval;
            best_move = new_best_move;
            nodes += new_nodes;
        }

        if verbose {
            println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, nodes, eval, print_move(&best_move));
        }

        // If there is a time limit, check to see if we have exceeded it
        if terminated {
            break;
        }

        if let Some(time_limit) = time_limit {
            if start_time.elapsed() > time_limit {
                if verbose {
                    println!("Time limit reached. Stopping search.");
                }
                break;
            }
        }

        // Store the result in the transposition table
        tt.store(&board.current_state(), depth, eval, best_move);
        last_fully_searched_depth = depth;

        depth += 1;
    }
    (last_fully_searched_depth, eval, best_move, nodes)
}

/// Perform aspiration window alpha-beta search from the given position
///
/// This function performs an aspiration window search, where the search is focused on a specific
/// window of possible scores. The window is initially set to a narrow range, and if the search
/// finds a move that falls outside this range, the window is expanded and the search is repeated.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `max_depth` - The maximum depth to search to
/// * `q_search_max_depth` - The maximum depth for the quiescence search
/// * `verbose` - A flag indicating whether to print verbose output
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation (in centipawns) of the final position
/// * The best move to play from the current position
/// * The number of nodes searched
pub fn aspiration_window_ab_search(board: &mut BoardStack, move_gen: &MoveGen, tt: &mut TranspositionTable, pesto: &PestoEval, max_depth: i32, q_search_max_depth: i32, verbose: bool) -> (i32, Move, i32) {
    // Perform aspiration window alpha-beta search from the given position
    // Also uses iterative deepening: After searching at a given depth, starts a new search at that depth + 1, but looks at most promising variation first
    // This is really helpful for alpha-beta pruning
    let lower_bound_param: i32 = -25;
    let upper_bound_param: i32 = 25;

    let mut target_eval: i32 = board.current_state().eval;
    let mut best_move: Move = Move::null();
    let mut nodes: i32;

    // First perform a quiescence search at a depth of 0
    let mut lower_bound: i32 = -1000000;
    let mut upper_bound: i32 = 1000000;
    let (mut eval, mut n) = quiescence_search(board, move_gen, pesto, lower_bound, upper_bound, q_search_max_depth, verbose);

    // Now perform an iterative deepening search with aspiration windows
    for d in 1..= max_depth {
        let depth = 2 * d; // Only even depths, due to the even/odd effect
        let mut lower_window_scale: i32 = 1;
        let mut upper_window_scale: i32 = 1;
        loop {
            lower_bound = target_eval + lower_bound_param * lower_window_scale;
            upper_bound = target_eval + upper_bound_param * upper_window_scale;
            if verbose {
                println!("Aspiration window search with window {} {}", lower_bound, upper_bound);
            }
            (eval, best_move, nodes, _) = alpha_beta_search(board, move_gen, pesto, tt, depth, lower_bound, upper_bound, q_search_max_depth, verbose, None, None);
            n += nodes;
            if verbose {
                println!("At depth {}, searched {} nodes. best eval and move are {} {}", depth, n, eval, print_move(&best_move));
            }
            if eval == lower_bound {
                if verbose {
                    println!("\nLower bound hit; retrying with larger window");
                }
                lower_window_scale *= 2;
            } else if eval == upper_bound {
                if verbose {
                    println!("\nUpper bound hit; retrying with larger window");
                }
                upper_window_scale *= 2;
            } else {
                if verbose {
                    println!("\nAspiration window search successful!");
                    println!("Best move: {}", print_move(&best_move));
                    println!("Eval: {}\n", eval);
                }
                target_eval = eval;
                break;
            }
        }
    }
    (eval, best_move, n)
}

/// Performs a quiescence search to evaluate tactical sequences and avoid the horizon effect.
///
/// This function uses the negamax framework and searches captures and promotions until a quiet
/// position is reached or the maximum depth is hit. It implements stand-pat evaluation and
/// various pruning techniques to improve efficiency.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state.
/// * `move_gen` - A reference to the move generator.
/// * `pesto` - A reference to the position evaluator.
/// * `alpha` - The lower bound of the search window.
/// * `beta` - The upper bound of the search window.
/// * `max_depth` - The (remaining) maximum depth for quiescence search.
/// * `verbose` - A boolean flag for verbose output.
///
/// # Returns
///
/// A tuple containing:
/// - The score of the position after quiescence search (from the perspective of the side to move).
/// - The number of nodes searched.
fn quiescence_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    mut alpha: i32,
    beta: i32,
    max_depth: i32,
    verbose: bool
) -> (i32, i32) {
    let mut nodes = 1;

    // Stand-pat evaluation
    let stand_pat = pesto.eval(&board.current_state(), move_gen);

    // Beta cutoff
    if stand_pat >= beta {
        return (beta, nodes);
    }

    // Update alpha
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Check if we've reached max depth
    if max_depth == 0 {
        if verbose {
            println!("Quiescence: Max depth reached! Eval: {}", stand_pat);
        }
        return (alpha, nodes);
    }

    // Generate captures and promotions
    let captures = move_gen.gen_pseudo_legal_captures(&board.current_state());

    if captures.is_empty() {
        if verbose {
            println!("Quiescence: No captures left! Eval: {}", stand_pat);
        }
        return (stand_pat, nodes);
    }

    // Search captures
    for capture in captures {
        board.make_move(capture);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }

        // Recursive call
        let (mut score, n) = quiescence_search(board, move_gen, pesto, -beta, -alpha, max_depth - 1, verbose);
        score = -score; // Negamax
        nodes += n;

        // Undo move
        board.undo_move();

        // Beta cutoff
        if score >= beta {
            return (beta, nodes);
        }

        // Update alpha
        if score > alpha {
            alpha = score;
        }
    }

    (alpha, nodes)
}

/// Perform a quiescence search with consistent side to move
///
/// This function performs a quiescence search, which is a selective search of tactically
/// active positions. It ensures that the evaluation is always from the perspective of the
/// same side to move, allowing for consistent comparisons.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `pesto` - A reference to the Pesto evaluation function
/// * `alpha` - The lower bound of the search window
/// * `beta` - The upper bound of the search window
/// * `eval_after_even_moves` - A flag indicating whether to evaluate after even or odd number of moves
/// * `verbose` - A flag indicating whether to print verbose output
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation (in centipawns) of the final position
/// * The number of nodes searched
///
/// # Notes
/// Interesting idea, but not used currently because it is too slow
fn q_search_consistent_side_to_move_for_final_eval(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, mut alpha: i32, beta: i32, eval_after_even_moves: bool, verbose: bool) -> (i32, i32) {
    let (checkmate, stalemate) = board.current_state().is_checkmate_or_stalemate(move_gen);
    if checkmate {
        if verbose {
            println!("Qsearch: Quiescence: Checkmate!");
        }
        return (-1000000, 1);
    } else if stalemate {
        if verbose {
            println!("Quiescence: Stalemate!");
        }
        return (0, 1);
    }
    // board.print();
    if eval_after_even_moves {
        // The problem here is that we are currently only comparing the eval at the end of the tactics, but
        // sometimes the player to move might not want to play a capture, so we need to consider the stand pat eval too
        // This side can either play a capture, or evaluate the position, whichever is better
        let eval = pesto.eval(&board.current_state(), move_gen);
        let captures = move_gen.gen_pseudo_legal_captures(&board.current_state());
        if captures.is_empty() {
            if verbose {
                println!("Quiescence: No captures left! Eval: {}", eval);
            }
            (eval, 1)
        } else {
            if verbose {
                println!("Stand pat eval: {}", eval);
            }
            if eval > alpha {
                if eval >= beta {
                    if verbose {
                        println!("Quiescence: Stand pat eval is better than beta! Eval: {}", eval);
                    }
                    return (beta, 1); // Return beta here because player to move has reached a position that is better than beta, so opponent will never play this position
                }
                alpha = eval;
            }
            let mut n: i32 = 1;
            for c in captures {
                board.make_move(c);
                if !board.current_state().is_legal(move_gen) {
                    board.undo_move();
                    continue;
                }
                let (mut score, nodes) = q_search_consistent_side_to_move_for_final_eval(board, move_gen, pesto, -beta, -alpha, !eval_after_even_moves, verbose);
                score = -score;
                if verbose {
                    println!("Capture eval: {}", score);
                }
                n += nodes;
                board.undo_move();
                if score > alpha {
                    alpha = score;
                    if score >= beta {
                        return (beta, n);
                    }
                }
            }
            (alpha, n)
        }
    } else {
        // Other side simply plays best move
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto);
        let mut n: i32 = 1;
        captures.extend(moves);
        for c in captures {
            board.make_move(c);
            if !board.current_state().is_legal(move_gen) {
                board.undo_move();
                continue;
            }
            let (mut score, nodes) = q_search_consistent_side_to_move_for_final_eval(board, move_gen, pesto, -beta, -alpha, !eval_after_even_moves, verbose);
            score = -score;
            if verbose {
                println!("Other side eval: {}", score);
            }
            n += nodes;
            if score > alpha {
                alpha = score;
            }
            board.undo_move();
            if alpha >= beta {
                break;
            }
        }
        (alpha, n)
    }
}

/// Perform a mate search from the given position
///
/// This function performs an iteratively deepening search for forced checkmates,
/// where the side to move always gives check. It finds checkmates but does not
/// find forced checkmates where the side to move does not give check, nor does
/// it find forced stalemates or threefold repetitions.
///
/// # Arguments
///
/// * `board` - A reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `max_depth` - The maximum depth to search to
/// * `verbose` - A flag indicating whether to print verbose output
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation: 1000000 for checkmate, -1000000 for checkmate against, or 0 for neither
/// * The best move to play from the current position
/// * The number of nodes searched
pub fn mate_search(board: &mut BoardStack, move_gen: &MoveGen, max_depth: i32, verbose: bool) -> (i32, Move, i32) {
    let mut eval: i32 = 0;
    let mut best_move: Move = Move::null();
    let mut n: i32 = 0;
    let mut alpha = -1000000;
    let beta = 1000000;

    // Iterative deepening loop
    for d in 1..=max_depth {
        let depth = 2 * d - 1; // Consider only odd depths, since we are only searching for forced mates
        if verbose {
            println!("Performing mate search at depth {} ply", depth);
        }

        // Generate and combine captures and regular moves
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&mut board.current_state());
        captures.extend(moves);

        // Iterate through all moves
        for m in captures {
            board.make_move(m);
            if !board.current_state().is_legal(move_gen) {
                board.undo_move();
                continue;
            }
            if !board.current_state().is_check(move_gen) {
                board.undo_move();
                continue;
            }
            let (score, nodes) = mate_search_recursive(board, move_gen, depth - 1, -beta, -alpha, false);
            eval = -score;
            n += nodes;
            if eval > alpha {
                alpha = eval;
            }
            board.undo_move();
            if alpha >= beta {
                best_move = m;
                break;
            }
        }
        if verbose{
            println!("At depth {} ply, searched {} nodes. best eval {}", depth, n, eval);
        }
        // If checkmate found, stop searching
        if eval == 1000000 {
            if verbose{
                println!("Mate search: Checkmate! No need to go deeper");
            }
            break;
        }
    }
    (eval, best_move, n)
}

/// Recursive helper function for mate search
///
/// This function performs a recursive mate search to the given depth, using alpha-beta pruning
/// to optimize the search process. It only considers moves that give check.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
/// * `move_gen` - A reference to the move generator
/// * `depth` - The current depth in the search tree
/// * `alpha` - The current alpha value for alpha-beta pruning
/// * `beta` - The current beta value for alpha-beta pruning
/// * `side_to_move` - A boolean indicating which side is to move (true for the initial side)
///
/// # Returns
///
/// A tuple containing:
/// * The evaluation: -1000000 for checkmate, 0 for no mate found
/// * The number of nodes searched
fn mate_search_recursive(board: &mut BoardStack, move_gen: &MoveGen, depth: i32, mut alpha: i32, beta: i32, side_to_move: bool) -> (i32, i32) {
    // Private recursive function used for mate search
    // External functions should call mate_search instead
    // Returns the eval (in centipawns) of the final position
    // Also returns number of nodes searched
    if depth == 0 {
        // Leaf node
        // Check whether this is checkmate (could be either side)
        let (checkmate, stalemate) = board.current_state().is_checkmate_or_stalemate(move_gen);
        if checkmate {
            return (-1000000, 1);
        } else if stalemate {
            panic!("Stalemate in mate search!");
        } else {
            return (0, 1);
        }
    }
    // Non-leaf node
    let mut n: i32 = 1;
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board.current_state());
    captures.extend(moves);
    for m in captures {
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        if side_to_move && !board.current_state().is_check(move_gen) {
            board.undo_move();
            continue;
        }
        let (mut eval, nodes) = mate_search_recursive(board, move_gen, depth - 1, -beta, -alpha, !side_to_move);
        eval = -eval;
        n += nodes;
        if eval > alpha {
            alpha = eval;
        }
        board.undo_move();
        if alpha >= beta {
            break;
        }
    }
    (alpha, n)
}