# Kingfisher Chess Engine

<img width="1038" alt="Kingfisher Chess Engine in action" src="https://github.com/aaholmes/chess/assets/4913443/ceab66cf-67c8-4685-bd28-d454c38ce756">

Kingfisher is a chess engine written in Rust, designed for research into combining classical alpha-beta search techniques with Monte Carlo Tree Search (MCTS).

The project aims to learn to play chess by self-play a la AlphaZero, but reduce the required compute by implementing a novel approach: performing a mate search before resorting to neural network evaluation.

## Research Focus

The primary goal of Kingfisher is to explore a hybrid approach that leverages the strengths of both traditional chess engines and modern AI techniques:

1.  **Mate Search First**: Before evaluating a position with a neural network, Kingfisher performs an exhaustive mate search, potentially replacing an expensive, noisy (and randomly initialized) neural network evaluation with an exact evaluation and corresponding best move from perfect play in the forced win.
2.  **Accelerated Training**: By identifying mate sequences early, we hypothesize that the engine can play much more effectively, especially at the start of training, by correctly evaluating positions where forced wins are possible.
3.  **Interpretable Evaluation**: Unlike the deep neural networks of AlphaZero, Kingfisher uses a simple, interpretable board evaluation function (Pesto), allowing for better insight into the engine's decision-making process and further reducing the required compute.

This research direction seeks to bridge the gap between traditional chess programming techniques and cutting-edge AI methods, potentially leading to more efficient and understandable chess AI systems.

## Features

*   **Board Representation:** Bitboards for efficient state representation.
*   **Move Generation:** Magic Bitboards for fast generation of sliding piece moves (rooks, bishops, queens), plus precomputed tables for other pieces. Handles castling, en passant, and promotions.
*   **Search:**
    *   Alpha-Beta search framework.
    *   Iterative Deepening.
    *   Transposition Tables (TT) for caching previously searched positions.
    *   Quiescence Search (QSearch) to mitigate the horizon effect.
    *   Null Move Pruning (NMP).
    *   Late Move Reductions (LMR).
    *   Aspiration Windows.
    *   Dedicated Mate Search function.
*   **Evaluation:** Pesto-style tapered evaluation using Piece-Square Tables (PSTs). Includes bonuses/penalties for Passed Pawns, Two Bishops, basic King Safety (pawn shield, castling rights), Pawn Structure (isolated, chains, duos, mobile duos), Rook positioning (doubled on 7th, behind friendly/enemy passed pawns, open/half-open files). Also includes MVV-LVA and heuristic move ordering.
*   **Protocol:** Basic UCI (Universal Chess Interface) support (`engine/src/uci.rs`).

## Installation

To use Kingfisher, you'll need Rust installed on your system. If you don't have Rust, you can install it from [rustup.rs](https://rustup.rs/).

1.  Clone the repository:
    ```bash
    git clone https://github.com/aaholmes/kingfisher.git
    cd kingfisher/engine
    ```
    *(Note: Adjusted `cd` path assuming the repo root contains the `engine` directory)*

2.  Build the project:
    ```bash
    cargo build --release
    ```

3.  Run the engine:
    The executable will be located at `target/release/engine` (relative to the `engine` directory).
    ```bash
    ./target/release/engine
    ```
    Or connect it to a UCI-compatible GUI.

## Project Structure

*   `src/board.rs`: Bitboard representation and operations (`Board` struct).
*   `src/board_utils.rs`: Utility functions for board coordinates and indices.
*   `src/boardstack.rs`: Manages the stack of board states for make/unmake.
*   `src/move_generation.rs`: Move generation including magic bitboards (`MoveGen` struct).
*   `src/magic_bitboard.rs`: Logic for initializing and using magic bitboards.
*   `src/magic_constants.rs`: Magic bitboard constants.
*   `src/eval.rs`: Position evaluation using Pesto function (`PestoEval` struct).
*   `src/eval_constants.rs`: Constants for the Pesto evaluation (PSTs, game phase increments).
*   `src/search.rs`: Negamax search with alpha-beta pruning, iterative deepening, etc.
*   `src/transposition.rs`: Transposition table implementation.
*   `src/make_move.rs`: Move execution and board state updates (integrated into `BoardStack`).
*   `src/uci.rs`: Handles the UCI protocol for communication with GUIs.
*   `src/utils.rs`: Utility functions (e.g., printing moves).
*   `src/bits.rs`: Low-level bitwise operations.
*   `src/move_types.rs`: Defines `Move` struct and related types (e.g., `CastlingRights`).
*   `src/piece_types.rs`: Constants for piece types and colors.
*   `src/hash.rs`: Zobrist hashing implementation.
*   `src/lib.rs`: Library root, declares modules.
*   `src/main.rs`: Entry point of the application.
*   `tests/`: Directory containing test files.

## Roadmap

### Completed Features
*   Bitboards
*   Move generation including magic bitboards
*   Negamax search / Alpha-beta pruning
*   Iterative deepening
*   Transposition table
*   MVV-LVA move ordering
*   Pawn and knight fork move ordering / Heuristic non-capture ordering
*   Mate search function
*   Quiescence search
*   Aspiration windows
*   Null move pruning (basic implementation enabled)
*   Late Move Reductions (LMR)
*   Evaluation Terms:
    *   Piece-Square Tables (PSTs)
    *   Passed Pawns
    *   Two Bishops Bonus
    *   King Safety (Pawn Shield, Castling Rights)
    *   Pawn Structure (Isolated, Chains, Duos, Mobile Duos)
    *   Rook Bonuses (Doubled on 7th, Behind Friendly/Enemy Passed Pawn, Open/Half-Open File)

### In Progress
*   Mate killer heuristic (mentioned in original roadmap)
*   UCI protocol (basic implementation exists)
*   Refining NMP (e.g., Zugzwang detection)

### Implementation Roadmap
*   **Improved Evaluation:** Add King Attack Score (including slider attacks), add more terms (e.g., open files near king), refine pawn structure logic, and tune all term weights.
*   **Time Management:** Implement robust time controls for UCI.
*   **Opening Book:** Integrate a standard opening book format.
*   **Endgame Tablebases:** Add support for querying tablebases (e.g., Syzygy).
*   **Parallel Search:** Explore parallelization techniques (e.g., Lazy SMP).
*   **Comprehensive Testing:** Expand test suite

### Research Directions
*   **Hybrid Search (Core Goal):** Integrate the classical search (especially mate search) with MCTS and a neural network evaluation/policy component.
*   **Neural Network Design:** Develop and train an interpretable neural network suitable for the hybrid approach.
*   **Style Mimicry:** Investigate training the NN component to emulate specific human or engine playing styles.
*   **Variant Exploration:** Apply and evaluate the hybrid approach in chess variants like King of the Hill.

## Inspirations

*   Monte Carlo Tree Search (MCTS)-based chess engines, such as AlphaZero and LeelaZero
*   Efficiently Updatable Neural Networks (NNUE) and state-of-the-art chess engines that combine it with classical search techniques, such as Stockfish and Berserk
*   The book "Neural Networks for Chess" by Dominik Klein

---

Kingfisher Chess Engine - Combining classical chess programming techniques with modern AI approaches.
