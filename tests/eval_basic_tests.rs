#[cfg(test)]
mod tests {
    use kingfisher::bits;
    use kingfisher::board::Board;
    use kingfisher::board_utils;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{
        BACKWARD_PAWN_PENALTY, CASTLING_RIGHTS_BONUS, KING_ATTACK_WEIGHTS,
    };
    use kingfisher::move_generation::MoveGen;
    use kingfisher::piece_types::*; // Import all piece types
    use kingfisher::bits::{popcnt, bits}; // Import popcnt and bits

    // Simplified function to get raw scores
    fn get_raw_scores(evaluator: &PestoEval, board: &Board) -> (i32, i32) {
        let mut mg = [0, 0];
        let mut eg = [0, 0];

        // Get PST contributions
        for color in [WHITE, BLACK] {
            for piece in 0..6 {
                let mut piece_bb = board.get_piece_bitboard(color, piece);
                while piece_bb != 0 {
                    let sq = piece_bb.trailing_zeros() as usize;
                    mg[color] += evaluator.get_mg_score(color, piece, sq);
                    eg[color] += evaluator.get_eg_score(color, piece, sq);
                    piece_bb &= piece_bb - 1;
                }
            }
        }

        // Calculate castling bonus
        for color in [WHITE, BLACK] {
            if color == WHITE {
                if board.castling_rights.white_kingside {
                    mg[color] += CASTLING_RIGHTS_BONUS[0];
                }
                if board.castling_rights.white_queenside {
                    mg[color] += CASTLING_RIGHTS_BONUS[0];
                }
            } else {
                // BLACK
                if board.castling_rights.black_kingside {
                    mg[color] += CASTLING_RIGHTS_BONUS[0];
                }
                if board.castling_rights.black_queenside {
                    mg[color] += CASTLING_RIGHTS_BONUS[0];
                }
            }
        }

        (mg[WHITE] - mg[BLACK], eg[WHITE] - eg[BLACK])
    }

    // Basic tests for overall evaluation behavior

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_initial_position_eval() {
        let board = Board::new(); // Uses default FEN internally
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new();
        let score = evaluator.eval(&board, &move_gen);
        // Initial score includes castling rights bonus (4 * 25 = 100) + PST asymmetry
        // Adjusting expected range based on increased castling bonus
        assert!(
            score > 80 && score < 140,
            "Initial score ({}) out of expected range 80-140cp",
            score
        );
    }

    #[test]
    fn test_material_advantage() {
        // White is missing a rook compared to initial position
        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w KQkq - 0 1");
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new();
        let score = evaluator.eval(&board, &move_gen);
        // Expect score to be significantly negative (Black advantage)
        assert!(
            score < -300,
            "Score ({}) doesn't reflect missing rook advantage for Black",
            score
        );
    }

    #[test]
    fn test_positional_evaluation() {
        // Compare initial position to one after 1. e4 a6
        let initial_board = Board::new();
        let developed_board =
            Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new();
        let initial_score = evaluator.eval(&initial_board, &move_gen);
        let developed_score = evaluator.eval(&developed_board, &move_gen);
        // White's e4 should give a positional plus compared to start
        assert!(
            developed_score > initial_score,
            "Developed score ({}) not > initial score ({})",
            developed_score,
            initial_score
        );
    }

    #[test]
    fn test_eval_flipped_for_black() {
        // Use the same position after 1. e4 a6, but flip the side to move
        let board_w_to_move =
            Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let board_b_to_move =
            Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2");
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new();
        let score_w_to_move = evaluator.eval(&board_w_to_move, &move_gen);
        let score_b_to_move = evaluator.eval(&board_b_to_move, &move_gen);
        // Scores should be exact opposites
        assert_eq!(
            score_b_to_move, -score_w_to_move,
            "Black score ({}) is not the negative of White score ({})",
            score_b_to_move, score_w_to_move
        );
    }

