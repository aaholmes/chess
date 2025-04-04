#[cfg(test)]
mod tests {
    use kingfisher::board::Board;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{TWO_BISHOPS_BONUS, PASSED_PAWN_BONUS_MG, PASSED_PAWN_BONUS_EG, KING_SAFETY_PAWN_SHIELD_BONUS};
    use kingfisher::piece_types::{WHITE, BLACK, BISHOP, PAWN, KING};
    use std::cmp::min; // Added for get_raw_scores if needed, though not directly used there

    // --- Original Tests (Refined Assertions/FENs) ---

    #[test]
    fn test_initial_position_eval() {
        let board = Board::new(); // Uses default FEN internally
        let evaluator = PestoEval::new();
        let score = evaluator.eval(&board);
        // Initial score might not be exactly 0 due to PST asymmetry, but should be small.
        assert!(score.abs() < 20, "Initial score ({}) out of expected range +/- 20cp", score);
    }

    #[test]
    fn test_material_advantage() {
        // White is missing a rook compared to initial position
        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w KQkq - 0 1").expect("Valid FEN");
        let evaluator = PestoEval::new();
        let score = evaluator.eval(&board);
        // Expect score to be significantly negative (Black advantage)
        assert!(score < -300, "Score ({}) doesn't reflect missing rook advantage for Black", score);
    }

    #[test]
    fn test_positional_evaluation() {
        // Compare initial position to one after 1. e4 a6
        let initial_board = Board::new();
        let developed_board = Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").expect("Valid FEN");
        let evaluator = PestoEval::new();
        let initial_score = evaluator.eval(&initial_board);
        let developed_score = evaluator.eval(&developed_board);
        // White's e4 should give a positional plus compared to start
        assert!(developed_score > initial_score, "Developed score ({}) not > initial score ({})", developed_score, initial_score);
    }

    #[test]
    fn test_eval_flipped_for_black() {
        // Use the same position after 1. e4 a6, but flip the side to move
        let board_w_to_move = Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").expect("Valid FEN");
        let board_b_to_move = Board::new_from_fen("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2").expect("Valid FEN");
        let evaluator = PestoEval::new();
        let score_w_to_move = evaluator.eval(&board_w_to_move);
        let score_b_to_move = evaluator.eval(&board_b_to_move);
        // Scores should be exact opposites
        assert_eq!(score_b_to_move, -score_w_to_move, "Black score ({}) is not the negative of White score ({})", score_b_to_move, score_w_to_move);
    }

    // --- Tests for Added Evaluation Terms ---

    // Helper to get the raw MG/EG scores *before* tapering and side-to-move adjustment
    // This allows testing the contribution of individual terms more easily.
    fn get_raw_scores(evaluator: &PestoEval, board: &Board) -> (i32, i32) {
        // Re-implement the core logic of eval_plus_game_phase without the final tapering/sign flip
        // to isolate the raw MG/EG accumulation for testing bonuses.
        // NOTE: This duplicates logic from eval.rs but is useful for testing specific term contributions.
        // A more robust solution might involve exposing internal calculation steps from PestoEval.

        let mut mg = [0, 0];
        let mut eg = [0, 0];

        // Base Pesto PST scores
        for color in [WHITE, BLACK] {
            for piece in 0..6 {
                let mut piece_bb = board.pieces[color][piece];
                while piece_bb != 0 {
                    let sq = piece_bb.trailing_zeros() as usize;
                    // Need access to the internal tables or re-initialize them here for test
                    // For simplicity, let's assume we can approximate the base score calculation
                    // or focus on the *difference* caused by bonuses.
                    // Let's recalculate base PST scores here (less ideal but works for isolated test)
                    // Accessing internal tables directly would be better if PestoEval allowed it.
                    let pesto = PestoEval::new(); // Recreate to access tables easily in test context
                    mg[color] += pesto.mg_table[color][piece][sq];
                    eg[color] += pesto.eg_table[color][piece][sq];

                    piece_bb &= piece_bb - 1;
                }
            }
        }

         // --- Add Bonus Terms ---
         for color in [WHITE, BLACK] {
            let enemy_color = 1 - color;

            // 1. Two Bishops Bonus
            if kingfisher::bits::popcnt(board.pieces[color][BISHOP]) >= 2 {
                mg[color] += TWO_BISHOPS_BONUS[0];
                eg[color] += TWO_BISHOPS_BONUS[1];
            }

            // 2. Passed Pawn Bonus
            let friendly_pawns = board.pieces[color][PAWN];
            let enemy_pawns = board.pieces[enemy_color][PAWN];
            for sq in kingfisher::bits::bits(&friendly_pawns) {
                let passed_mask = kingfisher::board_utils::get_passed_pawn_mask(color, sq);
                if (passed_mask & enemy_pawns) == 0 {
                    let rank = kingfisher::board_utils::sq_to_rank(sq);
                    let bonus_rank = if color == WHITE { rank } else { 7 - rank };
                    mg[color] += PASSED_PAWN_BONUS_MG[bonus_rank];
                    eg[color] += PASSED_PAWN_BONUS_EG[bonus_rank];
                }
            }

            // 3. King Safety (Pawn Shield) Bonus
            let king_sq = board.pieces[color][KING].trailing_zeros() as usize;
            if king_sq < 64 {
                let shield_zone_mask = kingfisher::board_utils::get_king_shield_zone_mask(color, king_sq);
                let shield_pawns = kingfisher::bits::popcnt(shield_zone_mask & friendly_pawns);
                mg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[0];
                eg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[1];
            }
        }

        (mg[WHITE] - mg[BLACK], eg[WHITE] - eg[BLACK]) // Return raw W-B score
    }


