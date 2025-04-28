# Chess Engine: Design Doc

Version: 1.2

Date: 2025-04-28

## 1. Introduction & Goals

This document outlines the design for a chess engine aimed at providing a human-like sparring experience, targeting players in a specific Elo range. The engine will feature an interpretable evaluation function, explore a novel combination of classical search and neural network guided Monte Carlo Tree Search (MCTS), and leverage existing chess knowledge to bootstrap its capabilities.

Key Goals:

*   **Humanlike Sparring Partner:** Create an engine whose style, opening choices, and (to some extent) strategic priorities resemble human play in a target Elo range (e.g. 2000-2200).
*   **Interpretable Evaluation:** Employ an evaluation function whose core components are understandable to humans, allowing for analysis and potential learning.
*   **Novel Architecture:** Explore interesting ways to combine classical search techniques (alpha-beta, quiescence) and modern neural network-guided MCTS.
*   **Leverage Chess Knowledge:** Incorporate established chess principles and classical engine techniques to reduce the learning burden on the NN and ensure baseline competence.

## 2. Core Architecture

The engine utilizes a Monte Carlo Tree Search (MCTS) algorithm as its core decision-making process.

*   **Search:** MCTS explores the game tree.
    *   *Status:* MCTS framework structure exists (`src/mcts/`), but the current primary search algorithm implemented and used by the `SimpleAgent` (`src/agent.rs`) is Iterative Deepening Alpha-Beta (`src/search/iterative_deepening.rs`, `src/search/alpha_beta.rs`).
*   **Guidance (Policy):** A Neural Network Policy (P) guides the MCTS exploration, prioritizing promising moves. This network will be trained on human games.
    *   *Status:* Planned. Policy network interface structure exists (`src/mcts/policy.rs`), but loading/inference logic and integration into the main search loop are not yet implemented. Training pipeline is external.
*   **Evaluation (Value):** A custom, enhanced static evaluation function (V_{enhanced}) provides position assessments for MCTS leaf nodes and informs Q-value updates. V_{enhanced} is based on an interpretable classical evaluation (E_{classical}) and a limited tactical search.
    *   *Status:* See Section 3.

## 3. Evaluation Subsystem (V_{enhanced})

The evaluation of positions within the MCTS relies on a two-tiered approach:

*   **E_{classical} (Classical Static Evaluation):**
    *   Purpose: Provides a fast, interpretable baseline evaluation.
    *   Components: Piece-Square Tables with adjustments for various features:
        * Two Bishops bonus
        * Pawn Structure (passed, isolated, backward pawns, pawn chains and duos)
        * King Safety heuristics
        * Mobility bonus
        * Rook placement bonus (open/half-open files, behind passed pawns, doubled on seventh)
    *   Tunability: Feature selection and weights can be tuned, potentially informed by analysis of the human training data.
    *   *Status:* Implemented as a Pesto-style Tapered Evaluation with adjustments in `src/eval.rs` and `src/eval_constants.rs`. This is currently used by the Alpha-Beta search.
*   **V_{enhanced} (Enhanced Static Evaluation via Quiescence Search):**
    *   Purpose: Provides a tactically robust evaluation for MCTS leaves by resolving immediate forcing lines.
    *   Algorithm: Node-limited, Alpha-Beta style quiescence search.
    *   Scope: Explores Checks, Captures, Promotions, and Escapes from Check (C+C+P+E).
    *   Leaf Evaluation: Uses E_{classical} at the leaves of the quiescence search (i.e., positions deemed "quiet" or where the node limit is hit).
    *   Termination: Tunable node count limit or reaching a quiet state.
    *   *Status:* Quiescence search implemented (`src/search/quiescence.rs`) and used by the current Alpha-Beta search. Static Exchange Evaluation (SEE) is also implemented (`src/search/see.rs`) for move ordering/pruning.

## 4. Search Algorithm (Per Move)

*   **Opening Book Check:**
    *   Check if the current board hash exists in the Human Opening Book database.
    *   If the position frequency >= threshold (e.g., 100 games): Probabilistically select a move based on human move frequencies from the database. Play the move and terminate search for this turn.
    *   If not in book or below threshold: Proceed to engine computation.
    *   *Status:* Planned. No opening book loading/probing logic found in `src`.
