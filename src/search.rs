/// Alpha-beta negamax search module
///
/// This module implements the negamax search algorithm for chess position evaluation.

use std::cmp::max;
use std::time::{Duration, Instant};
use crate::boardstack::BoardStack;
use crate::move_types::{Move, NULL_MOVE};
use crate::move_generation::MoveGen;
use crate::eval::PestoEval;
use crate::utils::print_move;
use crate::transposition::{TranspositionEntry, TranspositionTable};
use crate::piece_types::{PieceType, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, NO_PIECE_TYPE, WHITE, BLACK}; // Added for SEE
use crate::board::Board; // Added for SEE
use crate::board_utils::sq_ind_to_bit; // Added for SEE

const MAX_PLY: usize = 64; // Max search depth for killer/history table size

// Piece values for SEE (simple centipawn values)
// Order: P, N, B, R, Q, K (index 6 for NO_PIECE_TYPE is 0)
const SEE_PIECE_VALUES: [i32; 7] = [100, 320, 330, 500, 975, 10000, 0];

/// Perform negamax search from the given position (Simple version without AB pruning)
/// Kept for reference or simple testing.
pub fn negamax_search(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, depth: i32) -> (i32, Move, i32) {
    let mut best_eval: i32 = -1000000;
    let mut best_move: Move = Move::null(); // Use Move::null() if available, otherwise define NULL_MOVE
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

/// Recursive helper function for simple negamax search
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
/// * `ply` - Current search depth from root (0-based)
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
    history: &mut [[i32; 64]; 64], // History table [from_sq][to_sq]
    ply: usize,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    q_search_max_depth: i32,
    verbose: bool,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
) -> (i32, Move, i32, bool) {

    let mut nodes = 1;
    let mut best_move = NULL_MOVE; // Use constant
    let is_pv_node = beta - alpha > 1;
    let original_alpha = alpha; // Needed for TT storage type

    // --- Transposition Table Lookup ---
    if let Some(entry) = tt.probe(board.current_state(), depth) {
        // TODO: Add bound checks (if entry.score >= beta and entry is lower bound, etc.)
        // Basic check: if depth is sufficient, use the stored score and move
        if entry.depth >= depth {
             // Add more sophisticated TT hit logic (exact, lower, upper bound)
             // For now, just return if depth matches or exceeds
             return (entry.score, entry.best_move, 1, false);
        }
        // Can still use the move for move ordering even if depth is insufficient
        best_move = entry.best_move; // Use TT move as initial best guess
    }

    // --- Time Limit Check ---
    if ply > 0 && time_limit.is_some() { // Avoid checking at root? Or check less frequently
        // Check roughly every 2048 nodes for performance
        // Note: Node count isn't accurate here yet, check based on time directly
        if (nodes & 2047) == 0 && start_time.unwrap().elapsed() >= time_limit.unwrap() {
            return (alpha, best_move, nodes, true); // Indicate termination
        }
    }

    // --- Base Case: Quiescence Search ---
    if depth <= 0 {
        let (eval, q_nodes) = quiescence_search(board, move_gen, pesto, alpha, beta, q_search_max_depth, verbose);
        nodes += q_nodes;
        // Store result of qsearch? Typically not, as it's unstable.
        return (eval, NULL_MOVE, nodes, false);
    }

    // --- Null Move Pruning (NMP) ---
    let in_check = board.is_check(move_gen);
    let can_do_nmp = !in_check && ply > 0; // Don't do NMP at root or when in check

    if can_do_nmp && depth >= 3 { // Standard depth requirement for NMP
        // More sophisticated Zugzwang check (e.g., only pawns/king left)
        let is_zugzwang_candidate = {
            let current_board = board.current_state();
            let color = if current_board.w_to_move { WHITE } else { BLACK };
            let non_pawn_king_material = current_board.pieces_occ[color] & !current_board.pieces[color][PAWN] & !current_board.pieces[color][KING];
            non_pawn_king_material == 0
        };

        if !is_zugzwang_candidate {
            board.make_null_move();
            // Search with reduced depth (R=3 is common) and null window
            let (score, _, child_nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, killers, history, ply + 1,
                depth - 1 - 3, -beta, -beta + 1, // Null window search
                q_search_max_depth, false, start_time, time_limit
            );
            board.undo_null_move();
            nodes += child_nodes;

            if terminated { return (alpha, best_move, nodes, true); }

            // Null move cutoff
            if -score >= beta {
                // Store beta cutoff? Optional for NMP.
                // TT entry type would be Lower Bound here if storing.
                return (beta, NULL_MOVE, nodes, false); // Return beta, indicating cutoff
            }
        }
    }

    // --- Main Search Loop ---
    let mut moves_searched = 0;
    let mut legal_moves_found = 0;

    // --- Move Ordering ---
    // TODO: Ensure `gen_pseudo_legal_moves_ordered` exists and provides ordered captures (MVV-LVA) and separate quiet moves.
    let (mut captures, mut quiet_moves) = move_gen.gen_pseudo_legal_moves_ordered(&board.current_state(), pesto);

    // 1. Transposition Table Move (Use the one potentially found earlier)
    let tt_move = best_move; // best_move holds TT move if found, else NULL_MOVE
    if tt_move != NULL_MOVE {
       // Remove the TT move from captures or quiet moves if present, to avoid duplication
       if let Some(pos) = captures.iter().position(|&m| m == tt_move) {
           captures.remove(pos);
       } else if let Some(pos) = quiet_moves.iter().position(|&m| m == tt_move) {
           quiet_moves.remove(pos);
       }
       // If tt_move wasn't in generated moves, it might be illegal/outdated, keep best_move as NULL_MOVE?
       // For now, assume if TT had it, it's worth trying first.
    }


    // 2. Killer Moves (prioritize non-captures that caused cutoffs)
    let killer1 = if ply < MAX_PLY { killers[ply][0] } else { NULL_MOVE };
    let killer2 = if ply < MAX_PLY { killers[ply][1] } else { NULL_MOVE };

    // 3. History Heuristic Sort for Quiet Moves
    quiet_moves.sort_unstable_by_key(|m| -history[m.from()][m.to()]); // Sort descending

    // Build the ordered list for iteration
    let mut ordered_moves = Vec::with_capacity(captures.len() + quiet_moves.len() + 3);
    if tt_move != NULL_MOVE { ordered_moves.push(tt_move); }
    ordered_moves.extend(captures); // Assumes captures are already ordered (e.g., MVV-LVA)
    // Add killers if they are quiet moves and not the TT move
    if killer1 != NULL_MOVE && killer1 != tt_move && !move_gen.is_capture(&board.current_state(), killer1) {
         if let Some(pos) = quiet_moves.iter().position(|&m| m == killer1) {
             ordered_moves.push(quiet_moves.remove(pos));
         }
    }
     if killer2 != NULL_MOVE && killer2 != tt_move && killer1 != killer2 && !move_gen.is_capture(&board.current_state(), killer2) {
         if let Some(pos) = quiet_moves.iter().position(|&m| m == killer2) {
             ordered_moves.push(quiet_moves.remove(pos));
         }
    }
    ordered_moves.extend(quiet_moves); // Add remaining history-sorted quiet moves


    // --- Iterate Through Moves ---
    let mut current_best_score = -1000000; // Track best score found in this node

    for mov in ordered_moves {
        moves_searched += 1;

        board.make_move(mov);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }
        legal_moves_found += 1;

        let mut score;
        let search_depth = depth - 1;

        // --- Late Move Reduction (LMR) ---
        let mut reduced_depth = search_depth;
        let mut needs_re_search = false;
        // Conditions for LMR: depth >= 3, move is quiet, not in check, not PV node, move is not killer/TT move
        // Adjust LMR conditions and amount as needed
        if depth >= 3 && moves_searched > 2 && !is_pv_node && !in_check && !move_gen.is_capture(&board.current_state(), mov) && mov != killer1 && mov != killer2 && mov != tt_move {
             // Simple reduction based on depth and moves searched
             let reduction = if moves_searched > 6 { 2 } else { 1 }; // Example reduction
             reduced_depth = (search_depth - reduction).max(0);
             needs_re_search = true;
        }

        // --- Principal Variation Search (PVS) ---
        if legal_moves_found > 1 || !is_pv_node { // Zero-window search for non-PV moves
            let (pvs_score, _, child_nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, killers, history, ply + 1,
                reduced_depth, -(alpha + 1), -alpha, // Null window (alpha, alpha+1)
                q_search_max_depth, false, start_time, time_limit
            );
            score = -pvs_score;
            nodes += child_nodes;

            if terminated { board.undo_move(); return (alpha, best_move, nodes, true); }

            // If zero-window search failed high (score > alpha), re-search with full window
            // Only re-search if LMR was applied (needs_re_search) or if it's potentially a new PV move
            if score > alpha && (needs_re_search || is_pv_node) {
                 let (full_score, _, child_nodes, terminated) = alpha_beta_search(
                    board, move_gen, pesto, tt, killers, history, ply + 1,
                    search_depth, -beta, -alpha, // Full window re-search
                    q_search_max_depth, false, start_time, time_limit
                );
                score = -full_score;
                nodes += child_nodes; // Add nodes from re-search
                 if terminated { board.undo_move(); return (alpha, best_move, nodes, true); }
            }
        } else { // Full window search for the first move (PV move)
             let (full_score, _, child_nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, killers, history, ply + 1,
                search_depth, -beta, -alpha, // Full window
                q_search_max_depth, false, start_time, time_limit
            );
            score = -full_score;
            nodes += child_nodes;
             if terminated { board.undo_move(); return (alpha, best_move, nodes, true); }
        }


        board.undo_move();

        // --- Update Alpha and Best Move ---
        if score > current_best_score { // Keep track of the best score found so far in this node
             current_best_score = score;
             if score > alpha { // If it also improves alpha
                 alpha = score;
                 best_move = mov; // Update the best move for this node

                 // --- Beta Cutoff ---
                 if alpha >= beta {
                     // Store Killer Move if it's a quiet move and ply is valid
                     if ply < MAX_PLY && !move_gen.is_capture(&board.current_state(), mov) {
                         if mov != killers[ply][0] {
                            killers[ply][1] = killers[ply][0];
                            killers[ply][0] = mov;
                         }
                     }
                     // Update History Score for the move causing cutoff
                     if !move_gen.is_capture(&board.current_state(), mov) {
                         let bonus = (depth * depth).max(1);
                         let from_idx = mov.from();
                         let to_idx = mov.to();
                         if from_idx < 64 && to_idx < 64 {
                             history[from_idx][to_idx] = history[from_idx][to_idx].saturating_add(bonus);
                         }
                     }
                     // Store Beta cutoff in transposition table
                     tt.store(&board.current_state(), depth, beta, mov); // Store beta score and cutoff move
                     return (beta, mov, nodes, false); // Return beta score
                 }
             }
        }
    }

    // --- Checkmate/Stalemate Handling ---
    if legal_moves_found == 0 {
        if in_check {
            // Checkmate - score is losing, relative to ply
            alpha = -1000000 + ply as i32;
        } else {
            // Stalemate - score is 0
            alpha = 0;
        }
        best_move = NULL_MOVE; // No legal move
    }

    // --- Store Result in Transposition Table ---
    // Determine entry type based on score relative to original alpha
    let entry_type = if alpha >= beta { // Should have been caught by cutoff, but defensively...
        TranspositionEntry::LOWER_BOUND // Score is at least beta
    } else if alpha > original_alpha {
        TranspositionEntry::EXACT // Score is exactly alpha
    } else {
        TranspositionEntry::UPPER_BOUND // Score is at most alpha
    };
    tt.store_detailed(&board.current_state(), depth, alpha, best_move, entry_type);


    (alpha, best_move, nodes, false)
}


