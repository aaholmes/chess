use super::see::see;
use crate::boardstack::BoardStack;
use crate::eval::PestoEval;
use crate::move_generation::MoveGen;

/// Performs a quiescence search to evaluate tactical sequences and avoid the horizon effect.
pub fn quiescence_search(
    board: &mut BoardStack,
    move_gen: &MoveGen,
    pesto: &PestoEval,
    mut alpha: i32,
    beta: i32,
    max_depth: i32, // Remaining q-search depth
    verbose: bool,
    start_time: Option<Instant>, // Added
    time_limit: Option<Duration>, // Added
) -> (i32, i32) { // Return type remains (score, nodes) for now, termination checked after call
    let mut nodes = 1;

    // --- Time Check (Periodic) ---
    // Check time at the start of each quiescence call
    if let (Some(start), Some(limit)) = (start_time, time_limit) {
        // Check every N nodes? Simpler to check every call for now.
        if start.elapsed() >= limit {
            // Return stand_pat score if time runs out during qsearch
             let stand_pat = pesto.eval(&board.current_state(), move_gen);
             return (stand_pat, nodes);
        }
    }

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
    if max_depth <= 0 { // Check depth limit
        return (alpha, nodes); // Return current best score (alpha)
    }

    // --- Generate Captures ---
    let captures = move_gen.gen_pseudo_legal_captures(&board.current_state());

    // If no captures and not in check, return stand-pat based alpha
    if captures.is_empty() && !board.is_check(move_gen) {
        return (alpha, nodes);
    }

    // --- Iterate Through Captures ---
    for capture in captures {
        // --- Static Exchange Evaluation (SEE) Pruning ---
        if see(&board.current_state(), move_gen, capture.to, capture.from) < 0 {
            continue; // Skip this likely losing capture
        }

        board.make_move(capture);
        if !board.current_state().is_legal(move_gen) {
            board.undo_move();
            continue;
        }

        // Recursive call
        let (mut score, n) = quiescence_search(
            board,
            move_gen,
            pesto,
            -beta,
            -alpha,
            max_depth - 1,
            verbose,
            start_time, // Pass time info down
            time_limit, // Pass time info down
        );
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
