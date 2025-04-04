#[cfg(test)]
mod tests {
    // Include the helper module
    #[path = "eval_test_utils.rs"]
    mod eval_test_utils;
    use eval_test_utils::get_raw_scores;

    use kingfisher::board::Board;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{
        PASSED_PAWN_BONUS_MG, PASSED_PAWN_BONUS_EG, ISOLATED_PAWN_PENALTY,
        PAWN_CHAIN_BONUS, PAWN_DUO_BONUS, MOBILE_PAWN_DUO_BONUS_MG, MOBILE_PAWN_DUO_BONUS_EG
    };
    use kingfisher::piece_types::{WHITE, BLACK, PAWN};
    use kingfisher::board_utils;

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
        let e6_sq = board_utils::algebraic_to_sq_ind("e6");
        let d3_sq = board_utils::algebraic_to_sq_ind("d3");
        let pesto_instance = PestoEval::new(); // Create instance to access tables
        let pst_mg_w = pesto_instance.mg_table[WHITE][PAWN][e6_sq];
        let pst_eg_w = pesto_instance.eg_table[WHITE][PAWN][e6_sq];
        let pst_mg_b = pesto_instance.mg_table[BLACK][PAWN][d3_sq];
        let pst_eg_b = pesto_instance.eg_table[BLACK][PAWN][d3_sq];


        // Check White score difference includes PST + Bonus
        assert_eq!(mg_w_p - mg_w_b, pst_mg_w + expected_bonus_mg_w, "White MG passed pawn bonus mismatch");
        assert_eq!(eg_w_p - eg_w_b, pst_eg_w + expected_bonus_eg_w, "White EG passed pawn bonus mismatch");