/// Perform iterative deepening alpha-beta search from the given position
pub fn iterative_deepening_ab_search(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, tt: &mut TranspositionTable, max_depth: i32, q_search_max_depth: i32, time_limit: Option<Duration>, verbose: bool) -> (i32, i32, Move, i32) {

    let mut eval: i32 = 0;
    let mut best_move: Move = NULL_MOVE;
    let mut nodes: i32 = 0;
    let mut last_fully_searched_depth: i32 = 0;

    let start_time = Instant::now();
    let mut killers = [[NULL_MOVE; 2]; MAX_PLY];
    let mut history = [[0i32; 64]; 64]; // Initialize history table per search

    // Check TT for existing high-depth search result first? Optional.
    // if let Some(entry) = tt.probe(&board.current_state(), max_depth) { ... }

    // Iterate over increasing depths
    for depth in 1..=max_depth {

        // Optional: Skip shallow odd depths if desired
        // if depth < max_depth && depth % 2 == 1 { continue; }

        if verbose { println!("info depth {}", depth); }

        // Perform alpha-beta search for the current depth
        let (new_eval, new_best_move, new_nodes, terminated) = alpha_beta_search(
            board,
            move_gen,
            pesto,
            tt,
            &mut killers,
            &mut history, // Pass history table
            0, // Starting ply
            depth,
            -1000000, // Initial full window
            1000000,
            q_search_max_depth,
            false, // Typically less verbose in inner loops
            Some(start_time),
            time_limit
        );

        nodes += new_nodes; // Accumulate nodes

        // If the search was terminated early (e.g., time limit), use the results from the *previous* completed depth
        if terminated {
            if verbose { println!("info string Search terminated at depth {}", depth); }
            break; // Exit iterative deepening loop
        }

        // Search completed for this depth, update results
        eval = new_eval;
        // Only update best_move if it's not null (can happen on mate/stalemate)
        if new_best_move != NULL_MOVE {
             best_move = new_best_move;
        }
        last_fully_searched_depth = depth;

        // Print UCI info (optional, but standard)
        if verbose {
             let elapsed = start_time.elapsed().as_millis();
             let nps = if elapsed > 0 { (nodes as u128 * 1000) / elapsed } else { 0 };
             // TODO: Extract and print Principal Variation (PV) line from TT
             println!("info depth {} score cp {} time {} nodes {} nps {} pv {}",
                      depth, eval, elapsed, nodes, nps, print_move(&best_move));
        }

        // Check time limit *after* completing a depth and storing results
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                if verbose { println!("info string Time limit reached after depth {}", depth); }
                break;
            }
        }

        // Optional: Check for mate score and stop early
        if eval.abs() > 900000 { // If mate is found (adjust threshold as needed)
             if verbose { println!("info string Mate found at depth {}", depth); }
             break;
        }
    }

    // Return results from the last *fully completed* depth
    (last_fully_searched_depth, eval, best_move, nodes)
}