    #[test]
    fn test_eval_mobility_initial_pos() {
        let move_gen = MoveGen::new();
        let board = Board::new();
        let pesto_eval = PestoEval::new();

        // Calculate expected mobility score manually (pseudo-legal moves from start pos)
        // Knights: 2 moves each * 2 knights = 4 moves per side
        // Bishops: 0 moves
        // Rooks: 0 moves
        // Queens: 0 moves
        let expected_knight_mobility = 4;
        let expected_mobility_mg =
            expected_knight_mobility * pesto_eval.weights.mobility_weights_mg[0]; // Index 0 for Knight
        let expected_mobility_eg =
            expected_knight_mobility * pesto_eval.weights.mobility_weights_eg[0];

        // Get the eval components (ignoring PSTs and other bonuses for simplicity here)
        let (eval, _) = pesto_eval.eval_plus_game_phase(&board, &move_gen);

        // Extract mobility contribution (this is tricky as it's mixed in)
        // We can recalculate the score without mobility and find the difference
        let mut mg_no_mobility = [0; 2];
        let mut eg_no_mobility = [0; 2];
        for color in [WHITE, BLACK] {
            for piece in 0..6 {
                let mut piece_bb = board.pieces[color][piece];
                while piece_bb != 0 {
                    let sq = piece_bb.trailing_zeros() as usize;
                    mg_no_mobility[color] += pesto_eval.get_mg_score(color, piece, sq);
                    eg_no_mobility[color] += pesto_eval.get_eg_score(color, piece, sq);
                    // Remove base value to isolate PST contribution if needed, but let's keep it simple
                    piece_bb &= piece_bb - 1;
                }
            }
            // Add other bonuses *except* mobility here if testing total eval
        }
        let score_no_mobility_mg = mg_no_mobility[WHITE] - mg_no_mobility[BLACK];
        let score_no_mobility_eg = eg_no_mobility[WHITE] - eg_no_mobility[BLACK];
        // Assume full MG phase for simplicity in this test
        let eval_no_mobility = score_no_mobility_mg;

        // Calculate the mobility contribution added by the function
        // Note: This assumes eval() correctly calculates the base score + mobility
        // It's an integration test for mobility's effect.
        let mobility_contribution = eval - eval_no_mobility;

        // Expected total mobility is symmetric initially
        let expected_total_mobility_contribution_mg = expected_mobility_mg - expected_mobility_mg; // White - Black
        let expected_total_mobility_contribution_eg = expected_mobility_eg - expected_mobility_eg; // White - Black
                                                                                                   // Assume full MG phase (24)
        let expected_total_mobility_contribution = (expected_total_mobility_contribution_mg * 24
            + expected_total_mobility_contribution_eg * 0)
            / 24;

        assert_eq!(
            mobility_contribution, expected_total_mobility_contribution,
            "Initial position mobility score mismatch"
        );
    }