    #[test]
    fn test_two_bishops_bonus() {
        let evaluator = PestoEval::new();
        // Position with White having two bishops, Black having one knight/one bishop
        let board_base = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN"); // Base (1B vs 1B)
        let board_w_2b = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5B2/PPPPPPPP/RNBQK1NR w KQkq - 0 1").expect("Valid FEN"); // White gets 2B (replaces G1 N)
        let board_b_2n = Board::new_from_fen("r1bqkb1r/pppppppp/2n2n2/8/8/5B2/PPPPPPPP/RNBQK1NR b KQkq - 0 1").expect("Valid FEN"); // Black gets 2N (replaces C8 B, G8 N)

        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_base);
        let (mg_w_2b, eg_w_2b) = get_raw_scores(&evaluator, &board_w_2b);
        let (mg_b_2n, eg_b_2n) = get_raw_scores(&evaluator, &board_b_2n); // Black has 2N, White has 2B

        // Check White gets bonus relative to base
        // Note: Adding a bishop also adds its PST value, so we check the *difference* matches the bonus + PST diff
        let f3_sq = kingfisher::board_utils::algebraic_to_sq_ind("f3");
        let g1_sq = kingfisher::board_utils::algebraic_to_sq_ind("g1");
        let pst_diff_mg = evaluator.mg_table[WHITE][BISHOP][f3_sq] - evaluator.mg_table[WHITE][KNIGHT][g1_sq];
        let pst_diff_eg = evaluator.eg_table[WHITE][BISHOP][f3_sq] - evaluator.eg_table[WHITE][KNIGHT][g1_sq];

        assert_eq!(mg_w_2b - mg_base, pst_diff_mg + TWO_BISHOPS_BONUS[0], "MG Two Bishops Bonus mismatch");
        assert_eq!(eg_w_2b - eg_base, pst_diff_eg + TWO_BISHOPS_BONUS[1], "EG Two Bishops Bonus mismatch");

        // Check Black does not get the bonus in board_b_2n (score should reflect White's 2B bonus)
        // Need base score for board_b_2n if Black had 1B/1N instead of 2N
        let board_b_base_equiv = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5B2/PPPPPPPP/RNBQK1NR b KQkq - 0 1").expect("Valid FEN"); // White 2B, Black 1B/1N
        let (mg_b_base, eg_b_base) = get_raw_scores(&evaluator, &board_b_base_equiv);
        // Calculate PST diff for Black 2N vs 1B/1N
        let c6_sq = kingfisher::board_utils::algebraic_to_sq_ind("c6");
        let f6_sq = kingfisher::board_utils::algebraic_to_sq_ind("f6");
        let c8_sq = kingfisher::board_utils::algebraic_to_sq_ind("c8");
        let g8_sq = kingfisher::board_utils::algebraic_to_sq_ind("g8");
        let pst_diff_b_mg = (evaluator.mg_table[BLACK][KNIGHT][c6_sq] + evaluator.mg_table[BLACK][KNIGHT][f6_sq]) - (evaluator.mg_table[BLACK][BISHOP][c8_sq] + evaluator.mg_table[BLACK][KNIGHT][g8_sq]);
        let pst_diff_b_eg = (evaluator.eg_table[BLACK][KNIGHT][c6_sq] + evaluator.eg_table[BLACK][KNIGHT][f6_sq]) - (evaluator.eg_table[BLACK][BISHOP][c8_sq] + evaluator.eg_table[BLACK][KNIGHT][g8_sq]);