/// Perform aspiration window alpha-beta search from the given position
/// (Combines iterative deepening with aspiration windows)
pub fn aspiration_window_ab_search(board: &mut BoardStack, move_gen: &MoveGen, tt: &mut TranspositionTable, pesto: &PestoEval, max_depth: i32, q_search_max_depth: i32, verbose: bool) -> (i32, Move, i32) {
    let initial_delta = 25; // Initial window size around the predicted score
    let mut delta = initial_delta;

    let mut eval: i32 = 0; // Use eval from previous depth as prediction
    let mut best_move: Move = NULL_MOVE;
    let mut total_nodes: i32 = 0;

    let mut killers = [[NULL_MOVE; 2]; MAX_PLY];
    let mut history = [[0i32; 64]; 64]; // Initialize history table per search

    // Get initial eval guess (e.g., from shallow search or static eval)
    // For iterative deepening, use the result of the previous depth

    for depth in 1..=max_depth {
        let mut alpha = eval - delta; // Set window around previous eval
        let mut beta = eval + delta;

        loop { // Loop for re-searches if bounds are hit
            if verbose { println!("info depth {} searching window [{}, {}]", depth, alpha, beta); }

            let (current_eval, current_best_move, nodes, terminated) = alpha_beta_search(
                board, move_gen, pesto, tt, &mut killers, &mut history, 0,
                depth, alpha, beta, q_search_max_depth, false, None, None // No time limit within aspiration loop usually
            );
            total_nodes += nodes;

            if terminated { // Should not happen without time limit, but handle defensively
                 println!("Warning: Search terminated unexpectedly within aspiration window");
                 return (eval, best_move, total_nodes); // Return previous best
            }

            // Update eval and best_move from the latest search *before* checking bounds
            eval = current_eval;
            if current_best_move != NULL_MOVE { // Avoid overwriting with null move from failed search
                 best_move = current_best_move;
            }


            if eval <= alpha { // Failed low, widen window downwards and re-search
                if verbose { println!("info string Fail low at depth {}, widening window [{}, {}]", depth, -1000000, eval + 1); }
                // Widen significantly downwards, keep upper bound tight for re-search
                alpha = -1000000;
                beta = eval + 1;
                delta += delta / 2; // Increase delta for next iteration's window width
                // No break, loop again with wider window
            } else if eval >= beta { // Failed high, widen window upwards and re-search
                 if verbose { println!("info string Fail high at depth {}, widening window [{}, {}]", depth, eval - 1, 1000000); }
                 // Keep lower bound tight, widen significantly upwards
                 alpha = eval - 1;
                 beta = 1000000;
                 delta += delta / 2; // Increase delta for next iteration's window width
                 // No break, loop again with wider window
            } else { // Search successful within the window
                 if verbose {
                     let elapsed = 0; // No time tracking here, maybe add if needed
                     let nps = 0;
                     // TODO: Extract PV
                     println!("info depth {} score cp {} time {} nodes {} nps {} pv {}",
                              depth, eval, elapsed, total_nodes, nps, print_move(&best_move));
                 }
                 delta = initial_delta + delta / 4; // Reset delta towards initial, but adapt slightly based on success
                 break; // Exit aspiration loop for this depth, move to next depth
            }
        }
         // Optional: Check for mate score and stop early
        if eval.abs() > 900000 { break; }
    }
    (eval, best_move, total_nodes)
}