        // Check Black score difference includes PST + Bonus (remember raw score is W-B)
        assert_eq!(mg_b_p - mg_b_b, -(pst_mg_b + expected_bonus_mg_b), "Black MG passed pawn bonus mismatch");
        assert_eq!(eg_b_p - eg_b_b, -(pst_eg_b + expected_bonus_eg_b), "Black EG passed pawn bonus mismatch");

    }

    #[test]
    fn test_isolated_pawn_penalty() {
        let evaluator = PestoEval::new();
        // White has isolated d-pawn
        let board_w_iso = Board::new_from_fen("8/p7/k7/8/3P4/8/P1P1P1P1/K7 w - - 0 1").expect("Valid FEN");
        // Base without d-pawn
        let board_w_base = Board::new_from_fen("8/p7/k7/8/8/8/P1P1P1P1/K7 w - - 0 1").expect("Valid FEN");
        // Black has isolated e-pawn
        let board_b_iso = Board::new_from_fen("k7/p1p1p1p1/8/8/4p3/8/P7/K7 b - - 0 1").expect("Valid FEN");
        // Base without e-pawn
        let board_b_base = Board::new_from_fen("k7/p1p1p1p1/8/8/8/8/P7/K7 b - - 0 1").expect("Valid FEN");

        let (mg_w_iso, eg_w_iso) = get_raw_scores(&evaluator, &board_w_iso);
        let (mg_w_base, eg_w_base) = get_raw_scores(&evaluator, &board_w_base);
        let (mg_b_iso, eg_b_iso) = get_raw_scores(&evaluator, &board_b_iso);
        let (mg_b_base, eg_b_base) = get_raw_scores(&evaluator, &board_b_base);

        let d4_sq = board_utils::algebraic_to_sq_ind("d4");
        let e4_sq = board_utils::algebraic_to_sq_ind("e4");
        let pesto_instance = PestoEval::new();
        let pst_mg_w = pesto_instance.mg_table[WHITE][PAWN][d4_sq];
        let pst_eg_w = pesto_instance.eg_table[WHITE][PAWN][d4_sq];
        let pst_mg_b = pesto_instance.mg_table[BLACK][PAWN][e4_sq];
        let pst_eg_b = pesto_instance.eg_table[BLACK][PAWN][e4_sq];

        // Check White score difference includes PST + Penalty
        assert_eq!(mg_w_iso - mg_w_base, pst_mg_w + ISOLATED_PAWN_PENALTY[0], "White MG isolated pawn mismatch");
        assert_eq!(eg_w_iso - eg_w_base, pst_eg_w + ISOLATED_PAWN_PENALTY[1], "White EG isolated pawn mismatch");

        // Check Black score difference includes PST + Penalty (W-B score)
        assert_eq!(mg_b_iso - mg_b_base, -(pst_mg_b + ISOLATED_PAWN_PENALTY[0]), "Black MG isolated pawn mismatch");
        assert_eq!(eg_b_iso - eg_b_base, -(pst_eg_b + ISOLATED_PAWN_PENALTY[1]), "Black EG isolated pawn mismatch");
    }

     #[test]
    fn test_pawn_chain_bonus() {
        let evaluator = PestoEval::new();
        // White pawns c2-d3-e4 (2 chain links)
        let board_w_chain = Board::new_from_fen("k7/8/8/8/4P3/3P4/2P5/K7 w - - 0 1").expect("Valid FEN");
        // Base with only c2
        let board_w_base = Board::new_from_fen("k7/8/8/8/8/8/2P5/K7 w - - 0 1").expect("Valid FEN");

        let (mg_chain, eg_chain) = get_raw_scores(&evaluator, &board_w_chain);
        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_w_base);

        let d3_sq = board_utils::algebraic_to_sq_ind("d3");
        let e4_sq = board_utils::algebraic_to_sq_ind("e4");
        let pesto_instance = PestoEval::new();
        let pst_mg = pesto_instance.mg_table[WHITE][PAWN][d3_sq] + pesto_instance.mg_table[WHITE][PAWN][e4_sq];
        let pst_eg = pesto_instance.eg_table[WHITE][PAWN][d3_sq] + pesto_instance.eg_table[WHITE][PAWN][e4_sq];

        // d3 defends c2 (no), e4 defends d3 (yes), d3 defends e4 (no) -> 1 link counted at d3
        // c2 defends d3 (yes), d3 defends c2 (no) -> 1 link counted at c2
        // Total 2 links
        let expected_bonus_mg = 2 * PAWN_CHAIN_BONUS[0];
        let expected_bonus_eg = 2 * PAWN_CHAIN_BONUS[1];

        assert_eq!(mg_chain - mg_base, pst_mg + expected_bonus_mg, "White MG pawn chain mismatch");
        assert_eq!(eg_chain - eg_base, pst_eg + expected_bonus_eg, "White EG pawn chain mismatch");
    }

     #[test]
    fn test_pawn_duo_bonus() {
        let evaluator = PestoEval::new();
        // White pawns d4-e4 (1 duo pair)
        let board_w_duo = Board::new_from_fen("k7/8/8/8/3PP3/8/8/K7 w - - 0 1").expect("Valid FEN");
        // Base with only d4
        let board_w_base = Board::new_from_fen("k7/8/8/8/3P4/8/8/K7 w - - 0 1").expect("Valid FEN");

        let (mg_duo, eg_duo) = get_raw_scores(&evaluator, &board_w_duo);
        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_w_base);

        let e4_sq = board_utils::algebraic_to_sq_ind("e4");
        let pesto_instance = PestoEval::new();
        let pst_mg = pesto_instance.mg_table[WHITE][PAWN][e4_sq];
        let pst_eg = pesto_instance.eg_table[WHITE][PAWN][e4_sq];

        // Duo bonus applied once when checking d4's right neighbor (e4)
        let expected_bonus_mg = PAWN_DUO_BONUS[0];
        let expected_bonus_eg = PAWN_DUO_BONUS[1];

        assert_eq!(mg_duo - mg_base, pst_mg + expected_bonus_mg, "White MG pawn duo mismatch");
        assert_eq!(eg_duo - eg_base, pst_eg + expected_bonus_eg, "White EG pawn duo mismatch");
    }

    #[test]
    fn test_mobile_pawn_duo_bonus() {
        let evaluator = PestoEval::new();
        // White pawns d4-e4, squares d5, e5 empty
        let board_w_mobile = Board::new_from_fen("k7/8/8/8/3PP3/8/8/K7 w - - 0 1").expect("Valid FEN");
        // White pawns d4-e4, but black pawn on d5
        let board_w_blocked = Board::new_from_fen("k7/8/8/3p4/3PP3/8/8/K7 w - - 0 1").expect("Valid FEN");

        let (mg_mobile, eg_mobile) = get_raw_scores(&evaluator, &board_w_mobile);
        let (mg_blocked, eg_blocked) = get_raw_scores(&evaluator, &board_w_blocked);

        let d4_sq = board_utils::algebraic_to_sq_ind("d4"); // Bonus applied based on left pawn's square
        let pesto_instance = PestoEval::new();

        // Mobile bonus applied once when checking d4's right neighbor (e4)
        let expected_bonus_mg = MOBILE_PAWN_DUO_BONUS_MG[d4_sq];
        let expected_bonus_eg = MOBILE_PAWN_DUO_BONUS_EG[d4_sq];

        // Calculate PST diff for black d5 pawn
        let d5_sq = board_utils::algebraic_to_sq_ind("d5");
        let pst_diff_mg = -pesto_instance.mg_table[BLACK][PAWN][d5_sq]; // W-B score
        let pst_diff_eg = -pesto_instance.eg_table[BLACK][PAWN][d5_sq];

        // Difference between mobile and blocked should be the mobile bonus + PST of blocking pawn
        assert_eq!(mg_mobile - mg_blocked, expected_bonus_mg - pst_diff_mg, "White MG mobile duo mismatch");
        assert_eq!(eg_mobile - eg_blocked, expected_bonus_eg - pst_diff_eg, "White EG mobile duo mismatch");
    }
}