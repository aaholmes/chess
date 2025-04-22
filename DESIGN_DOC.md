# Design Document: Humanlike Chess Training Platform
Version: 1.2 (Reflecting Current Implementation Progress)

Date: 2025-04-23

Project Goal: To create a chess training platform featuring a unique "humanlike" chess engine for sparring and a hybrid repertoire builder. The engine should mimic the play style, common moves, and tactical alertness of human players within specific Elo rating bands (e.g., 2000-2200).

Target User: Chess players looking to improve by practicing against realistic opponents and building practical opening repertoires tailored to human responses.

## 1. High-Level Architecture

The platform comprises several interacting components:

*   **Core Engine (Rust):** A custom chess engine responsible for generating moves.
    *   *Status:* Foundation implemented in Rust (`src/`). Currently uses classical search (Alpha-Beta) and evaluation (Pesto), not the originally planned MCTS/Policy NN for the "humanlike" aspect. MCTS code exists (`src/mcts/`) but is not the primary search in `src/agent.rs`.
*   **Policy Network Models:** Neural network models (likely trained in Python, deployed in ONNX or TorchScript format) loaded by the Core Engine. Separate models exist for different target rating bands, providing move probabilities (policy).
    *   *Status:* Planned. Core engine integration (`src/mcts/policy.rs` exists) but loading/inference logic and connection to the main search (`src/search/` or `src/agent.rs`) is not apparent in the provided `src` files. Training pipeline is separate.
*   **Strong Engine (External):** An external, powerful UCI-compliant chess engine (e.g., Stockfish) used for objective analysis and suggesting strong moves for the user within the repertoire builder.
    *   *Status:* Planned. No integration code found in `src`.
*   **User Interface (UI):** The front-end application (technology TBD, potentially Web or Desktop) providing user interaction for sparring, repertoire building, and analysis.
    *   *Status:* Planned. No UI code found in `src`.
*   **Supporting Data Files:**
    *   Opening Books (Polyglot .bin): Rating-band specific books containing move frequencies from human games.
        *   *Status:* Planned. No loading/probing logic found in `src`.
    *   Endgame Tablebases (Syzygy .rtbm, .rtbw): 6-piece set for perfect mate resolution.
        *   *Status:* Planned. No probing logic found in `src`.
    *   User Data (Repertoires, Settings): Saved locally (e.g., PGN, JSON).
        *   *Status:* Planned.

*Interaction Flow Example (Sparring Move Request):*
*(Note: This flow describes the original MCTS/NN plan. The current implementation uses Alpha-Beta search as described below)*

1.  UI sends current board state (position command) and go command to Core Engine (via UCI protocol). *(UCI implemented in `src/uci.rs`)*
2.  Core Engine checks piece count: If <= 6 pieces -> Probes Syzygy EGTB (.rtbm file). If mate found, returns mate score/move immediately. *(EGTB probing planned)*
3.  If not EGTB mate, checks rating-specific Opening Book (Polyglot .bin). If move found & criteria met (e.g., >100 games in DB), selects move probabilistically based on stored weights. *(Opening book planned)*
4.  If not in book or book threshold not met, initiates MCTS search: *(Current implementation uses Alpha-Beta search instead)*
    *   Performs pre-expansion Deep Mate Search (configurable depth, e.g., 9-ply), unless EGTB already resolved mate. *(Mate search implemented in `src/search/mate_search.rs`)*
    *   Runs MCTS iterations: *(MCTS planned, Alpha-Beta implemented)*
        *   Node Selection: PUCT algorithm.
        *   Node Expansion: Prioritizes checks/captures (see Section 2.4), uses Policy NN priors for remaining moves.
        *   Simulation/Evaluation: Uses Heuristic Function (see Section 2.2).
        *   Backpropagation: Updates node statistics (visits N, value W).
    *   Selects best move based on MCTS results (e.g., highest visit count). *(Current implementation selects based on Alpha-Beta result)*