/// Performs a quiescence search to evaluate tactical sequences and avoid the horizon effect.
fn quiescence_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    mut alpha: i32,
    beta: i32,
    max_depth: i32, // Remaining depth
    verbose: bool
) -> (i32, i32) {
    let mut nodes = 1;

    // --- Stand-Pat Evaluation ---
    let stand_pat = pesto.eval(&board.current_state(), move_gen);

    // --- Beta Cutoff Check ---
    if stand_pat >= beta {
        return (beta, nodes); // Fail-high
    }

    // --- Update Alpha ---
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // --- Max Depth Check ---
    if max_depth <= 0 {
        return (alpha, nodes); // Return current best score (alpha)
    }

    // --- Generate Captures (and potentially Promotions/Checks) ---
    // TODO: Consider adding Promotions and Checks to QSearch for more tactical stability
    let captures = move_gen.gen_pseudo_legal_captures(&board.current_state()); // Assuming this generates captures only

    // If no captures and not in check, return stand-pat based alpha
    // Note: If checks are added to QSearch, this condition needs adjustment
    if captures.is_empty() && !board.is_check(move_gen) {
        return (alpha, nodes);
    }

    // --- Iterate Through Captures ---
    // TODO: Order captures using MVV-LVA before SEE pruning for better results
    for capture in captures {
        // --- Static Exchange Evaluation (SEE) Pruning ---
        // Prune captures predicted to lose material (or break even if desired)
        // Ensure SEE function exists and is correct before enabling
        // if see(&board.current_state(), move_gen, capture.to(), capture.from()) < 0 {
        //      continue;
        // }

        board.make_move(capture);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }

        // Recursive call
        let (mut score, n) = quiescence_search(board, move_gen, pesto, -beta, -alpha, max_depth - 1, verbose);
        score = -score; // Negamax adjustment
        nodes += n;

        board.undo_move();

        // --- Alpha-Beta Pruning ---
        if score >= beta {
            return (beta, nodes); // Fail-high (beta cutoff)
        }
        if score > alpha {
            alpha = score; // Update best score found so far
        }
    }

    (alpha, nodes) // Return the best score found within the alpha-beta bounds
}


