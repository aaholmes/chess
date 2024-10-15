use kingfisher::board::Board;
use kingfisher::eval::PestoEval;
use kingfisher::move_generation::MoveGen;

#[test]
fn test_initial_move_count() {
    let board = Board::new();
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 20); // 16 pawn moves + 4 knight moves
}

#[test]
fn test_knight_moves() {
    let board = Board::new_from_fen("K7/8/k7/8/4N3/8/8/8 w - - 0 1");
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 11); // Knight should have 8 possible moves and king should have 3 (moving into check is OK for this function)
}

#[test]
fn test_pawn_promotion() {
    let board = Board::new_from_fen("1r6/P7/K7/8/k7/8/8/8 w - - 0 1");
    let move_gen = MoveGen::new();
    let (captures, moves) = move_gen.gen_pseudo_legal_moves(&board);
    assert_eq!(captures.len() + moves.len(), 12); // 4 promotions, 4 capture-promotions, 4 king moves (moving into check is OK for this function)
}

#[test]
fn test_capture_ordering() {
    let board = Board::new_from_fen("r1bqk2r/pp3Npp/2n1p1PP/1Pp5/3p4/3P1Q2/PP3PP1/R1B2RK1 w - - 0 1");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    board.print();
    let (mut captures, _) = move_gen.gen_pseudo_legal_moves_with_evals(&board, &pesto);

    let mut capture_vals: Vec<i32> = captures.iter().map(|m| move_gen.mvv_lva(&board, m.from, m.to)).collect();
    println!("{} Captures:", captures.len());
    for (i, m) in captures.iter().enumerate() {
        println!("{}. {} ({})", i+1, m, capture_vals[i]);
    }

    // Check that captures are ordered by MVV-LVA score in descending order
    for i in 1..captures.len() {
        assert!(capture_vals[i-1] >= capture_vals[i], "Moves not properly ordered at index {}", i);
    }
}

#[test]
fn test_non_capture_ordering() {
    let board = Board::new_from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4");
    let move_gen = MoveGen::new();
    let pesto = PestoEval::new();

    let (captures, non_captures) = move_gen.gen_pseudo_legal_moves_with_evals(&board, &pesto);

    board.print();

    println!("Captures:");
    for (i, m) in captures.iter().enumerate() {
        println!("{}. {} ({})", i+1, m, move_gen.mvv_lva(&board, m.from, m.to));
    }
    println!("Non-captures:");
    for (i, m) in non_captures.iter().enumerate() {
        println!("{}. {} ({})", i+1, m, pesto.move_eval(&board, &move_gen, m.from, m.to));
    }

    // Check that non-captures are ordered by Pesto eval change in descending order
    for i in 1..non_captures.len() {
        assert!(pesto.move_eval(&board, &move_gen, non_captures[i-1].from, non_captures[i-1].to) >= pesto.move_eval(&board, &move_gen, non_captures[i].from, non_captures[i].to),
                "Non-captures not properly ordered at index {}. {} vs {}",
                i, pesto.move_eval(&board, &move_gen, non_captures[i-1].from, non_captures[i-1].to), pesto.move_eval(&board, &move_gen, non_captures[i].from, non_captures[i].to));
    }
}