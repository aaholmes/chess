// Struct representing the move generator, that generates pseudo-legal moves

use crate::move_types::Move;
use crate::bitboard::{Bitboard, sq_ind_to_bit, WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK, WOCC, BOCC, OCC};
use crate::bits::bits;
use crate::magic_constants::{R_MAGICS, B_MAGICS, R_BITS, B_BITS, R_MASKS, B_MASKS};
use crate::magic_bitboard::{init_pawn_moves, init_knight_moves, init_bishop_moves, init_rook_moves, init_king_moves, init_pawn_captures_promotions, append_promotions};

use crate::eval::PestoEval;

pub struct MoveGen {
    pub wp_captures: Vec<Vec<usize>>,
    pub bp_captures: Vec<Vec<usize>>,
    pub wp_capture_bitboard: [u64; 64],
    pub bp_capture_bitboard: [u64; 64],
    wp_promotions: Vec<Vec<usize>>,
    bp_promotions: Vec<Vec<usize>>,
    pub n_moves: Vec<Vec<usize>>,
    pub k_moves: Vec<Vec<usize>>,
    pub n_move_bitboard: [u64; 64],
    pub k_move_bitboard: [u64; 64],
    wp_moves: Vec<Vec<usize>>,
    bp_moves: Vec<Vec<usize>>,
    r_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    b_moves: Vec<Vec<(Vec<usize>, Vec<usize>)>>,
    r_move_bitboard: Vec<Vec<u64>>,
    b_move_bitboard: Vec<Vec<u64>>,
    b_magics: [u64; 64],
    r_magics: [u64; 64],
}

impl MoveGen {
    pub fn new() -> MoveGen {
        // Initialize the move generator by creating the iterators for Pawn, Knight, and King moves.
        let mut wp_captures: Vec<Vec<usize>> = Vec::new();
        let mut bp_captures: Vec<Vec<usize>> = Vec::new();
        let mut wp_capture_bitboard: [u64; 64] = [0; 64];
        let mut bp_capture_bitboard: [u64; 64] = [0; 64];
        let mut wp_promotions: Vec<Vec<usize>> = Vec::new();
        let mut bp_promotions: Vec<Vec<usize>> = Vec::new();
        let mut n_moves: Vec<Vec<usize>> = Vec::new();
        let mut k_moves: Vec<Vec<usize>> = Vec::new();
        let mut n_move_bitboard: [u64; 64] = [0; 64];
        let mut k_move_bitboard: [u64; 64] = [0; 64];
        let mut wp_moves: Vec<Vec<usize>> = Vec::new();
        let mut bp_moves: Vec<Vec<usize>> = Vec::new();
        let mut _wp: Vec<usize>;
        let mut _bp: Vec<usize>;
        let mut _wp_cap: Vec<usize>;
        let mut _bp_cap: Vec<usize>;
        let mut _wp_prom: Vec<usize>;
        let mut _bp_prom: Vec<usize>;
        for from_sq_ind in 0..64 {
            let (wp_cap, wp_prom, bp_cap, bp_prom) = init_pawn_captures_promotions(from_sq_ind);
            wp_captures.push(wp_cap.clone());
            bp_captures.push(bp_cap.clone());
            for i in &wp_captures[from_sq_ind] {
                wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            // Also add pawn moves from pawns on the first rank, as this is used in reverse to determine checks by pawns
            if from_sq_ind < 8 {
                if from_sq_ind < 7 {
                    wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind + 9);
                }
                if from_sq_ind > 0 {
                    wp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind + 7);
                }
            }
            for i in &bp_captures[from_sq_ind] {
                bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            // Also add pawn moves from pawns on the first rank, as this is used in reverse to determine checks by pawns
            if from_sq_ind > 55 {
                if from_sq_ind > 56 {
                    bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind - 9);
                }
                if from_sq_ind < 63 {
                    bp_capture_bitboard[from_sq_ind] |= sq_ind_to_bit(from_sq_ind - 7);
                }
            }
            wp_promotions.push(wp_prom.clone());
            bp_promotions.push(bp_prom.clone());
            n_moves.push(init_knight_moves(from_sq_ind));
            k_moves.push(init_king_moves(from_sq_ind));
            for i in &n_moves[from_sq_ind] {
                n_move_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            for i in &k_moves[from_sq_ind] {
                k_move_bitboard[from_sq_ind] |= sq_ind_to_bit(*i);
            }
            let (wp, bp) = init_pawn_moves(from_sq_ind);
            wp_moves.push(wp.clone());
            bp_moves.push(bp.clone());
        }
        let mut move_gen: MoveGen = MoveGen {
            wp_captures,
            bp_captures,
            wp_capture_bitboard,
            bp_capture_bitboard,
            wp_promotions,
            bp_promotions,
            n_moves,
            k_moves,
            n_move_bitboard,
            k_move_bitboard,
            wp_moves,
            bp_moves,
            r_moves: vec![],
            b_moves: vec![],
            r_move_bitboard: vec![],
            b_move_bitboard: vec![],
            b_magics: [0; 64],
            r_magics: [0; 64],
        };
        // Generate magic numbers for sliding pieces
        // let (b_magics, r_magics) = find_magic_numbers();
        move_gen.b_magics = B_MAGICS;
        move_gen.r_magics = R_MAGICS;
        let (b_moves, b_move_bitboard) = init_bishop_moves(move_gen.b_magics);
        move_gen.b_moves = b_moves;
        move_gen.b_move_bitboard = b_move_bitboard;
        let (r_moves, r_move_bitboard) = init_rook_moves(move_gen.r_magics);
        move_gen.r_moves = r_moves;
        move_gen.r_move_bitboard = r_move_bitboard;
        move_gen
    }