/// Perform a quiescence search with consistent side to move (Experimental, marked as slow)
fn q_search_consistent_side_to_move_for_final_eval(board: &mut BoardStack, move_gen: &MoveGen, pesto: &PestoEval, mut alpha: i32, beta: i32, eval_after_even_moves: bool, verbose: bool) -> (i32, i32) {
    let (checkmate, stalemate) = board.current_state().is_checkmate_or_stalemate(move_gen);
    if checkmate {
        if verbose { println!("Qsearch: Quiescence: Checkmate!"); }
        return (-1000000, 1);
    } else if stalemate {
        if verbose { println!("Quiescence: Stalemate!"); }
        return (0, 1);
    }

    if eval_after_even_moves {
        let eval = pesto.eval(&board.current_state(), move_gen);
        let captures = move_gen.gen_pseudo_legal_captures(&board.current_state());
        if captures.is_empty() {
             if verbose { println!("Quiescence: No captures left! Eval: {}", eval); }
            return (eval, 1);
        }

        if verbose { println!("Stand pat eval: {}", eval); }
        if eval > alpha {
            if eval >= beta {
                 if verbose { println!("Quiescence: Stand pat eval is better than beta! Eval: {}", eval); }
                return (beta, 1);
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
            if verbose { println!("Capture eval: {}", score); }
            n += nodes;
            board.undo_move();
            if score > alpha {
                alpha = score;
                if score >= beta { return (beta, n); }
            }
        }
        (alpha, n)
    } else {
        // Other side simply plays best move (including non-captures?) - This seems inconsistent with QSearch purpose
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves_with_evals(&mut board.current_state(), pesto);
        let mut n: i32 = 1;
        captures.extend(moves); // Includes non-captures, deviating from standard QSearch
        for c in captures {
            board.make_move(c);
            if !board.current_state().is_legal(move_gen) {
                board.undo_move();
                continue;
            }
            let (mut score, nodes) = q_search_consistent_side_to_move_for_final_eval(board, move_gen, pesto, -beta, -alpha, !eval_after_even_moves, verbose);
            score = -score;
            if verbose { println!("Other side eval: {}", score); }
            n += nodes;
            if score > alpha { alpha = score; }
            board.undo_move();
            if alpha >= beta { break; }
        }
        (alpha, n)
    }
}


/// Perform mate search from the given position
/// Searches only checking moves for the side to move.
pub fn mate_search(board: &mut BoardStack, move_gen: &MoveGen, max_depth: i32, verbose: bool) -> (i32, Move, i32) {
    let mut eval: i32 = 0;
    let mut best_move: Move = NULL_MOVE;
    let mut n: i32 = 0;
    let mut alpha = -1000000;
    let beta = 1000000;

    // Iterative deepening loop for mate search
    for d in 1..=max_depth {
        // Mate search often explores odd depths effectively
        let depth = d;
        if verbose { println!("info string Performing mate search at depth {} ply", depth); }

        let mut current_best_move_this_iter = NULL_MOVE;
        let mut current_alpha_this_iter = -1000000;

        // Generate all legal moves
        let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&mut board.current_state());
        captures.extend(moves);

        // Iterate through legal moves, prioritizing checks? (Or just filter later)
        for m in captures {
            board.make_move(m);
            if !board.current_state().is_legal(move_gen) {
                board.undo_move();
                continue;
            }
            // Only explore branches where the current move is a check
            if !board.is_check(move_gen) {
                board.undo_move();
                continue;
            }

            let (score, nodes) = mate_search_recursive(board, move_gen, depth - 1, -beta, -alpha); // Start recursive call
            let current_eval = -score;
            n += nodes;

            if current_eval > current_alpha_this_iter {
                current_alpha_this_iter = current_eval;
                current_best_move_this_iter = m;
            }
            board.undo_move();

            // Update overall alpha and best move for the iterative deepening
            if current_alpha_this_iter > alpha {
                 alpha = current_alpha_this_iter;
                 best_move = current_best_move_this_iter;
            }

            // Pruning within the top level loop (optional but can save time)
            if alpha >= beta { break; }
        }

        if verbose { println!("info depth {} score mate {} nodes {} pv {}", depth, alpha, n, print_move(&best_move)); }

        // If checkmate found, stop searching
        if alpha == 1000000 { // Check for mate score
            if verbose { println!("info string Mate found!"); }
            break;
        }
    }
    // Adjust score slightly based on depth? (e.g., faster mate is better)
    if alpha == 1000000 { eval = 1000000 - max_depth; } // Simple depth adjustment
    else if alpha == -1000000 { eval = -1000000 + max_depth; }
    else { eval = 0; } // No forced mate found within depth

    (eval, best_move, n)
}


