// Alpha-beta negamax search

use crate::bitboard::Bitboard;
use crate::gen_moves::MoveGen;
use crate::eval::PestoEval;


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
    // Returns the eval (in centipawns) of the final position, as well as the first move
    // to play from the current position
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

// pub fn alpha_beta_search(board: &Bitboard, move_gen: &MoveGen, depth: i32) -> (i32, (usize, usize, Option<usize>)) {
//     // Perform alpha-beta search from the given position
//     // Exhaustive search to the given depth
//     // Returns the eval (in centipawns) of the final position, as well as the first move
//     // to play from the current position
//     let mut best_eval: i32 = -1000000;
//     let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
//
//
//
//
// }
//
//
// fn alpha_beta(board: &Bitboard, depth: i32, alpha: i32, beta: i32) -> i32 {
//     // Private recursive function used for alpha-beta search
//     // External functions should call alpha_beta_search instead
//     // Returns the eval (in centipawns) of the final position
//     if board.is_checkmate() {
//         // Checkmate
//         if board.w_to_move {
//             return -1000000;
//         } else {
//             return 1000000;
//         }
//     } else if board.is_stalemate() {
//         // Stalemate
//         return 0;
//     } else if depth == 0 {
//         // Leaf node
//         return board.eval(); // TODO: put quiescence search here
//     }
//     // Non-leaf node
//     let mut best_eval: i32 = -1000000;
//     let mut best_move: (usize, usize, Option<usize>) = (0, 0, None);
//     for from_sq_ind in board.get_piece_squares() {
//         for to_sq_ind in board.get_legal_moves(from_sq_ind) {
//             let mut new_board: Bitboard = board.clone();
//             new_board.make_move(from_sq_ind, to_sq_ind, None);
//             let eval: i32 = -alpha_beta(&new_board, depth - 1, -beta, -alpha);
//             if eval > best_eval {
//                 best_eval = eval;
//                 best_move = (from_sq_ind, to_sq_ind, None);
//             }
//             if eval > alpha {
//                 alpha = eval;
//             }
//             if alpha >= beta {
//                 break;
//             }
//         }
//     }
//     return best_eval;
// }