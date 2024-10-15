use std::io::{self, BufRead, Write};
use crate::board::Board;
use crate::boardstack::BoardStack;
use crate::move_generation::MoveGen;
use crate::eval::PestoEval;
use crate::move_types::Move;
use crate::search::iterative_deepening_ab_search;

pub struct UCIEngine {
    board: BoardStack,
    move_gen: MoveGen,
    pesto: PestoEval,
}

impl UCIEngine {
    pub fn new() -> Self {
        UCIEngine {
            board: BoardStack::new(),
            move_gen: MoveGen::new(),
            pesto: PestoEval::new(),
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input = line.unwrap();
            let tokens: Vec<&str> = input.split_whitespace().collect();

            if tokens.is_empty() {
                continue;
            }

            match tokens[0] {
                "uci" => {
                    println!("id name Kingfisher");
                    println!("id author Adam Holmes");
                    println!("uciok");
                },
                "isready" => println!("readyok"),
                "ucinewgame" => self.board = BoardStack::new(),
                "position" => self.handle_position(&tokens[1..]),
                "go" => self.handle_go(&tokens[1..]),
                "quit" => break,
                _ => println!("Unknown command: {}", tokens[0]),
            }

            io::stdout().flush().unwrap();
        }
    }

    fn handle_position(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        if args[0] == "startpos" {
            self.board = BoardStack::new();
            let mut moves_idx = 2;

            if args.len() > 1 && args[1] == "moves" {
                for move_str in &args[moves_idx..] {
                    if let Some(chess_move) = Move::from_uci(move_str) {
                        self.board.make_move(chess_move);
                    }
                }
            }
        } else if args[0] == "fen" {
            // Find the index where "moves" starts, if present
            let moves_idx = args.iter().position(|&x| x == "moves").unwrap_or(args.len());

            // Join the FEN parts
            let fen = args[1..moves_idx].join(" ");

            // Create a new board from the FEN
            self.board = BoardStack::new_from_fen(&fen);

            // Apply moves if present
            if moves_idx < args.len() {
                for move_str in &args[moves_idx + 1..] {
                    if let Some(chess_move) = Move::from_uci(move_str) {
                        self.board.make_move(chess_move);
                    }
                }
            }
        } else {
            println!("info string Invalid position command");
        }
    }

    fn handle_go(&mut self, args: &[&str]) {
        let mut depth = 6; // Default depth
        for i in 0..args.len() {
            if args[i] == "depth" && i + 1 < args.len() {
                depth = args[i + 1].parse().unwrap_or(6);
                break;
            }
        }

        let (score, best_move, _) = iterative_deepening_ab_search(&mut self.board, &self.move_gen, &self.pesto, depth, 4, false);

        println!("info score cp {}", score);
        println!("bestmove {}", &best_move.print_algebraic());
    }
}