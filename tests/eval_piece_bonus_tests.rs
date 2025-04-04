#[cfg(test)]
mod tests {
    // Include the helper module
    #[path = "eval_test_utils.rs"]
    mod eval_test_utils;
    use eval_test_utils::get_raw_scores;

    use kingfisher::board::Board;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{TWO_BISHOPS_BONUS, KING_SAFETY_PAWN_SHIELD_BONUS};
    use kingfisher::piece_types::{WHITE, BLACK, BISHOP, PAWN, KNIGHT}; // Added KNIGHT
    use kingfisher::board_utils;

    #[test]
    fn test_two_bishops_bonus() {
        let evaluator = PestoEval::new();
        // Position with White having two bishops, Black having one knight/one bishop
        let board_base = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN"); // Base (1B vs 1B)
        let board_w_2b = Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5B2/PPPPPPPP/RNBQK1NR w KQkq - 0 1").expect("Valid FEN"); // White gets 2B (replaces G1 N)

        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_base);
        let (mg_w_2b, eg_w_2b) = get_raw_scores(&evaluator, &board_w_2b);

        // Check White gets bonus relative to base
        // Note: Adding a bishop also adds its PST value, so we check the *difference* matches the bonus + PST diff
        let f3_sq = board_utils::algebraic_to_sq_ind("f3");
        let g1_sq = board_utils::algebraic_to_sq_ind("g1");
        // Need direct access to tables for accurate PST diff
        let pesto_instance = PestoEval::new(); // Create instance to access tables
        let pst_diff_mg = pesto_instance.mg_table[WHITE][BISHOP][f3_sq] - pesto_instance.mg_table[WHITE][KNIGHT][g1_sq];
        let pst_diff_eg = pesto_instance.eg_table[WHITE][BISHOP][f3_sq] - pesto_instance.eg_table[WHITE][KNIGHT][g1_sq];

        assert_eq!(mg_w_2b - mg_base, pst_diff_mg + TWO_BISHOPS_BONUS[0], "MG Two Bishops Bonus mismatch");
        assert_eq!(eg_w_2b - eg_base, pst_diff_eg + TWO_BISHOPS_BONUS[1], "EG Two Bishops Bonus mismatch");

        // Test case where black gets 2 bishops vs 1B/1N
        let board_b_base_equiv = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1").expect("Valid FEN"); // White 1B/1N, Black 1B/1N
        let board_b_2b = Board::new_from_fen("rn1qkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1").expect("Valid FEN"); // Add black bishop (replace c8 pawn for simplicity)
        // Manually modify board state for test setup
        let mut board_b_2b_mut = board_b_2b;
        let c7_sq = board_utils::algebraic_to_sq_ind("c7");
        board_b_2b_mut.pieces[BLACK][PAWN] &= !board_utils::sq_ind_to_bit(c7_sq); // Remove c7 pawn
        board_b_2b_mut.pieces[BLACK][BISHOP] |= board_utils::sq_ind_to_bit(c7_sq); // Add bishop on c7
        board_b_2b_mut.update_occupancy(); // Important!

        let (mg_b_base, eg_b_base) = get_raw_scores(&evaluator, &board_b_base_equiv);
        let (mg_b_2b, eg_b_2b) = get_raw_scores(&evaluator, &board_b_2b_mut);

        let pst_diff_b_mg = pesto_instance.mg_table[BLACK][BISHOP][c7_sq] - pesto_instance.mg_table[BLACK][PAWN][c7_sq];
        let pst_diff_b_eg = pesto_instance.eg_table[BLACK][BISHOP][c7_sq] - pesto_instance.eg_table[BLACK][PAWN][c7_sq];

        // Black's score (W-B) should decrease by (PST diff + Bonus)
        assert_eq!(mg_b_2b - mg_b_base, -(pst_diff_b_mg + TWO_BISHOPS_BONUS[0]), "MG Black Two Bishops Bonus mismatch");
        assert_eq!(eg_b_2b - eg_b_base, -(pst_diff_b_eg + TWO_BISHOPS_BONUS[1]), "EG Black Two Bishops Bonus mismatch");
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
        let f2_sq = board_utils::algebraic_to_sq_ind("f2");
        let f3_sq = board_utils::algebraic_to_sq_ind("f3");
        let pesto_instance = PestoEval::new(); // Create instance to access tables
        let pst_diff_mg = pesto_instance.mg_table[WHITE][PAWN][f2_sq] - pesto_instance.mg_table[WHITE][PAWN][f3_sq];
        let pst_diff_eg = pesto_instance.eg_table[WHITE][PAWN][f2_sq] - pesto_instance.eg_table[WHITE][PAWN][f3_sq];

        let expected_diff_mg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[0] + pst_diff_mg;
        let expected_diff_eg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[1] + pst_diff_eg;

        // Need to account for the PST change of the moved pawn f2->f3
        assert_eq!(mg_safe - mg_less, expected_diff_mg, "MG King Safety difference mismatch");
        assert_eq!(eg_safe - eg_less, expected_diff_eg, "EG King Safety difference mismatch");
    }
}