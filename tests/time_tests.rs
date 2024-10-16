#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use kingfisher::boardstack::BoardStack;
    use kingfisher::eval::PestoEval;
    use kingfisher::move_generation::MoveGen;
    use kingfisher::search::iterative_deepening_ab_search;

    #[test]
    fn test_time_management_short_duration() {
        let mut board = BoardStack::new();
        let move_gen = MoveGen::new();
        let pesto = PestoEval::new();
        let max_depth = 10;
        let q_search_max_depth = 5;
        let time_limit = Some(Duration::from_millis(60)); // Very short time limit

        let start = Instant::now();
        let (depth, _, _, _) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, time_limit, false);
        let elapsed = start.elapsed();

        println!("Searched depth: {}", depth);

        assert!(elapsed <= (15 * time_limit.unwrap()) / 10, "Search took too long: {:?}", elapsed);
    }

    #[test]
    fn test_time_management_longer_duration() {
        let mut board = BoardStack::new();
        let move_gen = MoveGen::new();
        let pesto = PestoEval::new();
        let max_depth = 20;
        let q_search_max_depth = 5;
        let time_limit = Some(Duration::from_secs(2));

        let start = Instant::now();
        let (depth, _, _, _) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, time_limit, false);
        let elapsed = start.elapsed();

        println!("Searched depth: {}", depth);
        assert!(elapsed <= (15 * time_limit.unwrap()) / 10, "Search took too long: {:?}", elapsed);
    }

    #[test]
    fn test_time_management_completes_minimum_depth() {
        let mut board = BoardStack::new();
        let move_gen = MoveGen::new();
        let pesto = PestoEval::new();
        let max_depth = 5;
        let q_search_max_depth = 3;
        let time_limit = Some(Duration::from_secs(10)); // Generous time limit

        let (depth, _, _, nodes) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, time_limit, false);

        println!("Searched depth: {}", depth);

        // Ensure that at least the minimum depth was searched
        assert!(nodes > 0, "Search did not complete minimum depth");
    }

    #[test]
    fn test_time_management_uses_available_time() {
        let mut board = BoardStack::new();
        let move_gen = MoveGen::new();
        let pesto = PestoEval::new();
        let max_depth = 20;
        let q_search_max_depth = 5;
        let time_limit = Some(Duration::from_secs(1));

        let start = Instant::now();
        let (depth, _, _, _) = iterative_deepening_ab_search(&mut board, &move_gen, &pesto, max_depth, q_search_max_depth, time_limit, false);
        let elapsed = start.elapsed();

        println!("Searched depth: {}", depth);

        assert!(elapsed >= (9 * time_limit.unwrap()) / 10, "Search finished too quickly: {:?}", elapsed);
    }
}