5.  Core Engine sends `bestmove <move>` back to UI. *(Implemented)*

## 2. Core Engine Design (Rust)

Primary Goal: Simulate human play at specific rating levels, incorporating realistic opening choices, tactical awareness, and characteristic move patterns. *(Note: Current implementation is a classical engine)*

Language: Rust *(Implemented)*

Core Libraries: `lazy_static`, `rand`. *(Implemented)* Potential future: `rust-syzygy`, ONNX/TorchScript runtime.

**2.1. Search Algorithm:** MCTS *(Planned)*
*   *Current Implementation:* Iterative Deepening Alpha-Beta Search (`src/search/iterative_deepening.rs`, `src/search/alpha_beta.rs`) with Transposition Tables (`src/transposition.rs`), Quiescence Search (`src/search/quiescence.rs`), History Heuristic (`src/search/history.rs`), and Static Exchange Evaluation (`src/search/see.rs`). MCTS code exists (`src/mcts/`) but is not the primary search used by `SimpleAgent`.

**2.2. Position Evaluation:** Heuristic Function *(Planned: Specific heuristic for MCTS)*
*   *Current Implementation:* Pesto-style Tapered Evaluation (`src/eval.rs`, `src/eval_constants.rs`). Includes material, PSQTs, king safety, pawn structure, mobility, bishop pair, rook bonuses.

**2.3. Move Guidance:** Policy Network Interface *(Planned)*
*   *Status:* No NN model loading/inference integrated into the main search/agent flow. `src/mcts/policy.rs` exists.

**2.4. Tactical Enhancements**
*   Deep Mate Search: Before root MCTS expansion (if not resolved by EGTB), perform dedicated search for forced mates (configurable depth, e.g., 9-ply). If mate found, use result directly.
    *   *Status:* Mate search implemented (`src/search/mate_search.rs`). Integration point (before MCTS/Alpha-Beta root) confirmed in `src/agent.rs`.
*   Check/Capture Prioritization (MCTS Expansion): *(Planned for MCTS)*
    *   *Current Implementation (Alpha-Beta):* Moves are generated (`src/move_generation.rs`) and sorted using MVV-LVA for captures and History Heuristic / Pesto evaluation difference for quiet moves (`gen_pseudo_legal_moves_with_evals`).

**2.5. Opening Book Integration** *(Planned)*
*   *Status:* No implementation found in `src`.

**2.6. Endgame Tablebase (EGTB) Integration** *(Planned)*
*   *Status:* No implementation found in `src`.

**2.7. Engine Communication Protocol:** UCI Subset *(Implemented)*
*   *Status:* Implemented in `src/uci.rs` via standard input/output. Handles `uci`, `isready`, `ucinewgame`, `position`, `go`, `quit`. Parses time controls and depth limits. Outputs `bestmove` and basic `info` string. Custom options are planned but not implemented in the handler.

## 3. Policy Network Subsystem *(Planned)*

Goal: Train NNs to predict human moves for specific rating bands.
Technology: Python with PyTorch or TensorFlow recommended for training.
*   *Status:* External to the Rust codebase. No direct evidence in `src`.

## 4. Strong Engine Integration (Rust) *(Planned)*

Implement a Rust module to manage an external UCI engine process (e.g., Stockfish executable).
*   *Status:* No implementation found in `src`.

## 5. User Interface (UI) Design *(Planned)*

Technology: TBD.
*   *Status:* No implementation found in `src`.

## 6. Technology Stack & Data Summary

*   **Core Engine:** Rust *(Implemented)*
*   **Policy NN Training:** Python (PyTorch/TensorFlow) *(Planned/External)*
*   **NN Model Format:** ONNX / TorchScript *(Planned)*
*   **EGTB:** Syzygy 6-piece (.rtbm, .rtbw) on local SSD *(Planned)*
*   **EGTB Library:** rust-syzygy *(Planned)*
*   **Opening Book:** Polyglot (.bin), per rating band, local storage *(Planned)*
*   **Strong Engine:** External UCI (e.g., Stockfish) *(Planned)*
*   **UI:** TBD *(Planned)*
*   **User Repertoires:** PGN (recommended), local storage *(Planned)*

