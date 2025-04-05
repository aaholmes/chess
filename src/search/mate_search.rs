use crate::boardstack::BoardStack;
use crate::move_generation::MoveGen;
use crate::move_types::Move;

/// Search for forced mate
///
/// This function performs an exhaustive search to find forced checkmate.
/// It only considers positions that lead to checkmate within the maximum depth.
///
/// # Arguments
///
/// * `board` - A mutable reference to the current board state
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
    let mut best_move = Move::new(0, 0, None);
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
                best_move = m;
            }
            board.undo_move();
            if alpha >= beta {
                break;
            }
        }
        if verbose {
            println!("At depth {} ply, searched {} nodes. best eval {}", depth, n, eval);
        }
        // If checkmate found, stop searching
        if eval == 1000000 {
            if verbose {
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