#[cfg(test)]
mod tests {
    use kingfisher::board::Board;
    use kingfisher::board_utils;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{
        ISOLATED_PAWN_PENALTY, MOBILE_PAWN_DUO_BONUS_EG, MOBILE_PAWN_DUO_BONUS_MG,
        PASSED_PAWN_BONUS_EG, PASSED_PAWN_BONUS_MG, PAWN_CHAIN_BONUS, PAWN_DUO_BONUS,
    };
    use kingfisher::piece_types::{BLACK, PAWN, WHITE};

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

        // Return raw scores (White - Black)
        (mg[WHITE] - mg[BLACK], eg[WHITE] - eg[BLACK])
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_passed_pawn_bonus() {
        let evaluator = PestoEval::new();
        // White passed pawn on e5
        let board_w_passed = Board::new_from_fen("k7/8/8/4P3/8/8/8/K7 w - - 0 1");
        // Base position without the pawn
        let board_base = Board::new_from_fen("k7/8/8/8/8/8/8/K7 w - - 0 1");

        let (mg_w_passed, eg_w_passed) = get_raw_scores(&evaluator, &board_w_passed);
        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_base);

        // Calculate PST value for the pawn
        let e5_sq = board_utils::algebraic_to_sq_ind("e5");
        let pst_mg_w = evaluator.get_mg_score(WHITE, PAWN, e5_sq);
        let pst_eg_w = evaluator.get_eg_score(WHITE, PAWN, e5_sq);

