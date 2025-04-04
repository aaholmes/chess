#[cfg(test)]
mod tests {
    // Include the helper module
    #[path = "eval_test_utils.rs"]
    mod eval_test_utils;
    use eval_test_utils::get_raw_scores;

    use kingfisher::board::Board;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{
        ROOK_ON_SEVENTH_BONUS, // Keep for reference, though bonus is removed
        ROOK_BEHIND_PASSED_PAWN_BONUS,
        DOUBLED_ROOKS_ON_SEVENTH_BONUS, ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS,
        ROOK_OPEN_FILE_BONUS, ROOK_HALF_OPEN_FILE_BONUS
    };
    use kingfisher::piece_types::{WHITE, BLACK, ROOK, PAWN};
    use kingfisher::board_utils;

    #[test]
    fn test_rook_on_seventh_removed() {
        // This test verifies the *single* rook bonus is gone,
        // but PSTs still apply. Doubled bonus tested separately.
        let evaluator = PestoEval::new();
        let board_w_r7 = Board::new_from_fen("k7/3R4/8/8/8/8/8/K7 w - - 0 1").expect("Valid FEN");
        let board_w_base = Board::new_from_fen("k7/8/8/8/8/8/8/K7 w - - 0 1").expect("Valid FEN");

        let (mg_w_r7, eg_w_r7) = get_raw_scores(&evaluator, &board_w_r7);
        let (mg_w_base, eg_w_base) = get_raw_scores(&evaluator, &board_w_base);

        let d7_sq = board_utils::algebraic_to_sq_ind("d7");
        let pesto_instance = PestoEval::new();
        let pst_mg_w = pesto_instance.mg_table[WHITE][ROOK][d7_sq];
        let pst_eg_w = pesto_instance.eg_table[WHITE][ROOK][d7_sq];

        // Check White score difference includes ONLY PST
        assert_eq!(mg_w_r7 - mg_w_base, pst_mg_w, "White MG Rook on 7th PST mismatch (bonus should be removed)");
        assert_eq!(eg_w_r7 - eg_w_base, pst_eg_w, "White EG Rook on 7th PST mismatch (bonus should be removed)");
    }

     #[test]
    fn test_doubled_rooks_on_seventh() {
        let evaluator = PestoEval::new();
        // White rooks on d7, e7
        let board_w_2r7 = Board::new_from_fen("k7/3RR3/8/8/8/8/8/K7 w - - 0 1").expect("Valid FEN");
        // Base with only d7 rook
        let board_w_1r7 = Board::new_from_fen("k7/3R4/8/8/8/8/8/K7 w - - 0 1").expect("Valid FEN");

        let (mg_2r7, eg_2r7) = get_raw_scores(&evaluator, &board_w_2r7);
        let (mg_1r7, eg_1r7) = get_raw_scores(&evaluator, &board_w_1r7);

        let e7_sq = board_utils::algebraic_to_sq_ind("e7");
        let pesto_instance = PestoEval::new();
        let pst_mg = pesto_instance.mg_table[WHITE][ROOK][e7_sq];
        let pst_eg = pesto_instance.eg_table[WHITE][ROOK][e7_sq];

        // Difference should be PST of 2nd rook + Doubled Rook Bonus
        assert_eq!(mg_2r7 - mg_1r7, pst_mg + DOUBLED_ROOKS_ON_SEVENTH_BONUS[0], "White MG Doubled Rooks 7th mismatch");
        assert_eq!(eg_2r7 - eg_1r7, pst_eg + DOUBLED_ROOKS_ON_SEVENTH_BONUS[1], "White EG Doubled Rooks 7th mismatch");
    }


    #[test]
    fn test_rook_behind_passed_pawn() {
        let evaluator = PestoEval::new();
        // White rook R_e1 behind passed pawn P_e6
        let board_w_rpp = Board::new_from_fen("8/8/4P3/k7/8/8/8/K3R3 w - - 0 1").expect("Valid FEN");
        // Base without rook
        let board_w_base = Board::new_from_fen("8/8/4P3/k7/8/8/8/K7 w - - 0 1").expect("Valid FEN");
        // Black rook R_d8 behind passed pawn p_d3
        let board_b_rpp = Board::new_from_fen("k2r4/8/8/8/K7/3p4/8/8 b - - 0 1").expect("Valid FEN");
        // Base without rook
        let board_b_base = Board::new_from_fen("k7/8/8/8/K7/3p4/8/8 b - - 0 1").expect("Valid FEN");

        let (mg_w_rpp, eg_w_rpp) = get_raw_scores(&evaluator, &board_w_rpp);
        let (mg_w_base, eg_w_base) = get_raw_scores(&evaluator, &board_w_base);
        let (mg_b_rpp, eg_b_rpp) = get_raw_scores(&evaluator, &board_b_rpp);
        let (mg_b_base, eg_b_base) = get_raw_scores(&evaluator, &board_b_base);

        let e1_sq = board_utils::algebraic_to_sq_ind("e1");
        let d8_sq = board_utils::algebraic_to_sq_ind("d8");
        let pesto_instance = PestoEval::new();
        let pst_mg_w = pesto_instance.mg_table[WHITE][ROOK][e1_sq];
        let pst_eg_w = pesto_instance.eg_table[WHITE][ROOK][e1_sq];
        let pst_mg_b = pesto_instance.mg_table[BLACK][ROOK][d8_sq];
        let pst_eg_b = pesto_instance.eg_table[BLACK][ROOK][d8_sq];

        // Check White score difference includes PST + Bonus
        assert_eq!(mg_w_rpp - mg_w_base, pst_mg_w + ROOK_BEHIND_PASSED_PAWN_BONUS[0], "White MG Rook behind PP mismatch");
        assert_eq!(eg_w_rpp - eg_w_base, pst_eg_w + ROOK_BEHIND_PASSED_PAWN_BONUS[1], "White EG Rook behind PP mismatch");

        // Check Black score difference includes PST + Bonus (W-B score)
        assert_eq!(mg_b_rpp - mg_b_base, -(pst_mg_b + ROOK_BEHIND_PASSED_PAWN_BONUS[0]), "Black MG Rook behind PP mismatch");
        assert_eq!(eg_b_rpp - eg_b_base, -(pst_eg_b + ROOK_BEHIND_PASSED_PAWN_BONUS[1]), "Black EG Rook behind PP mismatch");
    }