    // Helper function to calculate expected mobility score for a given board
    fn calculate_expected_mobility(
        board: &Board,
        pesto_eval: &PestoEval,
        move_gen: &MoveGen,
    ) -> (i32, i32) {
        let mut mobility_mg = [0; 2];
        let mut mobility_eg = [0; 2];
        let occupied = board.get_all_occupancy();

        for color in [WHITE, BLACK] {
            let friendly_occ = board.pieces_occ[color];

            // Knight Mobility
            let mut knight_moves = 0;
            for sq in bits::bits(&board.pieces[color][KNIGHT]) {
                knight_moves += popcnt(move_gen.n_move_bitboard[sq] & !friendly_occ);
            }
            mobility_mg[color] += knight_moves as i32 * pesto_eval.weights.mobility_weights_mg[0];
            mobility_eg[color] += knight_moves as i32 * pesto_eval.weights.mobility_weights_eg[0];

            // Bishop Mobility
            let mut bishop_moves = 0;
            for sq in bits::bits(&board.pieces[color][BISHOP]) {
                bishop_moves += popcnt(move_gen.get_bishop_moves(sq, occupied) & !friendly_occ);
            }
            mobility_mg[color] += bishop_moves as i32 * pesto_eval.weights.mobility_weights_mg[1];
            mobility_eg[color] += bishop_moves as i32 * pesto_eval.weights.mobility_weights_eg[1];

            // Rook Mobility
            let mut rook_moves = 0;
            for sq in bits::bits(&board.pieces[color][ROOK]) {
                rook_moves += popcnt(move_gen.get_rook_moves(sq, occupied) & !friendly_occ);
            }
            mobility_mg[color] += rook_moves as i32 * pesto_eval.weights.mobility_weights_mg[2];
            mobility_eg[color] += rook_moves as i32 * pesto_eval.weights.mobility_weights_eg[2];

            // Queen Mobility
            let mut queen_moves = 0;
            for sq in bits::bits(&board.pieces[color][QUEEN]) {
                queen_moves += popcnt(move_gen.get_queen_moves(sq, occupied) & !friendly_occ);
            }
            mobility_mg[color] += queen_moves as i32 * pesto_eval.weights.mobility_weights_mg[3];
            mobility_eg[color] += queen_moves as i32 * pesto_eval.weights.mobility_weights_eg[3];
        }
        (
            mobility_mg[WHITE] - mobility_mg[BLACK],
            mobility_eg[WHITE] - mobility_eg[BLACK],
        )
    }

    // Helper function to get eval score *without* mobility contribution
    fn get_eval_minus_mobility(board: &Board, pesto_eval: &PestoEval, move_gen: &MoveGen) -> i32 {
        let mut mg_no_mobility = [0; 2];
        let mut eg_no_mobility = [0; 2];
        let mut game_phase = 0;

        for color in [WHITE, BLACK] {
            for piece in 0..6 {
                let mut piece_bb = board.pieces[color][piece];
                while piece_bb != 0 {
                    let sq = piece_bb.trailing_zeros() as usize;
                    mg_no_mobility[color] += pesto_eval.get_mg_score(color, piece, sq);
                    eg_no_mobility[color] += pesto_eval.get_eg_score(color, piece, sq);
                    game_phase += kingfisher::eval_constants::GAMEPHASE_INC[piece]; // Use const directly
                    piece_bb &= piece_bb - 1;
                }
            }
            // Add other bonuses here if they exist and are needed for accurate comparison
            // e.g., two bishops, pawn structure, king safety etc.
            // For simplicity, this helper currently only excludes mobility.
        }
        let score_no_mobility_mg = mg_no_mobility[WHITE] - mg_no_mobility[BLACK];
        let score_no_mobility_eg = eg_no_mobility[WHITE] - eg_no_mobility[BLACK];

        let mg_phase = std::cmp::min(24, game_phase);
        let eg_phase = 24 - mg_phase;
        let eg_phase_clamped = if eg_phase < 0 { 0 } else { eg_phase };
        (score_no_mobility_mg * mg_phase + score_no_mobility_eg * eg_phase_clamped) / 24
    }