        // Black's score (W-B) in 2N case vs base case should only differ by PSTs, not bishop bonus
        assert_eq!(mg_b_2n - mg_b_base, pst_diff_b_mg, "MG Black score mismatch (should not have bonus)");
        assert_eq!(eg_b_2n - eg_b_base, pst_diff_b_eg, "EG Black score mismatch (should not have bonus)");
    }

    #[test]
    fn test_passed_pawn_bonus() {
        let evaluator = PestoEval::new();
        // White pawn on e6, no black pawns on d, e, f files in front
        let board_w_pass = Board::new_from_fen("8/8/4P3/8/8/k7/pppppppp/RNBQ1BNR w KQ - 0 1").expect("Valid FEN"); // Removed K for simplicity
        // Base position without the passed pawn
        let board_w_base = Board::new_from_fen("8/8/8/8/8/k7/pppppppp/RNBQ1BNR w KQ - 0 1").expect("Valid FEN");
        // Black pawn on d3, no white pawns on c, d, e files in front
        let board_b_pass = Board::new_from_fen("rnbq1bnr/PPPPPPPP/K7/8/8/3p4/8/8 b kq - 0 1").expect("Valid FEN"); // Removed k
        // Base position without the passed pawn
        let board_b_base = Board::new_from_fen("rnbq1bnr/PPPPPPPP/K7/8/8/8/8/8 b kq - 0 1").expect("Valid FEN");


        let (mg_w_p, eg_w_p) = get_raw_scores(&evaluator, &board_w_pass);
        let (mg_w_b, eg_w_b) = get_raw_scores(&evaluator, &board_w_base);
        let (mg_b_p, eg_b_p) = get_raw_scores(&evaluator, &board_b_pass);
        let (mg_b_b, eg_b_b) = get_raw_scores(&evaluator, &board_b_base);

        // Calculate expected bonus for e6 pawn (rank 5 from White's perspective)
        let expected_bonus_mg_w = PASSED_PAWN_BONUS_MG[5];
        let expected_bonus_eg_w = PASSED_PAWN_BONUS_EG[5];
        // Calculate expected bonus for d3 pawn (rank 2 from White's perspective -> rank 5 for Black)
        let expected_bonus_mg_b = PASSED_PAWN_BONUS_MG[5];
        let expected_bonus_eg_b = PASSED_PAWN_BONUS_EG[5];

        // Calculate PST value for the pawn
        let e6_sq = kingfisher::board_utils::algebraic_to_sq_ind("e6");
        let d3_sq = kingfisher::board_utils::algebraic_to_sq_ind("d3");
        let pst_mg_w = evaluator.mg_table[WHITE][PAWN][e6_sq];
        let pst_eg_w = evaluator.eg_table[WHITE][PAWN][e6_sq];
        let pst_mg_b = evaluator.mg_table[BLACK][PAWN][d3_sq];
        let pst_eg_b = evaluator.eg_table[BLACK][PAWN][d3_sq];


        // Check White score difference includes PST + Bonus
        assert_eq!(mg_w_p - mg_w_b, pst_mg_w + expected_bonus_mg_w, "White MG passed pawn bonus mismatch");
        assert_eq!(eg_w_p - eg_w_b, pst_eg_w + expected_bonus_eg_w, "White EG passed pawn bonus mismatch");

        // Check Black score difference includes PST + Bonus (remember raw score is W-B)
        assert_eq!(mg_b_p - mg_b_b, -(pst_mg_b + expected_bonus_mg_b), "Black MG passed pawn bonus mismatch");
        assert_eq!(eg_b_p - eg_b_b, -(pst_eg_b + expected_bonus_eg_b), "Black EG passed pawn bonus mismatch");

    }

     #[test]
    fn test_king_safety_pawn_shield() {
        let evaluator = PestoEval::new();
        // White king on g1, pawns on f2, g2, h2 (good shield - 3 pawns in zone f2,g2,h2)
        let board_w_safe = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN");
        // White king on g1, pawns on f3, g2, h2 (weaker shield - 1 pawn g2 in zone)
        let board_w_less_safe = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5P2/PPPP1PPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN");

        let (mg_safe, eg_safe) = get_raw_scores(&evaluator, &board_w_safe);
        let (mg_less, eg_less) = get_raw_scores(&evaluator, &board_w_less_safe);

        // Calculate expected difference:
        // Safe: 3 pawns * bonus
        // Less Safe: 1 pawn * bonus (g2) + PST diff for f3 vs f2
        // Diff = (2 * bonus) + (PST(f2) - PST(f3))
        let f2_sq = kingfisher::board_utils::algebraic_to_sq_ind("f2");
        let f3_sq = kingfisher::board_utils::algebraic_to_sq_ind("f3");
        let pst_diff_mg = evaluator.mg_table[WHITE][PAWN][f2_sq] - evaluator.mg_table[WHITE][PAWN][f3_sq];
        let pst_diff_eg = evaluator.eg_table[WHITE][PAWN][f2_sq] - evaluator.eg_table[WHITE][PAWN][f3_sq];

        let expected_diff_mg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[0] + pst_diff_mg;
        let expected_diff_eg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[1] + pst_diff_eg;

        // Need to account for the PST change of the moved pawn f2->f3
        assert_eq!(mg_safe - mg_less, expected_diff_mg, "MG King Safety difference mismatch");
        assert_eq!(eg_safe - eg_less, expected_diff_eg, "EG King Safety difference mismatch");
    }
}