    pub fn gen_pseudo_legal_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all pseudo-legal moves for the current position, i.e., these moves may move into check.
        // Elsewhere we need to check for legality and perform move ordering.
        // Returns a vector of captures and a vector of non-captures.
        // Includes promotions as captures here
        let (mut captures, mut promotions, mut moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, mut moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, mut moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, mut moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, mut moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, mut moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);
        moves.append(&mut moves_knights);
        moves.append(&mut moves_bishops);
        moves.append(&mut moves_rooks);
        moves.append(&mut moves_queens);
        moves.append(&mut moves_kings);
        (captures, moves)
    }

    pub fn gen_pseudo_legal_moves_with_evals(&self, board: &mut Bitboard, pesto: &PestoEval) -> (Vec<Move>, Vec<Move>) {
        // Generate all pseudo-legal moves for the current position, i.e., these moves may move into check.
        // Elsewhere we need to check for legality and perform move ordering.
        // Returns a vector of captures and a vector of non-captures.
        // Includes promotions as captures here
        let (mut captures, mut promotions, mut moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, mut moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, mut moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, mut moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, mut moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, mut moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);
        moves.append(&mut moves_knights);
        moves.append(&mut moves_bishops);
        moves.append(&mut moves_rooks);
        moves.append(&mut moves_queens);
        moves.append(&mut moves_kings);

        // Here let's sort captures by MVV-LVA
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

        // Also sort moves by pesto eval change
        pesto.eval_update_board(board);
        moves.sort_unstable_by_key(|m| -pesto.move_eval(board, self, m.from, m.to));

        (captures, moves)
    }
    pub fn gen_pseudo_legal_captures(&self, board: &Bitboard) -> Vec<Move> {
        // Same as above, but only generate captures
        let (mut captures, mut promotions, _moves) = self.gen_pawn_moves(board);
        let (mut captures_knights, _moves_knights) = self.gen_knight_moves(board);
        let (mut captures_kings, _moves_kings) = self.gen_king_moves(board);
        let (mut captures_rooks, _moves_rooks) = self.gen_rook_moves(board);
        let (mut captures_bishops, _moves_bishops) = self.gen_bishop_moves(board);
        let (mut captures_queens, _moves_queens) = self.gen_queen_moves(board);
        captures.append(&mut captures_knights);
        captures.append(&mut captures_bishops);
        captures.append(&mut captures_rooks);
        captures.append(&mut captures_queens);
        captures.append(&mut captures_kings);
        captures.append(&mut promotions);

        // Here let's sort captures by MVV-LVA
        captures.sort_unstable_by_key(|m| -self.mvv_lva(board, m.from, m.to));

    captures
    }

    fn mvv_lva(&self, board: &Bitboard, from_sq_ind: usize, to_sq_ind: usize) -> i32 {
        // Return the MVV-LVA score for a capture move.
        // To enable sorting by MVV, then by LVA, we return the score as 10 * victim - attacker,
        // where value is 012345 for kpnbrq
        if board.get_piece(to_sq_ind).is_none() {
            return 0;
        }
        let victim = board.get_piece(to_sq_ind).unwrap() / 2;
        let attacker = board.get_piece(from_sq_ind).unwrap() / 2;
        10 * victim as i32 - attacker as i32
    }

    fn gen_pawn_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>, Vec<Move>) {
        // Generate all possible pawn moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Treats promotions as captures.
        // Lists promotions in the following order: queen, rook, knight, bishop, since bishop promotions are very rare.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut promotions: Vec<Move> = Vec::new();
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WP]) {
                for to_sq_ind in &self.wp_captures[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.wp_promotions[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                    }
                }
                for to_sq_ind in &self.wp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        if from_sq_ind > 47 && from_sq_ind < 56 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else if from_sq_ind > 7 && from_sq_ind < 16 {
                            if board.pieces[OCC] & (1 << (from_sq_ind + 8)) == 0 {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        } else {
                            moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BP]) {
                for to_sq_ind in &self.bp_captures[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 || board.en_passant == Some(*to_sq_ind) {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else {
                            captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
                for to_sq_ind in &self.bp_promotions[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        append_promotions(&mut promotions, from_sq_ind, to_sq_ind, board.w_to_move);
                    }
                }
                for to_sq_ind in &self.bp_moves[from_sq_ind] {
                    if board.get_piece(*to_sq_ind).is_none() {
                        if from_sq_ind > 7 && from_sq_ind < 16 {
                            append_promotions(&mut captures, from_sq_ind, to_sq_ind, board.w_to_move);
                        } else if from_sq_ind > 47 && from_sq_ind < 56 {
                            if board.pieces[OCC] & (1 << (from_sq_ind - 8)) == 0 {
                                moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                            }
                        } else {
                            moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                        }
                    }
                }
            }
        }
        (captures, promotions, moves)
    }

    fn gen_knight_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible knight moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        let mut moves: Vec<Move> = Vec::with_capacity(16);
        let mut captures: Vec<Move> = Vec::with_capacity(16);
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BN]) {
                for to_sq_ind in &self.n_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_king_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible king moves for the current position.
        // For castling, checks whether in check and whether the king moves through check.
        // Returns a vector of captures and a vector of non-captures.
        let mut moves: Vec<Move> = Vec::with_capacity(10);
        let mut captures: Vec<Move> = Vec::with_capacity(8);
        if board.w_to_move {
            // White to move
            if board.w_castle_k {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WR] & (1 << 7) != 0 && board.pieces[OCC] & ((1 << 5) | (1 << 6)) == 0 && !board.is_square_attacked(4, false, self) && !board.is_square_attacked(5, false, self) && !board.is_square_attacked(6, false, self) {
                    moves.push(Move::new(4, 6, None));
                }
            }
            if board.w_castle_q {
                // Make sure a rook is there because it could have been captured
                if board.pieces[WR] & (1 << 0) != 0 && board.pieces[OCC] & ((1 << 1) | (1 << 2) | (1 << 3)) == 0 && !board.is_square_attacked(4, false, self) && !board.is_square_attacked(3, false, self) && !board.is_square_attacked(2, false, self) {
                    moves.push(Move::new(4, 2, None));
                }
            }
            for from_sq_ind in bits(&board.pieces[WK]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            if board.b_castle_k {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BR] & (1 << 63) != 0 && board.pieces[OCC] & ((1 << 61) | (1 << 62)) == 0 && !board.is_square_attacked(60, true, self) && !board.is_square_attacked(61, true, self) && !board.is_square_attacked(62, true, self) {
                    moves.push(Move::new(60, 62, None));
                }
            }
            if board.b_castle_q {
                // Make sure a rook is there because it could have been captured
                if board.pieces[BR] & (1 << 56) != 0 && board.pieces[OCC] & ((1 << 57) | (1 << 58) | (1 << 59)) == 0 && !board.is_square_attacked(60, true, self) && !board.is_square_attacked(59, true, self) && !board.is_square_attacked(58, true, self) {
                    moves.push(Move::new(60, 58, None));
                }
            }
            for from_sq_ind in bits(&board.pieces[BK]) {
                for to_sq_ind in &self.k_moves[from_sq_ind] {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    } else if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_rook_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible rook moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WR]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BR]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    pub fn gen_bishop_potential_captures(&self, board: &Bitboard, from_sq_ind: usize) -> u64 {
        // Generate potential bishop captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = board.pieces[OCC] & B_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.b_move_bitboard[from_sq_ind][key]
    }

    pub fn gen_rook_potential_captures(&self, board: &Bitboard, from_sq_ind: usize) -> u64 {
        // Generate potential rook captures from the given square.
        // Used to determine whether a king is in check.

        // Mask blockers
        let blockers: u64 = board.pieces[OCC] & R_MASKS[from_sq_ind];

        // Generate the key using a multiplication and right shift
        let key: usize = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

        // Return the preinitialized capture set bitboard from the table
        self.r_move_bitboard[from_sq_ind][key]
    }

    fn gen_bishop_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible bishop moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WB]) {
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BB]) {
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }

    fn gen_queen_moves(&self, board: &Bitboard) -> (Vec<Move>, Vec<Move>) {
        // Generate all possible queen moves for the current position.
        // Returns a vector of captures and a vector of non-captures.
        // Uses magic bitboards.
        let mut moves: Vec<Move> = Vec::new();
        let mut captures: Vec<Move> = Vec::new();
        let mut blockers: u64;
        let mut key: usize;
        if board.w_to_move {
            // White to move
            for from_sq_ind in bits(&board.pieces[WQ]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[BOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[WOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        } else {
            // Black to move
            for from_sq_ind in bits(&board.pieces[BQ]) {
                // Mask blockers
                blockers = board.pieces[OCC] & R_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.r_magics[from_sq_ind])) >> (64 - R_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.r_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.r_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                // Mask blockers
                blockers = board.pieces[OCC] & B_MASKS[from_sq_ind];

                // Generate the key using a multiplication and right shift
                key = ((blockers.wrapping_mul(self.b_magics[from_sq_ind])) >> (64 - B_BITS[from_sq_ind])) as usize;

                // Return the preinitialized attack set bitboard from the table
                for to_sq_ind in &self.b_moves[from_sq_ind][key].0 {
                    if board.pieces[WOCC] & (1 << to_sq_ind) != 0 {
                        captures.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
                for to_sq_ind in &self.b_moves[from_sq_ind][key].1 {
                    // Have to make sure we're not capturing our own piece, since pieces on the edge are not included in blockers
                    if board.pieces[BOCC] & (1 << to_sq_ind) == 0 {
                        moves.push(Move::new(from_sq_ind, *to_sq_ind, None));
                    }
                }
            }
        }
        (captures, moves)
    }
}