     #[test]
    fn test_rook_behind_enemy_passed_pawn() {
        let evaluator = PestoEval::new();
        // White rook R_d1 behind black passed pawn p_d3
        let board_w_rep = Board::new_from_fen("k7/8/8/8/K7/3p4/8/3R4 w - - 0 1").expect("Valid FEN");
        // Base without rook
        let board_w_base = Board::new_from_fen("k7/8/8/8/K7/3p4/8/8 w - - 0 1").expect("Valid FEN");

        let (mg_w_rep, eg_w_rep) = get_raw_scores(&evaluator, &board_w_rep);
        let (mg_w_base, eg_w_base) = get_raw_scores(&evaluator, &board_w_base);

        let d1_sq = board_utils::algebraic_to_sq_ind("d1");
        let pesto_instance = PestoEval::new();
        let pst_mg_w = pesto_instance.mg_table[WHITE][ROOK][d1_sq];
        let pst_eg_w = pesto_instance.eg_table[WHITE][ROOK][d1_sq];

        // Check White score difference includes PST + Bonus
        assert_eq!(mg_w_rep - mg_w_base, pst_mg_w + ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS[0], "White MG Rook behind Enemy PP mismatch");
        assert_eq!(eg_w_rep - eg_w_base, pst_eg_w + ROOK_BEHIND_ENEMY_PASSED_PAWN_BONUS[1], "White EG Rook behind Enemy PP mismatch");
    }

     #[test]
    fn test_rook_open_half_open_file() {
        let evaluator = PestoEval::new();
        // White rook R_d1 on open d-file
        let board_open = Board::new_from_fen("k7/8/8/8/8/8/P6P/K2R4 w - - 0 1").expect("Valid FEN");
        // White rook R_d1 on half-open d-file (enemy pawn on d7)
        let board_half_open = Board::new_from_fen("k7/3p4/8/8/8/8/P6P/K2R4 w - - 0 1").expect("Valid FEN");
        // Base without rook
        let board_base = Board::new_from_fen("k7/8/8/8/8/8/P6P/K7 w - - 0 1").expect("Valid FEN");

        let (mg_open, eg_open) = get_raw_scores(&evaluator, &board_open);
        let (mg_half, eg_half) = get_raw_scores(&evaluator, &board_half_open);
        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_base);

        let d1_sq = board_utils::algebraic_to_sq_ind("d1");
        let pesto_instance = PestoEval::new();
        let pst_mg = pesto_instance.mg_table[WHITE][ROOK][d1_sq];
        let pst_eg = pesto_instance.eg_table[WHITE][ROOK][d1_sq];

        // Check Open file bonus
        assert_eq!(mg_open - mg_base, pst_mg + ROOK_OPEN_FILE_BONUS[0], "White MG Rook Open File mismatch");
        assert_eq!(eg_open - eg_base, pst_eg + ROOK_OPEN_FILE_BONUS[1], "White EG Rook Open File mismatch");

        // Check Half-Open file bonus
        // Need PST of black d7 pawn
        let d7_sq = board_utils::algebraic_to_sq_ind("d7");
        let pst_mg_bp = -pesto_instance.mg_table[BLACK][PAWN][d7_sq]; // W-B score
        let pst_eg_bp = -pesto_instance.eg_table[BLACK][PAWN][d7_sq];
        // Score diff between half-open and base should be PST + HalfOpenBonus + BlackPawnPST
        assert_eq!(mg_half - mg_base, pst_mg + ROOK_HALF_OPEN_FILE_BONUS[0] + pst_mg_bp, "White MG Rook Half-Open File mismatch");
        assert_eq!(eg_half - eg_base, pst_eg + ROOK_HALF_OPEN_FILE_BONUS[1] + pst_eg_bp, "White EG Rook Half-Open File mismatch");
    }
}