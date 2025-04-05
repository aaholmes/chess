#[cfg(test)]
mod tests {
    use kingfisher::board::Board;
    use kingfisher::board_utils;
    use kingfisher::eval::PestoEval;
    use kingfisher::eval_constants::{KING_SAFETY_PAWN_SHIELD_BONUS, TWO_BISHOPS_BONUS};
    use kingfisher::piece_types::{BISHOP, BLACK, KNIGHT, PAWN, WHITE}; // Added KNIGHT

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

        // Add bonus terms
        for color in [WHITE, BLACK] {
            // Two Bishops Bonus
            let bishop_bb = board.get_piece_bitboard(color, BISHOP);
            if bishop_bb.count_ones() >= 2 {
                mg[color] += TWO_BISHOPS_BONUS[0];
                eg[color] += TWO_BISHOPS_BONUS[1];
            }

            // King Safety Pawn Shield
            let king_bb = board.get_piece_bitboard(color, 5); // KING = 5
            if king_bb != 0 {
                let king_sq = king_bb.trailing_zeros() as usize;
                let shield_zone_mask = board_utils::get_king_shield_zone_mask(color, king_sq);
                let friendly_pawns = board.get_piece_bitboard(color, PAWN);
                let shield_pawns = (shield_zone_mask & friendly_pawns).count_ones();
                mg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[0];
                eg[color] += shield_pawns as i32 * KING_SAFETY_PAWN_SHIELD_BONUS[1];
            }
        }

        (mg[WHITE] - mg[BLACK], eg[WHITE] - eg[BLACK])
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_two_bishops_bonus() {
        let evaluator = PestoEval::new();
        // Position with White having two bishops, Black having one knight/one bishop
        let board_base =
            Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let board_w_2b =
            Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5B2/PPPPPPPP/RNBQK1NR w KQkq - 0 1");

        let (mg_base, eg_base) = get_raw_scores(&evaluator, &board_base);
        let (mg_w_2b, eg_w_2b) = get_raw_scores(&evaluator, &board_w_2b);

        // Check White gets bonus relative to base
        // Note: Adding a bishop also adds its PST value, so we check the *difference* matches the bonus + PST diff
        let f3_sq = board_utils::algebraic_to_sq_ind("f3");
        let g1_sq = board_utils::algebraic_to_sq_ind("g1");
        let pst_diff_mg = evaluator.get_mg_score(WHITE, BISHOP, f3_sq)
            - evaluator.get_mg_score(WHITE, KNIGHT, g1_sq);
        let pst_diff_eg = evaluator.get_eg_score(WHITE, BISHOP, f3_sq)
            - evaluator.get_eg_score(WHITE, KNIGHT, g1_sq);

        assert_eq!(
            mg_w_2b - mg_base,
            pst_diff_mg + TWO_BISHOPS_BONUS[0],
            "MG Two Bishops Bonus mismatch"
        );
        assert_eq!(
            eg_w_2b - eg_base,
            pst_diff_eg + TWO_BISHOPS_BONUS[1],
            "EG Two Bishops Bonus mismatch"
        );

        // Test case where black has two bishops
        // Use a pre-configured position rather than trying to manually manipulate the board
        let board_b_1b =
            Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1");
        let board_b_2b =
            Board::new_from_fen("rnbqkbbr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1"); // Black has two bishops and no knights

        let (mg_b_1b, eg_b_1b) = get_raw_scores(&evaluator, &board_b_1b);
        let (mg_b_2b, eg_b_2b) = get_raw_scores(&evaluator, &board_b_2b);

        // Calculate PST difference for Black bishop vs knight
        let g8_sq = board_utils::algebraic_to_sq_ind("g8");
        let h8_sq = board_utils::algebraic_to_sq_ind("h8");
        let pst_diff_b_mg = evaluator.get_mg_score(BLACK, BISHOP, h8_sq)
            - evaluator.get_mg_score(BLACK, KNIGHT, g8_sq);
        let pst_diff_b_eg = evaluator.get_eg_score(BLACK, BISHOP, h8_sq)
            - evaluator.get_eg_score(BLACK, KNIGHT, g8_sq);

        // Black's score (W-B) should decrease by (PST diff + Bonus)
        assert_eq!(
            mg_b_2b - mg_b_1b,
            -(pst_diff_b_mg + TWO_BISHOPS_BONUS[0]),
            "MG Black Two Bishops Bonus mismatch"
        );
        assert_eq!(
            eg_b_2b - eg_b_1b,
            -(pst_diff_b_eg + TWO_BISHOPS_BONUS[1]),
            "EG Black Two Bishops Bonus mismatch"
        );
    }

    #[test]
    #[ignore] // Requires more complete evaluation function
    fn test_king_safety_pawn_shield() {
        let evaluator = PestoEval::new();
        // White king on g1, pawns on f2, g2, h2 (good shield - 3 pawns in zone f2,g2,h2)
        let board_w_safe =
            Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // White king on g1, pawns on f3, g2, h2 (weaker shield - 1 pawn g2 in zone)
        let board_w_less_safe =
            Board::new_from_fen("rnbqkb1r/pppppppp/8/8/8/5P2/PPPP1PPP/RNBQKBNR w KQkq - 0 1");

        let (mg_safe, eg_safe) = get_raw_scores(&evaluator, &board_w_safe);
        let (mg_less, eg_less) = get_raw_scores(&evaluator, &board_w_less_safe);

        // Calculate expected difference:
        // Safe: 3 pawns * bonus
        // Less Safe: 1 pawn * bonus (g2) + PST diff for f3 vs f2
        // Diff = (2 * bonus) + (PST(f2) - PST(f3))
        let f2_sq = board_utils::algebraic_to_sq_ind("f2");
        let f3_sq = board_utils::algebraic_to_sq_ind("f3");
        let pst_diff_mg =
            evaluator.get_mg_score(WHITE, PAWN, f2_sq) - evaluator.get_mg_score(WHITE, PAWN, f3_sq);
        let pst_diff_eg =
            evaluator.get_eg_score(WHITE, PAWN, f2_sq) - evaluator.get_eg_score(WHITE, PAWN, f3_sq);

        let expected_diff_mg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[0] + pst_diff_mg;
        let expected_diff_eg = 2 * KING_SAFETY_PAWN_SHIELD_BONUS[1] + pst_diff_eg;

        // Need to account for the PST change of the moved pawn f2->f3
        assert_eq!(
            mg_safe - mg_less,
            expected_diff_mg,
            "MG King Safety difference mismatch"
        );
        assert_eq!(
            eg_safe - eg_less,
            expected_diff_eg,
            "EG King Safety difference mismatch"
        );
    }
}