*   **Pre-computation (Root Node Only):**
    *   **Tablebase Lookup:** Check for endgame tablebase hits. Play mate/draw if found.
        *   *Status:* Planned. EGTB probing logic using `shakmaty-syzygy` is implemented in `src/egtb.rs`, but integration into the main search loop (`src/agent.rs`) is needed. Dependency added to `Cargo.toml`.
    *   **Mate Search:** Perform a node-limited Iterative Deepening search specifically for forced mates or forced sequences resulting in a tablebase win (e.g., M1, M2 within 3-ply, M3 within 5-ply, up to a node limit). Play mate or forced reaching of tablebase win if found.
        *   *Status:* Implemented (`src/search/mate_search.rs`) and integrated into the agent (`src/agent.rs`) before the main search.
*   **MCTS Execution (If no book/TB/mate result):** *(Note: Current implementation uses Alpha-Beta)*
    *   Budget: Run MCTS for a predetermined time or node count.
    *   Selection (Root Node): Use a modified PUCT (Polynomial Upper Confidence Trees) algorithm:
        *   Ensure at least N=1 visit for all legal root moves that are Checks, Captures, Promotions, or Forks (C+C+P+F).
        *   After forced exploration, revert to standard PUCT selection using Policy P and Q-values derived from V_{enhanced} for all legal moves.
    *   Selection (Tree Nodes): Use standard PUCT selection based on Policy P and stored Q-values (V_{enhanced}).
    *   Expansion: When a leaf node is reached, expand it using the NN Policy (P) to provide priors for child nodes.
    *   Evaluation (Simulation): Evaluate the newly expanded leaf node using V_{enhanced} (triggering the quiescence search).
    *   Backpropagation: Update Q-values (V_{enhanced}) and visit counts back up the selected path to the root.
    *   Final Move Selection: Choose the move from the root node based on MCTS statistics (typically the move with the highest visit count after the budget expires).
    *   *Status (Alpha-Beta):* Iterative Deepening Alpha-Beta search is used. Moves are generated (`src/move_generation.rs`) and sorted using MVV-LVA, History Heuristic (`src/search/history.rs`), and SEE. Transposition Tables (`src/transposition.rs`) are used. Final move selected based on Alpha-Beta result.
*   **Engine Communication:**
    *   *Status:* UCI protocol subset implemented (`src/uci.rs`) via standard input/output. Handles `uci`, `isready`, `ucinewgame`, `position`, `go`, `quit`. Parses time controls and depth limits. Outputs `bestmove` and basic `info` string.

## 5. Human-like Elements & Considerations

This design incorporates several elements to achieve human-like play:

*   **Training Data:** The core NN Policy (P) is trained via supervised learning on a large database of human games within the target Elo range. E_{classical} weights may also be tuned based on this data. *(Status: Training pipeline external, status unknown)*
*   **Opening Book:** Directly mimics human opening choices in frequently occurring positions. *(Status: Planned)*
*   **Root Search Heuristics:** The C+C+P+F priority and initial Mate Search mimic human focus on immediate tactical possibilities. *(Status: Mate search implemented. C+C+P+F planned for MCTS; current Alpha-Beta uses MVV-LVA/History/SEE for move ordering)*
*   **Evaluation Style:** E_{classical} uses understandable concepts. V_{enhanced} provides tactical robustness, avoiding simple blunders often found in pure static evaluation engines but perhaps less deep than full-width search. *(Status: E_classical (Pesto) and V_enhanced (Quiescence) implemented)*
*   **Tactical Acumen:** While trained on human data, the combination of MCTS, the dedicated Mate Search, the V_{enhanced} quiescence search, and the endgame tablebases will likely give the engine stronger tactical calculation and late endgame playing abilities than the average player in the training Elo range. This may result in an engine that plays positionally like the target group but is tactically sharper, making it a challenging and beneficial sparring partner. *(Note: Current Alpha-Beta implementation is already tactically strong)*

## 6. Training Strategy