/// Recursive helper function for mate search
/// Returns score relative to the side *whose turn it was at the start of this function call*.
/// Score: 1_000_000 for finding a mate, -1_000_000 if opponent can force mate, 0 otherwise.
fn mate_search_recursive(board: &mut BoardStack, move_gen: &MoveGen, depth: i32, mut alpha: i32, beta: i32) -> (i32, i32) {

    let mut nodes = 1;

    // Check for terminal node (mate) at the start of the ply
    // is_checkmate_or_stalemate checks for the *next* player
    let (is_mate, is_stalemate) = board.current_state().is_checkmate_or_stalemate(move_gen);
    if is_mate {
        return (-1000000, nodes); // Previous move delivered checkmate
    }
    if is_stalemate {
        return (0, nodes); // Stalemate is not a mate
    }

    if depth == 0 {
        return (0, nodes); // Reached depth limit without finding mate
    }

    // Generate legal moves for the current player
    let (mut captures, moves) = move_gen.gen_pseudo_legal_moves(&board.current_state());
    captures.extend(moves);

    let mut best_score = -1000000; // Assume we are mated unless a move avoids it

    for m in captures {
        board.make_move(m);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }

        // Recursive call for the opponent
        let (score, n_child) = mate_search_recursive(board, move_gen, depth - 1, -beta, -alpha);
        let current_eval = -score;
        nodes += n_child;

        board.undo_move();

        best_score = best_score.max(current_eval); // Find the best score we can achieve

        // Alpha-beta pruning
        if best_score > alpha {
            alpha = best_score;
        }
        if alpha >= beta {
            break; // Beta cutoff
        }
    }

    (best_score, nodes)
}


