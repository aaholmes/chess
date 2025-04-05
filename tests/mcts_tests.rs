#[cfg(test)]
mod mcts_tests {
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::time::Duration;

    use kingfisher::board::Board;
    use kingfisher::move_generation::MoveGen;
    use kingfisher::mcts::{MctsNode, mcts_search, select_leaf, backpropagate, simulate_random_playout, EXPLORATION_CONSTANT};
    use kingfisher::move_types::{Move, NULL_MOVE};
    use kingfisher::board_utils; // For algebraic_to_sq_ind if needed

    // Helper to create basic setup
    fn setup() -> MoveGen {
        MoveGen::new()
    }

    // Helper to create a specific move (requires board state context usually)
    // Or use Move::new(from, to, promo) directly
    fn create_move(from: &str, to: &str) -> Move {
        let from_sq = board_utils::algebraic_to_sq_ind(from);
        let to_sq = board_utils::algebraic_to_sq_ind(to);
        Move::new(from_sq, to_sq, None)
    }

    // --- Node Tests ---

    #[test]
    fn test_node_new_root() {
        let move_gen = setup();
        let board = Board::new(); // Initial position
        let root_node_rc = MctsNode::new_root(board.clone(), &move_gen);
        let root_node = root_node_rc.borrow();

        assert!(root_node.parent.is_none());
        assert!(root_node.action.is_none());
        assert_eq!(root_node.visits, 0);
        assert_eq!(root_node.reward, 0.0);
        assert!(!root_node.is_terminal());
        assert!(!root_node.untried_actions.is_empty()); // Should have legal moves from start
        assert!(root_node.children.is_empty());
    }

