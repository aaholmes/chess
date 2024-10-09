# Kingfisher Chess Engine

<img width="1038" alt="Kingfisher Chess Engine in action" src="https://github.com/aaholmes/chess/assets/4913443/ceab66cf-67c8-4685-bd28-d454c38ce756">

Kingfisher is a chess engine written in Rust, designed for research into combining classical alpha-beta search techniques with Monte Carlo Tree Search (MCTS).

The project aims to learn to play chess by self-play a la AlphaZero, but reduce the required compute by implementing a novel approach: performing a mate search before resorting to neural network evaluation.

## Research Focus

The primary goal of Kingfisher is to explore a hybrid approach that leverages the strengths of both traditional chess engines and modern AI techniques:

1. **Mate Search First**: Before evaluating a position with a neural network, Kingfisher performs an exhaustive mate search, potentially replacing an expensive, noisy (and randomly initialized) neural network evaluation with an exact evaluation and corresponding best move from perfect play in the forced win.

2. **Accelerated Training**: By identifying mate sequences early, we hypothesize that the engine can play much more effectively, especially at the start of training, by correctly evaluating positions where forced wins are possible.

3. **Interpretable Evaluation**: Unlike the deep neural networks of AlphaZero, Kingfisher uses a simple, interpretable board evaluation function, allowing for better insight into the engine's decision-making process and further reducing the required compute.


This research direction seeks to bridge the gap between traditional chess programming techniques and cutting-edge AI methods, potentially leading to more efficient and understandable chess AI systems.

## Features

- Bitboard representation for fast move generation and evaluation
- Magic bitboards for sliding piece move generation
- Negamax search with alpha-beta pruning
- Iterative deepening
- Pesto evaluation function (tapered)
- Advanced move ordering techniques
- Quiescence search
- Aspiration windows

## Installation

To use Kingfisher, you'll need Rust installed on your system. If you don't have Rust, you can install it from [rustup.rs](https://rustup.rs/).

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/kingfisher.git
   cd kingfisher
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the engine:
   ```
   cargo run --release
   ```

## Project Structure

- `src/bitboard.rs`: Bitboard representation and operations
  - Implements the `Bitboard` struct and associated methods
  - Includes utility functions for bitboard manipulation

- `src/gen_moves.rs`: Move generation including magic bitboards
  - Implements the `MoveGen` struct for move generation
  - Contains functions for generating legal moves for all piece types
  - Implements magic bitboard techniques for sliding piece move generation

- `src/gen_moves_arr.rs`: Alternative move generation implementation
  - Provides an array-based approach to move generation
  - May be used for performance comparison or specific use cases

- `src/eval.rs`: Position evaluation using Pesto function
  - Implements the positional evaluation function
  - Contains piece-square tables for both middlegame and endgame phases

- `src/search.rs`: Negamax search with alpha-beta pruning
  - Implements the main search algorithm
  - Includes iterative deepening, quiescence search, and aspiration windows

- `src/make_move.rs`: Move execution and board state updates
  - Contains functions to apply and undo moves on the board
  - Handles special moves like castling and en passant

- `src/utils.rs`: Utility functions
  - Includes helper functions for printing board state and moves
  - May contain other miscellaneous utility functions

- `src/bits.rs`: Low-level bitwise operations
  - Implements efficient bitwise manipulations used throughout the engine

- `src/magic_constants.rs`: Magic bitboard constants
  - Stores pre-computed magic numbers and related constants for magic bitboards

- `main.rs`: Entry point of the application
  - Sets up the engine and handles user interaction or UCI protocol

- `tests/`: Directory containing test files
  - Includes unit tests and integration tests for various components of the engine

## Roadmap

### Completed Features
- Bitboards
- Move generation including magic bitboards
- Negamax search
- Alpha-beta pruning
- Iterative deepening
- Pesto evaluation function (tapered)
- MVV-LVA move ordering
- Pawn and knight fork move ordering
- Non-captures ordered according to Pesto eval
- Mate search
- Quiescence search
- Aspiration windows

### In Progress
- Mate killer heuristic
- Transposition table
- Null move pruning
- UCI protocol

### Future Goals
- Interpretable neural network for evaluation function and move proposal probabilities
- Time management
- Opening book
- Endgame tablebases
- Parallel search

## Inspirations

- AlphaZero, of course
- Efficiently Updatable Neural Networks (NNUE) and state-of-the-art chess engines that combine it with classical search techniques, such as Stockfish and Berserk
- The book "Neural Networks for Chess" by Dominik Klein

---

Kingfisher Chess Engine - Combining classical chess programming techniques with modern AI approaches.