        // Check White score difference includes PST + Passed Pawn Bonus
        // For a pawn on rank 5 (bonus_rank = 4)
        assert_eq!(
            mg_w_passed - mg_base,
            pst_mg_w + PASSED_PAWN_BONUS_MG[4],
            "White MG Passed Pawn bonus mismatch"
        );
        assert_eq!(
            eg_w_passed - eg_base,
            pst_eg_w + PASSED_PAWN_BONUS_EG[4],
            "White EG Passed Pawn bonus mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_isolated_pawn_penalty() {
        let evaluator = PestoEval::new();
        // White isolated pawn on e4
        let board_w_isolated = Board::new_from_fen("k7/8/8/8/4P3/8/8/K7 w - - 0 1");
        // White connected pawns on d4, e4
        let board_w_connected = Board::new_from_fen("k7/8/8/8/3PP3/8/8/K7 w - - 0 1");

        let (mg_isolated, eg_isolated) = get_raw_scores(&evaluator, &board_w_isolated);
        let (mg_connected, eg_connected) = get_raw_scores(&evaluator, &board_w_connected);

        // Calculate difference between isolated and connected positions
        // Isolated pawn gets a penalty, while connected doesn't
        // Need to account for PST of d4 pawn in the connected position
        let d4_sq = board_utils::algebraic_to_sq_ind("d4");
        let pst_mg_d4 = evaluator.get_mg_score(WHITE, PAWN, d4_sq);
        let pst_eg_d4 = evaluator.get_eg_score(WHITE, PAWN, d4_sq);

        // Difference should be: connected - isolated = pst_d4 - ISOLATED_PENALTY
        assert_eq!(
            mg_connected - mg_isolated,
            pst_mg_d4 - ISOLATED_PAWN_PENALTY[0],
            "Isolated pawn penalty MG mismatch"
        );
        assert_eq!(
            eg_connected - eg_isolated,
            pst_eg_d4 - ISOLATED_PAWN_PENALTY[1],
            "Isolated pawn penalty EG mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_pawn_chain_bonus() {
        let evaluator = PestoEval::new();
        // White pawn chain: e4 supported by d3
        let board_w_chain = Board::new_from_fen("k7/8/8/8/4P3/3P4/8/K7 w - - 0 1");
        // White pawns not in chain: e4, a3
        let board_w_no_chain = Board::new_from_fen("k7/8/8/8/4P3/P7/8/K7 w - - 0 1");

        let (mg_chain, eg_chain) = get_raw_scores(&evaluator, &board_w_chain);
        let (mg_no_chain, eg_no_chain) = get_raw_scores(&evaluator, &board_w_no_chain);

        // Calculate difference between chain and no-chain positions
        // Need to account for PST differences (d3 vs a3)
        let d3_sq = board_utils::algebraic_to_sq_ind("d3");
        let a3_sq = board_utils::algebraic_to_sq_ind("a3");
        let pst_mg_diff =
            evaluator.get_mg_score(WHITE, PAWN, d3_sq) - evaluator.get_mg_score(WHITE, PAWN, a3_sq);
        let pst_eg_diff =
            evaluator.get_eg_score(WHITE, PAWN, d3_sq) - evaluator.get_eg_score(WHITE, PAWN, a3_sq);

        // Chain position should have PAWN_CHAIN_BONUS more than no-chain position
        assert_eq!(
            mg_chain - mg_no_chain,
            pst_mg_diff + PAWN_CHAIN_BONUS[0],
            "Pawn chain bonus MG mismatch"
        );
        assert_eq!(
            eg_chain - eg_no_chain,
            pst_eg_diff + PAWN_CHAIN_BONUS[1],
            "Pawn chain bonus EG mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_pawn_duo_bonus() {
        let evaluator = PestoEval::new();
        // White pawns d4-e4 (1 duo pair)
        let board_w_duo = Board::new_from_fen("k7/8/8/8/3PP3/8/8/K7 w - - 0 1");
        // Base with only d4
        let board_w_base = Board::new_from_fen("k7/8/8/8/3P4/8/8/K7 w - - 0 1");

        let (mg_duo, eg_duo) = get_raw_scores(&evaluator, &board_w_duo);
        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_w_base);

        let e4_sq = board_utils::algebraic_to_sq_ind("e4");
        let pst_mg = evaluator.get_mg_score(WHITE, PAWN, e4_sq);
        let pst_eg = evaluator.get_eg_score(WHITE, PAWN, e4_sq);

        // Duo bonus applied once when checking d4's right neighbor (e4)
        let expected_bonus_mg = PAWN_DUO_BONUS[0];
        let expected_bonus_eg = PAWN_DUO_BONUS[1];

        assert_eq!(
            mg_duo - mg_base,
            pst_mg + expected_bonus_mg,
            "White MG pawn duo mismatch"
        );
        assert_eq!(
            eg_duo - eg_base,
            pst_eg + expected_bonus_eg,
            "White EG pawn duo mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_mobile_pawn_duo_bonus() {
        let evaluator = PestoEval::new();
        // White pawns d4-e4, squares d5, e5 empty
        let board_w_mobile = Board::new_from_fen("k7/8/8/8/3PP3/8/8/K7 w - - 0 1");
        // White pawns d4-e4, but black pawn on d5
        let board_w_blocked = Board::new_from_fen("k7/8/8/3p4/3PP3/8/8/K7 w - - 0 1");

        let (mg_mobile, eg_mobile) = get_raw_scores(&evaluator, &board_w_mobile);
        let (mg_blocked, eg_blocked) = get_raw_scores(&evaluator, &board_w_blocked);

        let d4_sq = board_utils::algebraic_to_sq_ind("d4"); // Bonus applied based on left pawn's square

        // Mobile bonus applied once when checking d4's right neighbor (e4)
        let expected_bonus_mg = MOBILE_PAWN_DUO_BONUS_MG[d4_sq];
        let expected_bonus_eg = MOBILE_PAWN_DUO_BONUS_EG[d4_sq];

        // Calculate PST diff for black d5 pawn
        let d5_sq = board_utils::algebraic_to_sq_ind("d5");
        let pst_diff_mg = -evaluator.get_mg_score(BLACK, PAWN, d5_sq); // W-B score
        let pst_diff_eg = -evaluator.get_eg_score(BLACK, PAWN, d5_sq);

        // Difference between mobile and blocked should be the mobile bonus + PST of blocking pawn
        assert_eq!(
            mg_mobile - mg_blocked,
            expected_bonus_mg - pst_diff_mg,
            "White MG mobile duo mismatch"
        );
        assert_eq!(
            eg_mobile - eg_blocked,
            expected_bonus_eg - pst_diff_eg,
            "White EG mobile duo mismatch"
        );
    }
}