     #[test]
    fn test_node_new_root_terminal() {
        let move_gen = setup();
        // Fool's Mate position (Black checkmated)
        let board = Board::new_from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 3").expect("Valid FEN");
        let root_node_rc = MctsNode::new_root(board.clone(), &move_gen);
        let root_node = root_node_rc.borrow();

        assert!(root_node.is_terminal());
        assert!(root_node.untried_actions.is_empty()); // No legal moves
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn test_node_uct_value() {
        let move_gen = setup();
        let board = Board::new();
        let root_node_rc = MctsNode::new_root(board.clone(), &move_gen);
        let mut root_node = root_node_rc.borrow_mut();
        root_node.visits = 10;
        root_node.reward = 7.0; // 7 wins out of 10 visits

        // Child 1: visited once, won once
        let child1_state = board.clone(); // Placeholder state
        let child1_action = create_move("e2", "e4");
        let child1_rc = MctsNode::new_child(child1_state, child1_action, Rc::downgrade(&root_node_rc), &move_gen);
        {
            let mut child1 = child1_rc.borrow_mut();
            child1.visits = 1;
            child1.reward = 1.0;
        }
        root_node.children.push(child1_rc.clone());

        // Child 2: visited 5 times, won 2 times (reward = 2.0)
        let child2_state = board.clone(); // Placeholder state
        let child2_action = create_move("d2", "d4");
        let child2_rc = MctsNode::new_child(child2_state, child2_action, Rc::downgrade(&root_node_rc), &move_gen);
         {
            let mut child2 = child2_rc.borrow_mut();
            child2.visits = 5;
            child2.reward = 2.0;
        }
        root_node.children.push(child2_rc.clone());

        // Child 3: unvisited
        let child3_state = board.clone(); // Placeholder state
        let child3_action = create_move("g1", "f3");
        let child3_rc = MctsNode::new_child(child3_state, child3_action, Rc::downgrade(&root_node_rc), &move_gen);
        root_node.children.push(child3_rc.clone());


        let parent_visits = root_node.visits;
        let c = EXPLORATION_CONSTANT;

        let uct1 = child1_rc.borrow().uct_value(parent_visits, c);
        let uct2 = child2_rc.borrow().uct_value(parent_visits, c);
        let uct3 = child3_rc.borrow().uct_value(parent_visits, c);

        let expected_uct1 = (1.0 / 1.0) + c * ((10.0f64.ln()) / 1.0).sqrt();
        let expected_uct2 = (2.0 / 5.0) + c * ((10.0f64.ln()) / 5.0).sqrt();

        assert_eq!(uct1, expected_uct1);
        assert_eq!(uct2, expected_uct2);
        assert_eq!(uct3, f64::INFINITY); // Unvisited nodes have infinite UCT

        // Test selection
        let best_child = root_node.select_best_child(c);
        assert_eq!(best_child.borrow().id, child3_rc.borrow().id); // Unvisited child should be selected
    }

    #[test]
    fn test_node_expand() {
        let move_gen = setup();
        let board = Board::new();
        let root_node_rc = MctsNode::new_root(board.clone(), &move_gen);

        assert!(!root_node_rc.borrow().is_fully_expanded());
        let initial_untried_count = root_node_rc.borrow().untried_actions.len();
        assert!(initial_untried_count > 0);
        assert!(root_node_rc.borrow().children.is_empty());

        // Expand one node
        // Note: Assumes apply_move_to_board works correctly
        let child1_rc = MctsNode::expand(root_node_rc.clone(), &move_gen);

        assert_eq!(root_node_rc.borrow().untried_actions.len(), initial_untried_count - 1);
        assert_eq!(root_node_rc.borrow().children.len(), 1);
        assert_eq!(root_node_rc.borrow().children[0].borrow().id, child1_rc.borrow().id);
        assert!(child1_rc.borrow().parent.is_some());
        assert!(child1_rc.borrow().action.is_some());

        // Check parent weak reference
        let parent_rc = child1_rc.borrow().parent.as_ref().unwrap().upgrade().unwrap();
        assert_eq!(parent_rc.borrow().id, root_node_rc.borrow().id);
    }

     #[test]
    fn test_node_backpropagate() {
        let move_gen = setup();
        let board = Board::new(); // White to move

        // Create a small tree: Root -> Child1 -> Child2
        let root_rc = MctsNode::new_root(board.clone(), &move_gen);
        let child1_action = create_move("e2", "e4");
        let child1_state = board.apply_move_to_board(child1_action); // Black to move
        let child1_rc = MctsNode::new_child(child1_state.clone(), child1_action, Rc::downgrade(&root_rc), &move_gen);
        root_rc.borrow_mut().children.push(child1_rc.clone());

        let child2_action = create_move("e7", "e5"); // Black move
        let child2_state = child1_state.apply_move_to_board(child2_action); // White to move
        let child2_rc = MctsNode::new_child(child2_state.clone(), child2_action, Rc::downgrade(&child1_rc), &move_gen);
        child1_rc.borrow_mut().children.push(child2_rc.clone());

        // Backpropagate a White win (result = 1.0) from Child2
        backpropagate(child2_rc.clone(), 1.0);

        // Check Child2 stats
        assert_eq!(child2_rc.borrow().visits, 1);
        assert_eq!(child2_rc.borrow().reward, 1.0); // White moved into this state, add White's score (1.0)

        // Check Child1 stats
        assert_eq!(child1_rc.borrow().visits, 1);
        assert_eq!(child1_rc.borrow().reward, 0.0); // Black moved into this state, add Black's score (1.0 - 1.0 = 0.0)

        // Check Root stats
        assert_eq!(root_rc.borrow().visits, 1);
        assert_eq!(root_rc.borrow().reward, 1.0); // White moved into this state, add White's score (1.0)

         // Backpropagate a Black win (result = 0.0) from Child2 again
        backpropagate(child2_rc.clone(), 0.0);

         // Check Child2 stats
        assert_eq!(child2_rc.borrow().visits, 2);
        assert_eq!(child2_rc.borrow().reward, 1.0 + 0.0); // White moved here, add White's score (0.0)

        // Check Child1 stats
        assert_eq!(child1_rc.borrow().visits, 2);
        assert_eq!(child1_rc.borrow().reward, 0.0 + 1.0); // Black moved here, add Black's score (1.0 - 0.0 = 1.0)

        // Check Root stats
        assert_eq!(root_rc.borrow().visits, 2);
        assert_eq!(root_rc.borrow().reward, 1.0 + 0.0); // White moved here, add White's score (0.0)
    }

    // --- Simulation Tests ---

    #[test]
    fn test_simulation_immediate_white_win() {
        let move_gen = setup();
        // Fool's Mate position (Black checkmated, White to move - White won)
        let board = Board::new_from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 3").expect("Valid FEN");
        let result = simulate_random_playout(&board, &move_gen);
        assert_eq!(result, 1.0); // White wins
    }

     #[test]
    fn test_simulation_immediate_black_win() {
        let move_gen = setup();
        // Position where White is checkmated, Black to move (Black won)
        let board = Board::new_from_fen("rnbqkbnr/ppppp2p/5p2/6pQ/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 0 3").expect("Valid FEN");
        let result = simulate_random_playout(&board, &move_gen);
        assert_eq!(result, 0.0); // Black wins (White score is 0.0)
    }

     #[test]
    fn test_simulation_immediate_stalemate() {
        let move_gen = setup();
        // Stalemate position, White to move
        let board = Board::new_from_fen("k7/8/8/8/8/5Q2/8/K7 w - - 0 1").expect("Valid FEN");
        let result = simulate_random_playout(&board, &move_gen);
        assert_eq!(result, 0.5); // Draw
    }

    // --- Integration Tests ---

    #[test]
    fn test_mcts_search_iterations() {
        let move_gen = setup();
        let board = Board::new();
        let iterations = 100; // Run a small number of iterations

        let best_move_opt = mcts_search(board, &move_gen, Some(iterations), None);

        assert!(best_move_opt.is_some(), "MCTS should return a move from the initial position");
        // Cannot easily assert the *specific* best move due to randomness,
        // but we can check it ran without panicking.
    }

     #[test]
    fn test_mcts_search_forced_mate_in_1() {
        let move_gen = setup();
        // White to move, mate in 1 (Qh5#)
        let board = Board::new_from_fen("r1bqkbnr/pppp1ppp/2n5/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 3").expect("Valid FEN");
        let iterations = 200; // More iterations to increase chance of finding mate quickly

        let best_move_opt = mcts_search(board, &move_gen, Some(iterations), None);
        let expected_move = create_move("h5", "f7"); // Qh5xf7#

        assert!(best_move_opt.is_some());
        assert_eq!(best_move_opt.unwrap(), expected_move, "MCTS failed to find mate in 1");
    }

     #[test]
    fn test_mcts_search_avoids_immediate_loss() {
        let move_gen = setup();
        // White to move. Moving King to b1 loses immediately to Qb2#. Any other King move is safe for now.
        // FEN: 8/8/k7/8/8/8/1q6/K7 w - - 0 1
        let board = Board::new_from_fen("8/8/k7/8/8/8/1q6/K7 w - - 0 1").expect("Valid FEN");
        let iterations = 300; // Give it some time to explore

        let losing_move = create_move("a1", "b1");
        let best_move_opt = mcts_search(board, &move_gen, Some(iterations), None);

        assert!(best_move_opt.is_some());
        assert_ne!(best_move_opt.unwrap(), losing_move, "MCTS chose a move leading to immediate loss");
    }

    // TODO: Add test for time limit termination
    // TODO: Add tests for more complex positions if possible (might require mocking simulation)

}