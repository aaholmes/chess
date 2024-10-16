//! UCI interface module for the chess engine
//!
//! This module implements the UCI interface for the chess engine. It allows the user to interact with the engine using UCI commands.

use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};
use crate::boardstack::BoardStack;
use crate::eval::PestoEval;
use crate::move_types::Move;
use crate::move_generation::MoveGen;
use crate::search::iterative_deepening_ab_search;

pub struct UCIEngine {
    board: BoardStack,
    move_gen: MoveGen,
    pesto: PestoEval,
    time_left: Duration,
    increment: Duration,
    moves_to_go: Option<u32>,
    depth: Option<i32>,
    nodes: Option<u64>,
    mate: Option<i32>,
    movetime: Option<Duration>,
}

impl UCIEngine {
    pub fn new() -> Self {
        UCIEngine {
            board: BoardStack::new(),
            move_gen: MoveGen::new(),
            pesto: PestoEval::new(),
            time_left: Duration::from_secs(0),
            increment: Duration::from_secs(0),
            moves_to_go: None,
            depth: None,
            nodes: None,
            mate: None,
            movetime: None,
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
        self.parse_go_command(args);

        let allocated_time = self.calculate_allocated_time();
        let start_time = Instant::now();

        let max_depth = self.depth.unwrap_or(100);
        let mut best_move = Move::null();
        let mut best_score = 0;

        let (depth, score, current_best_move, nodes) = iterative_deepening_ab_search(
            &mut self.board,
            &self.move_gen,
            &self.pesto,
            max_depth,
            4,
            Some(allocated_time),
            false
        );

        let elapsed = start_time.elapsed();

        // Update best move and score
        best_move = current_best_move;
        best_score = score;

        // Print info
        println!("info depth {} score cp {} nodes {} time {} pv {}",
                 depth, score, nodes, elapsed.as_millis(), &best_move.print_algebraic());

        println!("bestmove {}", &best_move.print_algebraic());
    }

    fn parse_go_command(&mut self, args: &[&str]) {
        self.time_left = Duration::from_secs(0);
        self.increment = Duration::from_secs(0);
        self.moves_to_go = None;
        self.depth = None;
        self.nodes = None;
        self.mate = None;
        self.movetime = None;

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "wtime" | "btime" => {
                    if (args[i] == "wtime" && self.board.current_state().w_to_move) ||
                        (args[i] == "btime" && !self.board.current_state().w_to_move) {
                        self.time_left = Duration::from_millis(args[i + 1].parse().unwrap_or(0));
                    }
                    i += 2;
                },
                "winc" | "binc" => {
                    if (args[i] == "winc" && self.board.current_state().w_to_move) ||
                        (args[i] == "binc" && !self.board.current_state().w_to_move) {
                        self.increment = Duration::from_millis(args[i + 1].parse().unwrap_or(0));
                    }
                    i += 2;
                },
                "movestogo" => {
                    self.moves_to_go = Some(args[i + 1].parse().unwrap_or(30));
                    i += 2;
                },
                "depth" => {
                    self.depth = Some(args[i + 1].parse().unwrap_or(100));
                    i += 2;
                },
                "nodes" => {
                    self.nodes = Some(args[i + 1].parse().unwrap_or(0));
                    i += 2;
                },
                "mate" => {
                    self.mate = Some(args[i + 1].parse().unwrap_or(0));
                    i += 2;
                },
                "movetime" => {
                    self.movetime = Some(Duration::from_millis(args[i + 1].parse().unwrap_or(0)));
                    i += 2;
                },
                _ => i += 1,
            }
        }
    }

    /// This function calculates the allocated time for a chess move based on the time control settings.
    ///
    /// Formula: time left per move until time control is reached (or 5% of time left if no time
    /// control is specified) + 50% of increment
    /// Defaults to 5 seconds if no time control is specified.
    fn calculate_allocated_time(&self) -> Duration {
        if let Some(movetime) = self.movetime {
            return movetime;
        }

        if self.time_left.as_millis() == 0 {
            return Duration::from_secs(5); // Default to 5 seconds if no time control is specified
        }

        let moves_left = self.moves_to_go.unwrap_or(20) as f32;
        let base_time = self.time_left.as_secs_f32() / moves_left;
        let bonus_time = self.increment.as_secs_f32();

        Duration::from_secs_f32(base_time + bonus_time * 0.5)
    }
}