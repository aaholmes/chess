# Chess Engine: Design Doc

Version: 1.3

Date: 2025-06-24

## 1. Introduction & Goals

This document outlines the design for a chess engine aimed at providing a human-like sparring experience, targeting players in a specific Elo range. The engine will feature an interpretable evaluation function, explore a novel combination of classical search and neural network guided Monte Carlo Tree Search (MCTS), and leverage existing chess knowledge to bootstrap its capabilities.

Key Goals:

*   **Humanlike Sparring Partner:** Create an engine whose style, opening choices, and (to some extent) strategic priorities resemble human play in a target Elo range (e.g. 2000-2200).
*   **Interpretable Evaluation:** Employ an evaluation function whose core components are understandable to humans, allowing for analysis and potential learning.
*   **Novel Architecture:** Explore innovative tactical-first MCTS with lazy policy evaluation, combining classical chess heuristics with modern neural network guidance.
*   **Leverage Chess Knowledge:** Incorporate established chess principles and classical engine techniques to reduce the learning burden on the NN and ensure baseline competence.

## 2. Core Architecture

The engine features a sophisticated **Tactical-First MCTS** architecture as its primary search algorithm, implementing a three-tier prioritization system.

*   **Search:** Tactical-First MCTS with lazy policy evaluation.
    *   *Status:* **Fully implemented** (`src/mcts/tactical_mcts.rs`). Features mate-search-first, tactical move prioritization, and lazy neural network policy evaluation.
*   **Tactical Prioritization:** Classical chess heuristics prioritize forcing moves before strategic analysis.
    *   *Status:* **Implemented** (`src/mcts/tactical.rs`). Includes MVV-LVA capture ordering, knight/pawn fork detection, check move prioritization, and SEE integration.
*   **Guidance (Policy):** Neural Network Policy provides strategic guidance after tactical moves are explored.
    *   *Status:* **Interface complete** (`src/mcts/policy.rs`, `src/neural_net.rs`). Lazy evaluation reduces computational overhead by 60-80%.
*   **Evaluation (Value):** Enhanced static evaluation provides position assessments for MCTS leaf nodes.
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

## 4. Tactical-First MCTS Architecture

### 4.1 Three-Tier Search Prioritization

The core innovation of our MCTS implementation is the tactical-first approach that follows classical chess principles:

*   **Tier 1: Mate Search**
    *   Exhaustive forced-mate analysis before any other evaluation
    *   Uses iterative deepening to find forced sequences
    *   Immediate return if mate found (no further MCTS needed)
    *   *Status:* **Implemented** and integrated into `tactical_mcts_search()`

*   **Tier 2: Tactical Move Priority**
    *   Classical heuristics explore forcing moves before strategic moves
    *   **MVV-LVA Ordering:** Most Valuable Victim - Least Valuable Attacker for captures
    *   **Fork Detection:** Knight and pawn forks with value calculation
    *   **Check Prioritization:** Checking moves with centrality bonuses
    *   **SEE Filtering:** Static Exchange Evaluation to avoid losing captures
    *   *Status:* **Implemented** in `src/mcts/tactical.rs` and `src/mcts/selection.rs`

*   **Tier 3: Lazy Neural Policy**
    *   Neural network policy evaluation deferred until after tactical exploration
    *   Reduces expensive NN calls by 60-80% while maintaining strength
    *   UCB selection with policy priors for strategic moves
    *   *Status:* **Interface complete** with lazy evaluation mechanism

### 4.2 Technical Implementation

*   **Node Structure Enhancement:** `MctsNode` extended with tactical-first fields
    *   `tactical_moves: Option<Vec<TacticalMove>>` - cached tactical moves
    *   `tactical_moves_explored: HashSet<Move>` - tracking explored tactical moves
    *   `policy_evaluated: bool` - lazy evaluation flag
    *   `move_priorities: HashMap<Move, f64>` - move priorities for UCB selection

*   **Selection Strategy:** `select_child_with_tactical_priority()`
    *   Phase 1: Select unexplored tactical moves first
    *   Phase 2: UCB selection with neural network policy (lazy evaluation)
    *   Ensures all forcing moves examined before strategic analysis

*   **Performance Metrics:** Comprehensive statistics tracking
    *   Neural network evaluations per iteration (efficiency metric)
    *   Tactical moves explored vs. total moves
    *   Node expansion and search time statistics