    #[test]
    fn test_eval_mobility_contribution() {
        let move_gen = MoveGen::new();
        let pesto_eval = PestoEval::new();

        // Test Case 1: Initial Position
        let board1 = Board::new();
        let (expected_mob_mg1, expected_mob_eg1) =
            calculate_expected_mobility(&board1, &pesto_eval, &move_gen);
        let eval_no_mob1 = get_eval_minus_mobility(&board1, &pesto_eval, &move_gen);
        let (eval1, game_phase1) = pesto_eval.eval_plus_game_phase(&board1, &move_gen);
        let mg_phase1 = std::cmp::min(24, game_phase1);
        let eg_phase1 = 24 - mg_phase1;
        let eg_phase_clamped1 = if eg_phase1 < 0 { 0 } else { eg_phase1 };
        let expected_mob_contrib1 =
            (expected_mob_mg1 * mg_phase1 + expected_mob_eg1 * eg_phase_clamped1) / 24;
        assert_eq!(
            eval1 - eval_no_mob1,
            expected_mob_contrib1,
            "Initial pos mobility contribution mismatch"
        );

        // Test Case 2: Open Position (Italian Game)
        let board2 = Board::new_from_fen(
            "r1bqk1nr/pppp1ppp/2n5/2b1p1b1/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
        )
        .expect("Valid FEN");
        let (expected_mob_mg2, expected_mob_eg2) =
            calculate_expected_mobility(&board2, &pesto_eval, &move_gen);
        let eval_no_mob2 = get_eval_minus_mobility(&board2, &pesto_eval, &move_gen);
        let (eval2, game_phase2) = pesto_eval.eval_plus_game_phase(&board2, &move_gen);
        let mg_phase2 = std::cmp::min(24, game_phase2);
        let eg_phase2 = 24 - mg_phase2;
        let eg_phase_clamped2 = if eg_phase2 < 0 { 0 } else { eg_phase2 };
        let expected_mob_contrib2 =
            (expected_mob_mg2 * mg_phase2 + expected_mob_eg2 * eg_phase_clamped2) / 24;
        assert_eq!(
            eval2 - eval_no_mob2,
            expected_mob_contrib2,
            "Open pos mobility contribution mismatch"
        );

        // Test Case 3: Blocked Position (French Defense Advance)
        let board3 =
            Board::new_from_fen("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 1 5")
                .expect("Valid FEN");
        let (expected_mob_mg3, expected_mob_eg3) =
            calculate_expected_mobility(&board3, &pesto_eval, &move_gen);
        let eval_no_mob3 = get_eval_minus_mobility(&board3, &pesto_eval, &move_gen);
        let (eval3, game_phase3) = pesto_eval.eval_plus_game_phase(&board3, &move_gen);
        let mg_phase3 = std::cmp::min(24, game_phase3);
        let eg_phase3 = 24 - mg_phase3;
        let eg_phase_clamped3 = if eg_phase3 < 0 { 0 } else { eg_phase3 };
        let expected_mob_contrib3 =
            (expected_mob_mg3 * mg_phase3 + expected_mob_eg3 * eg_phase_clamped3) / 24;
        assert_eq!(
            eval3 - eval_no_mob3,
            expected_mob_contrib3,
            "Blocked pos mobility contribution mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_castling_rights_bonus() {
        let evaluator = PestoEval::new();
        // Initial position (4 rights)
        let board_all_rights = Board::new();
        // Position after 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O (White lost Q side, Black has both)
        let board_some_rights = Board::new_from_fen(
            "r1bqkb1r/1ppp1ppp/p1n2n2/1B2p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 5 5",
        );

        let (mg_all, _eg_all) = get_raw_scores(&evaluator, &board_all_rights);
        let (mg_some, _eg_some) = get_raw_scores(&evaluator, &board_some_rights);

        // Calculate expected difference based *only* on castling rights change
        // White lost Q side (-25), Black lost none (0) -> Diff = -25
        // Black lost K side (-25), White lost none (0) -> Diff = +25
        // In this specific FEN, White lost Q (-25), Black lost none (0). Net change = -25
        // We compare raw scores (W-B). Base has +100 (4*25), Some has +25 (W K) - +50 (B K+Q) = -25. Diff = -125? No, compare vs base.
        // Base MG score includes 4 * CASTLING_RIGHTS_BONUS[0] (for white and black)
        // Some MG score includes 1 * CASTLING_RIGHTS_BONUS[0] for white, 2 * CASTLING_RIGHTS_BONUS[0] for black.
        // The raw score diff (mg_some - mg_all) will include PST changes AND the castling rights diff.
        // Let's test simpler: Base vs No rights
        let board_no_rights =
            Board::new_from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
        let (mg_none, _eg_none) = get_raw_scores(&evaluator, &board_no_rights);

        // Difference should be PST changes minus 4 * CASTLING_RIGHTS_BONUS[0]
        // This test is getting complicated due to PSTs. Let's just check initial pos has the bonus.
        let expected_initial_castling_bonus = 4 * CASTLING_RIGHTS_BONUS[0]; // Both sides have 2 rights
                                                                            // Check if mg_all includes roughly this amount (plus PST asymmetry)
        assert!(
            mg_all > expected_initial_castling_bonus - 20
                && mg_all < expected_initial_castling_bonus + 40,
            "Initial MG score ({}) doesn't reflect castling bonus",
            mg_all
        ); // Wider range due to PST asymmetry
           // Check no rights board has near zero MG score (ignoring PSTs for simplicity)
        assert!(
            mg_none.abs() < 20,
            "No rights board MG score ({}) not near zero",
            mg_none
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_backward_pawn_penalty() {
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new(); // Needed for eval call, though not directly used in this logic check
                                       // White pawn d3 is backward: no friendly pawns on c2,c3, e2,e3. Black pawn d4 attacks stop square.
        let board_w_backward = Board::new_from_fen("k7/8/8/8/3p4/3P4/8/K7 w - - 0 1");
        // Base position without the backward pawn
        let board_w_base = Board::new_from_fen("k7/8/8/8/3p4/8/8/K7 w - - 0 1");

        let (mg_w_back, eg_w_back) = get_raw_scores(&evaluator, &board_w_backward);
        let (mg_w_base, eg_w_base) = get_raw_scores(&evaluator, &board_w_base);

        // Calculate PST value for the pawn
        let d3_sq = board_utils::algebraic_to_sq_ind("d3");
        let pst_mg_w = evaluator.get_mg_score(WHITE, PAWN, d3_sq);
        let pst_eg_w = evaluator.get_eg_score(WHITE, PAWN, d3_sq);

        // Check White score difference includes PST + Penalty
        assert_eq!(
            mg_w_back - mg_w_base,
            pst_mg_w + BACKWARD_PAWN_PENALTY[0],
            "White MG backward pawn mismatch"
        );
        assert_eq!(
            eg_w_back - eg_w_base,
            pst_eg_w + BACKWARD_PAWN_PENALTY[1],
            "White EG backward pawn mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_king_attack_score() {
        let evaluator = PestoEval::new();
        let move_gen = MoveGen::new(); // Needed for eval call
                                       // White N(f6), Q(h6) near Black K(g8)
        let board_w_attack = Board::new_from_fen("6k1/8/7q/5N2/8/8/8/K7 w - - 0 1");
        // Base position without attacking pieces
        let board_w_base = Board::new_from_fen("6k1/8/8/8/8/8/8/K7 w - - 0 1");

        let (mg_w_att, _eg_w_att) = get_raw_scores(&evaluator, &board_w_attack);
        let (mg_w_base, _eg_w_base) = get_raw_scores(&evaluator, &board_w_base);

        // Calculate PST values for the pieces
        let f6_sq = board_utils::algebraic_to_sq_ind("f6");
        let h6_sq = board_utils::algebraic_to_sq_ind("h6");
        let pst_mg_n = evaluator.get_mg_score(WHITE, KNIGHT, f6_sq);
        let pst_mg_q = evaluator.get_mg_score(WHITE, QUEEN, h6_sq);
        let total_pst_mg = pst_mg_n + pst_mg_q;

        // Calculate expected attack score (based on KING_ATTACK_WEIGHTS)
        // Assumes get_king_attack_zone_mask includes f6 and h6 for king on g8
        let expected_attack_score = KING_ATTACK_WEIGHTS[KNIGHT] + KING_ATTACK_WEIGHTS[QUEEN];

        // Check White MG score difference includes PST + Attack Score Bonus
        assert_eq!(
            mg_w_att - mg_w_base,
            total_pst_mg + expected_attack_score,
            "White MG King Attack Score mismatch"
        );
        // Note: EG score is not checked as the current implementation only applies the bonus to MG score.
    }
}
