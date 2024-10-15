#[cfg(test)]
mod tests {
    use kingfisher::boardstack::BoardStack;
    use kingfisher::move_types::Move;

    #[test]
    fn test_threefold_repetition() {
        let mut board = BoardStack::new(); // Assume this creates a standard starting position

        // Make some moves that will lead to a threefold repetition
        let moves = [
            "e2e4", "e7e5",
            "g1f3", "b8c6",
            "f1b5", "a7a6",
            "b5a4", "g8f6",
            "e1g1", "b7b5",
            "a4b3", "d7d6",
        ];

        for mv_str in moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        // Print out the position history
        for (hash, count) in board.position_history.iter() {
            println!("{}: {}", hash, count);
        }

        assert!(!board.is_draw_by_repetition(), "Should not be a draw yet");

        // Now make moves that repeat the position
        let repeating_moves = [
            "d1e2", "d8e7",
            "e2d1", "e7d8",
            "d1e2", "d8e7",
            "e2d1", "e7d8",
        ];

        for mv_str in repeating_moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        assert!(board.is_draw_by_repetition(), "Should be a draw by repetition");
    }

    #[test]
    fn test_repetition_with_different_castling_rights() {
        let mut board = BoardStack::new();

        // Set up a position where castling rights change
        let moves = [
            "e2e4", "e7e5",
            "g1f3", "b8c6",
            "f1b5", "a7a6",
            "b5a4", "g8f6",
            "e1g1", "b7b5",
            "a4b3", "d7d6",
            "d1e2", "d8e7",
            "e2d1", "e7d8", // Second time position appears
            "d1e2", "h8g8", // Black loses kingside castle rights
            "e2d1", "g8h8", // Third time position appears, but castle rights have changed so it's not a draw
            "d1e2", "d8e7",
            "e2d1", "e7d8", // Fourth time position appears (second with these castling rights)
            "d1e2", "a8b8", // Black loses queenside castle rights
            "e2d1", "b8a8", // Fifth time position appears, but castle rights have changed so it's not a draw
            "d1e2", "d8e7",
            "e2d1", "e7d8", // Sixth time position appears (second with these castling rights)
        ];

        for mv_str in moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        // Print out the position history
        for (hash, count) in board.position_history.iter() {
            println!("{}: {}", hash, count);
        }

        assert!(!board.is_draw_by_repetition(), "Should not be a draw due to different castling rights");

        // Finally repeat the position again
        let repeating_moves = [
            "d1e2", "d8e7",
            "e2d1", "e7d8",
        ];

        for mv_str in repeating_moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        assert!(board.is_draw_by_repetition(), "Should be a draw by repetition");
    }

    #[test]
    fn test_repetition_with_different_en_passant() {
        let mut board = BoardStack::new();

        // Set up a position where en passant changes
        let moves = [
            "e2e4", "e7e6",
            "e4e5", "d7d5", // En passant move available
            "d1e2", /* En passant rights lost */ "d8e7",
            "e2d1", "e7d8", // Second time position appears, but en passant has changed
            "d1e2", "d8e7",
            "e2d1", "e7d8", // Third time position appears, but en passant has changed so it's not a draw
        ];

        for mv_str in moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        assert!(!board.is_draw_by_repetition(), "Should not be a draw due to different en passant");

        // Finally repeat the position again
        let repeating_moves = [
            "d1e2", // Third time position appears, including en passant rights, so it is now a draw
        ];

        for mv_str in repeating_moves.iter() {
            let mv = Move::from_uci(mv_str).unwrap();
            board.make_move(mv);
        }

        assert!(board.is_draw_by_repetition(), "Should be a draw by repetition");
    }
}