/// Calculates the Static Exchange Evaluation (SEE) for a move to a target square.
/// Determines if a sequence of captures on `target_sq` initiated by the current side to move
/// is likely to win material.
///
/// # Arguments
/// * `board` - The current board state.
/// * `move_gen` - The move generator (needed for finding attackers).
/// * `target_sq` - The square where the capture sequence occurs.
/// * `initial_attacker_sq` - The square of the piece making the initial capture.
///
/// # Returns
/// The estimated material balance after the exchange sequence. Positive means gain, negative means loss.
/// Returns 0 if the target square is empty or the initial attacker is invalid.
fn see(board: &Board, move_gen: &MoveGen, target_sq: usize, initial_attacker_sq: usize) -> i32 {
    let mut gain = [0; 32]; // Max possible captures in a sequence
    let mut depth = 0;
    let mut current_board = board.clone(); // Clone board to simulate captures
    let mut side_to_move = board.w_to_move;

    // Get initial captured piece type and value
    let captured_piece_type = match current_board.get_piece_type_on_sq(target_sq) {
        Some(pt) => pt,
        None => return 0, // Target square is empty, not a capture
    };
    gain[depth] = SEE_PIECE_VALUES[captured_piece_type as usize];

    // Get initial attacker piece type
    let attacker_piece_type = match current_board.get_piece_type_on_sq(initial_attacker_sq) {
         Some(pt) => pt,
         None => return 0, // Should not happen for a valid capture move
    };

    // Simulate the initial capture
    current_board.clear_square(initial_attacker_sq);
    current_board.set_square(target_sq, attacker_piece_type, side_to_move);
    // Important: Update occupancy after manual board changes if `attackers_to` relies on it
    current_board.update_occupancy();
    side_to_move = !side_to_move; // Switch sides

    loop {
        depth += 1;
        // Score relative to previous capture. gain[d] = value of attacker - gain[d-1]
        gain[depth] = SEE_PIECE_VALUES[attacker_piece_type as usize] - gain[depth - 1];

        // If the previous capture resulted in a score < 0 for the player who just captured,
        // and the current capture sequence score is also < 0 for that player,
        // then they wouldn't continue the sequence.
        if max(-gain[depth - 1], gain[depth]) < 0 {
            break;
        }

        // Find the least valuable attacker for the current side_to_move attacking the target square
        // Need move_gen methods: attackers_to, least_valuable_attacker_sq
        // TODO: Ensure these methods exist and work correctly on the cloned/modified board
        let attackers_bb = move_gen.attackers_to(&current_board, target_sq, side_to_move);
        if attackers_bb == 0 {
            break; // No more attackers for the current side
        }

        // Find the square of the least valuable piece attacking target_sq
        // This requires a helper in move_gen or here.
        let next_attacker_sq = find_least_valuable_attacker_sq(&current_board, attackers_bb, side_to_move);
         if next_attacker_sq == 64 { // No valid attacker found (shouldn't happen if attackers_bb > 0)
             break;
         }

        // Update attacker_piece_type for the next iteration's gain calculation
        let next_attacker_type = current_board.get_piece_type_on_sq(next_attacker_sq).unwrap();

        // Simulate the next capture
        current_board.clear_square(next_attacker_sq);
        current_board.set_square(target_sq, next_attacker_type, side_to_move);
        current_board.update_occupancy(); // Update occupancy again
        side_to_move = !side_to_move; // Switch sides
    }

    // Calculate final score by propagating the gains/losses back up the sequence
    while depth > 0 {
        depth -= 1;
        // The score at depth d is the negation of the maximum possible score after the opponent's move at depth d+1
        gain[depth] = -max(-gain[depth], gain[depth + 1]);
    }
    gain[0] // Final score from the perspective of the initial side_to_move
}

/// Helper function to find the square of the least valuable attacker from a bitboard of attackers.
/// TODO: Move this into move_generation or board_utils if appropriate.
fn find_least_valuable_attacker_sq(board: &Board, attackers_bb: u64, side: bool) -> usize {
     for piece_type_idx in PAWN..=KING { // Iterate from Pawn (least valuable) to King
         let piece_bb = board.pieces[side as usize][piece_type_idx as usize];
         let intersection = attackers_bb & piece_bb;
         if intersection != 0 {
             return intersection.trailing_zeros() as usize; // Return square of the first found attacker of this type
         }
     }
     64 // Indicate no attacker found (error condition)
}