## 7. Development Roadmap & Priorities

(Updated status based on `src` review)

*   **P1: Core Engine Foundation (Highest Priority)**
    *   Rust project setup, basic crate integration. *(DONE)*
    *   Implement MCTS framework structure. *(Partially DONE - `src/mcts/` exists, but Alpha-Beta is primary)*
    *   Implement Heuristic Evaluation (v1). *(DONE - Pesto implemented in `src/eval.rs`)*
    *   Implement Check/Capture prioritization logic. *(DONE - Implemented for Alpha-Beta move ordering)*
    *   Implement Deep Mate Search logic stub. *(DONE - Implemented in `src/search/mate_search.rs`)*
    *   Implement basic UCI handling (uci, isready, position, go, quit). *(DONE - Implemented in `src/uci.rs`)*
    *   *Current Goal Status:* Engine plays legal moves via UCI using Alpha-Beta search and Pesto evaluation.
*   **P2: Policy Network Pipeline (High Priority - Parallelizable)**
    *   Develop Python data processing scripts (Lichess PGN -> Training Data). *(External - Status Unknown)*
    *   Implement/adapt Policy NN architecture. *(External - Status Unknown)*
    *   Setup training pipeline. Train initial model (one rating band). *(External - Status Unknown)*
    *   Implement model export (ONNX/TorchScript). *(External - Status Unknown)*
    *   *Current Goal Status:* Unknown (External task).
*   **P3: Engine Integration (High Priority - Depends on P1 & P2)**
    *   Integrate NN model loading/inference in Rust. Connect to MCTS. *(TODO)*
    *   Integrate Syzygy EGTB probing (DTM mate resolution). *(TODO)*
    *   Integrate Polyglot Opening Book probing. *(TODO)*
    *   Initial tuning of MCTS parameters, heuristic function, mate search depth. *(Partially DONE - Tuning likely needed for Alpha-Beta/Pesto)*
    *   *Current Goal Status:* Basic Alpha-Beta engine structure exists. Integration of NN, EGTB, Book needed for original "humanlike" goal.
*   **P4: Strong Engine Integration (Medium Priority)**
    *   Implement Rust UCI wrapper for external engine. *(TODO)*
    *   *Current Goal Status:* Not started.
*   **P5: Basic UI / Test Harness (Medium Priority)**
    *   Develop a simple way to interact with the engine (CLI, test GUI, basic API). *(Partially DONE - UCI serves as a basic CLI interface, `src/main.rs` can run a simple game)*
    *   *Current Goal Status:* Basic interaction via UCI possible.
*   **P6: Full UI Feature Implementation (Lower Priority - Depends on P5)**
    *   Build out Sparring, Repertoire Builder, Analysis modules. *(TODO)*
    *   *Current Goal Status:* Not started.
*   **P7: Refinement & Testing (Ongoing, Increased Priority Later)**
    *   Extensive testing for "humanlike" feel..., bug fixing, performance tuning. *(Ongoing for current Alpha-Beta engine, major effort needed if switching to MCTS/NN)*
    *   Refine Heuristic function, MCTS parameters, potentially retrain Policy NN. *(Ongoing/Planned)*
    *   Documentation. *(Partially DONE - Doc comments exist)*
    *   *Current Goal Status:* Basic engine functional, requires significant testing and refinement, especially towards the "humanlike" goal.

## 8. Future Considerations

*   Training/Integrating a Value Network.
*   Subtle use of EGTB WDL scores in evaluation.
*   More rating band models.
*   Cloud deployment / Web service.
*   User accounts / Cloud storage.
*   Fully integrating or replacing Alpha-Beta with the MCTS implementation.