*   **Data Source:** A large database of human games (e.g., Lichess Open Database, FICS games) filtered for players within the Elo range.
*   **Policy Network (P) Training:** Supervised Learning. Train the NN to predict the move played by the human player given the board state. Standard NN architectures for policy prediction (e.g., ResNet-based similar to AlphaZero/Leela) can be used.
*   **E_{classical} Tuning:** Weights for E_{classical} features are initialized using the PESTO evaluation function from Rofchade, with reasonable guesses for the other adjustments, then will be optimized using Texel Tuning, based on the human game dataset.
*   *Status:* Initial E_{classical} values hardcoded in `eval_constants.rs`. Training/tuning pipeline not started yet.

## 7. Opening Book Implementation

*   **Source:** Processed human game database (can be the same used for training).
*   **Format:** Hash table mapping position hash (e.g., Zobrist) to move frequencies. Polyglot (.bin) format planned.
*   **Logic:** On move request, calculate position hash. Query book. If hash exists and game count >= threshold (e.g., 100), sample a legal move according to the recorded human frequencies.
*   *Status:* Planned. No implementation found in `src`.

## 8. Technology Stack & Status Summary

*   **Core Engine:** Rust *(Implemented)*
    *   Libraries: `lazy_static`, `rand`, `shakmaty`, `shakmaty-syzygy` *(Implemented)*
    *   Search: Alpha-Beta with Iterative Deepening, Quiescence, TT, History, SEE *(Implemented)*. MCTS *(Partially Implemented, Inactive)*
    *   Evaluation: Pesto Tapered Eval with adjustments for pawn structure, king safety, etc. *(Implemented)*
    *   Mate Search: *(Implemented)*
    *   UCI Protocol: *(Implemented)*
*   **Policy NN Training:** Python (PyTorch/TensorFlow) *(Planned)*
*   **NN Model Format:** ONNX / TorchScript *(Planned)*
*   **EGTB:** Syzygy 6-piece (.rtbm, .rtbw) on local SSD *(Planned)*
*   **EGTB Library:** `shakmaty-syzygy` *(Implemented)*
*   **Opening Book:** Polyglot (.bin), per rating band, local storage *(Planned)*
*   **User Repertoires:** PGN (recommended), local storage *(Planned)*

**Current Overall Status:** The core engine foundation is built in Rust and functional, capable of playing chess via the UCI protocol using a classical Alpha-Beta search algorithm with Pesto evaluation. Key components like move generation, board representation, basic search enhancements (quiescence, TT, history, SEE), and mate search are implemented. The originally planned MCTS/NN architecture for "humanlike" play is partially present in the codebase (`src/mcts/`) but is not the active search method. The training pipeline for the Policy Network, integration of NN model loading/inference, opening book support, and full EGTB probing into the main search loop are the next major steps required to align with the original design goals.

## 9. Key Tunable Parameters

*   E_{classical} feature set and weights.
*   V_{enhanced} quiescence search node count limit.
*   Mate search node count limit / Iterative Deepening depths.
*   MCTS parameters: Exploration factor (C_{puct}), total budget (nodes/time per move).
*   Opening book frequency threshold.
*   NN Policy (P) architecture and training hyperparameters.

## 10. Future Considerations (Out of Scope for Initial Version)

*   **Strength Maximization:** While the current focus is human-like sparring, the architecture could be adapted for maximum playing strength, likely sacrificing interpretability. This probably involves:
    *   **Neural Network Evaluation:** Replacing the interpretable `E_classical` evaluation function with a neural network (e.g., NNUE or a deep value network) for significantly stronger positional understanding.
    *   **Self-Play Reinforcement Learning:** Training the policy and value networks via self-play (potentially after pre-training on human or engine games) allows the engine to surpass human knowledge and optimize for pure performance, similar to engines like AlphaZero or Leela Chess Zero.
    *   **Advanced Hybrid Evaluation:** An innovative but complex approach could involve using a deep network for primary strategic evaluation (`E_{classical}(deep NN)`) and adding a tactical correction derived from a separate, fast NNUE-powered quiescence search (`Correction = V_{enhanced}(NNUE) - E_{classical}(NNUE)`), feeding `V_final = E_{classical}(deep NN) + Correction` into the MCTS. This aims to combine deep strategic insight with robust tactical verification.

## 11. Summary

This chess engine design integrates NN-guided MCTS with classical search techniques and an interpretable evaluation function (V_{enhanced} based on E_{classical}). By training the NN policy on human data from the target Elo range and incorporating a human-derived opening book and specific search heuristics, the engine aims to provide a unique, human-like, and instructive sparring experience.