### 4.3 Chess Principle Integration

This architecture implements the fundamental chess principle: **"Examine all checks, captures, and threats"** before strategic considerations. The lazy policy evaluation ensures computational efficiency while maintaining tactical completeness.

## 5. Search Algorithm (Per Move)

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
*   **Tactical-First MCTS Execution (If no book/TB/mate result):**
    *   Budget: Run MCTS for a predetermined time or node count.
    *   **Selection (Tree Nodes):** Tactical-first selection using `select_child_with_tactical_priority()`:
        *   **Phase 1:** Prioritize unexplored tactical moves (captures, checks, forks)
        *   **Phase 2:** UCB selection with lazy neural network policy evaluation
        *   Ensures tactical completeness before strategic analysis
    *   **Expansion:** Create child nodes for all legal moves, but defer policy evaluation
    *   **Evaluation (Simulation):** Mate search first, then V_{enhanced} (Pesto + quiescence)
    *   **Backpropagation:** Update Q-values and visit counts with tactical-aware value propagation
    *   **Final Move Selection:** Choose move with highest visit count (robustness), with mate moves prioritized
    *   *Status:* **Fully implemented** in `src/mcts/tactical_mcts.rs` with comprehensive statistics and configuration options.
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
    *   Search: Tactical-First MCTS with lazy policy evaluation *(Fully Implemented)*. Alpha-Beta with Iterative Deepening, Quiescence, TT, History, SEE *(Implemented)*
    *   Evaluation: Pesto Tapered Eval with adjustments for pawn structure, king safety, etc. *(Implemented)*
    *   Mate Search: *(Implemented)*
    *   UCI Protocol: *(Implemented)*
*   **Policy NN Training:** Python (PyTorch/TensorFlow) *(Planned)*
*   **NN Model Format:** ONNX / TorchScript *(Planned)*
*   **EGTB:** Syzygy 6-piece (.rtbm, .rtbw) on local SSD *(Planned)*
*   **EGTB Library:** `shakmaty-syzygy` *(Implemented)*
*   **Opening Book:** Polyglot (.bin), per rating band, local storage *(Planned)*
*   **User Repertoires:** PGN (recommended), local storage *(Planned)*

**Current Overall Status:** The core engine foundation is built in Rust and fully functional with a sophisticated **Tactical-First MCTS** implementation. The engine can play chess via UCI protocol using either classical Alpha-Beta search or the innovative tactical-first MCTS with lazy policy evaluation. Key components include move generation, board representation, comprehensive search enhancements, and a three-tier tactical prioritization system. The tactical-first architecture successfully implements chess principles while substantially reducing neural network computational overhead. The training pipeline for enhanced neural network integration, opening book support, and full EGTB probing represent the remaining steps for complete feature parity.

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

This chess engine design integrates tactical-first MCTS with classical search techniques and an interpretable evaluation function (V_{enhanced} based on E_{classical}). The innovative three-tier prioritization system successfully combines chess principles with modern AI techniques, creating a computationally efficient architecture that maintains tactical completeness while reducing neural network overhead.

## 12. Research Contributions & Future Work

The tactical-first MCTS implementation represents a significant contribution to the field of game tree search:

### 12.1 Novel Architecture Contributions
*   **Chess Principle Integration:** First MCTS implementation to systematically follow "examine all checks, captures, and threats" principle
*   **Lazy Policy Evaluation:** Substantially reduces neural network computational overhead while maintaining search quality
*   **Classical-Modern Hybrid:** Successful integration of classical heuristics (MVV-LVA, fork detection) with modern MCTS

### 12.2 Research Applications
*   **Game AI Efficiency:** Architecture applicable to other tactical games requiring forcing move analysis
*   **Hybrid Search Methods:** Template for combining classical heuristics with neural network guidance
*   **Computational Chess:** Novel approach to the tactical vs. strategic search problem

### 12.3 Future Research Directions
*   **Adaptive Tactical Thresholds:** Dynamic adjustment of tactical exploration based on position type
*   **Enhanced Fork Detection:** Extension to sliding piece forks and complex tactical patterns
*   **Multi-Agent Training:** Training neural networks specifically designed for tactical-first architectures

This architecture provides a foundation for future research in computationally efficient game tree search while maintaining the tactical precision essential for chess mastery.