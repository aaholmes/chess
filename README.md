# Kingfisher Chess Engine
## Tactical-Enhanced MCTS with Lazy Policy Evaluation

<img width="1038" alt="Kingfisher Chess Engine in action" src="https://github.com/aaholmes/chess/assets/4913443/ceab66cf-67c8-4685-bd28-d454c38ce756">

A chess engine implementing a novel three-tier search architecture that systematically prioritizes tactical moves before neural network evaluation, combining classical chess principles with modern Monte Carlo Tree Search.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Architecture

### Three-Tier Search Prioritization

**Tier 1: Mate Search**
- Forced mate analysis at each leaf node (configurable depth 1-5)
- Immediate termination when mate sequences found
- Zero neural network overhead for tactical positions

**Tier 2: Tactical Move Priority**
- MVV-LVA capture ordering with piece value calculations
- Knight and pawn fork detection algorithms
- Check move identification and prioritization
- Classical heuristics applied before expensive NN evaluation

**Tier 3: Lazy Neural Policy Evaluation**
- Neural network calls deferred until after tactical exploration
- UCB selection with policy priors for strategic moves
- PyTorch integration with ResNet architecture

```
MCTS Tree Traversal
       │
       ▼
┌─────────────────────────────────────┐
│  Mate Search (depth 3-5)           │
│  ├── Return if mate found           │
│  └── Continue if no mate            │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│  Tactical Move Exploration          │
│  ├── MVV-LVA captures              │
│  ├── Fork detection                 │
│  ├── Check moves                    │
│  └── Mark moves as explored         │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│  Neural Policy Evaluation          │
│  ├── Policy network inference       │
│  ├── UCB child selection           │
│  └── Strategic move analysis        │
└─────────────────────────────────────┘
```

## Features

The tactical-enhanced search demonstrates:
- Systematic mate detection at configurable search depths
- Tactical move prioritization (captures, checks, forks)
- Neural network integration with lazy evaluation
- Comprehensive validation across tactical test positions

## Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For neural network features (optional)
pip install torch numpy python-chess
```

### Build and Test
```bash
git clone https://github.com/yourusername/kingfisher.git
cd kingfisher/engine
cargo build --release

# Run tactical benchmark
cargo run --release --bin tactical_benchmark

# Performance profiling
cargo run --release --bin tactical_profiler

# Full validation suite
./run_validation_experiments.sh
```

### Example Usage
```bash
# Tactical search demonstration
cargo run --release --bin tactical_benchmark

# Performance analysis
cargo run --release --bin tactical_profiler

# Neural network integration test
cargo run --bin neural_test
```

## Implementation

### Core Components

**`src/mcts/tactical_mcts.rs`** - Main search algorithm
- Three-tier node selection with statistics tracking
- Configurable mate search depth and time limits
- Performance monitoring and cache management

**`src/mcts/tactical.rs`** - Tactical move detection
- MVV-LVA scoring with piece value tables
- Knight fork detection using attack patterns
- Pawn fork identification with target analysis
- Check move classification with king safety

**`src/mcts/selection.rs`** - Lazy policy evaluation
- Tactical move prioritization before NN calls
- UCB formula with policy priors integration
- Node expansion with statistics aggregation

**`src/benchmarks/`** - Validation and testing
- Comprehensive tactical position suite
- Performance comparison methodology
- Statistical analysis and reporting

### Key Algorithms

**MVV-LVA Capture Ordering**:
```rust
fn calculate_mvv_lva(mv: Move, board: &Board) -> f64 {
    let victim_value = piece_value(board.piece_at(mv.to));
    let attacker_value = piece_value(board.piece_at(mv.from));
    (victim_value * 10.0) - attacker_value
}
```

**Fork Detection**:
```rust
fn detect_fork_move(mv: Move, board: &Board, new_board: &Board) -> Option<f64> {
    if piece_type == KNIGHT {
        let knight_attacks = get_knight_attacks(mv.to);
        let targets = knight_attacks & valuable_pieces;
        if targets.count_ones() >= 2 { return Some(calculate_fork_value(targets)); }
    }
    // Similar logic for pawn forks
}
```

## Testing

The engine includes comprehensive validation:

```bash
# Run unit tests
cargo test

# Tactical functionality tests
cargo test tactical_

# Validation experiments
./run_validation_experiments.sh
```

## Neural Network Integration

Optional PyTorch integration provides:
- Policy network for move probability estimation
- Value network for position evaluation
- Training pipeline with PGN data processing
- Rust-Python bridge for seamless inference

```bash
# Train neural networks
python3 python/train_chess_ai.py --epochs 50 --data-source lichess

# Test integration
cargo run --bin neural_test
```

## Configuration

The engine supports various configurations:

```rust
let config = TacticalMctsConfig {
    max_iterations: 1000,
    time_limit: Duration::from_millis(5000),
    mate_search_depth: 3,
    exploration_constant: 1.414,
    use_neural_policy: true,
};
```

## Technical Stack

- **Rust**: High-performance systems programming with memory safety
- **PyTorch**: Neural network training and inference
- **Bitboards**: Efficient board representation and move generation
- **Magic Bitboards**: Ultra-fast sliding piece move generation
- **Zobrist Hashing**: Position caching and transposition tables

## Binary Targets

- `tactical_benchmark` - Tactical search analysis
- `tactical_profiler` - Performance analysis
- `strength_test` - Engine testing and comparison
- `neural_test` - Neural network integration validation
- `generate_training_data` - Training dataset creation
- `texel_tune` - Evaluation parameter optimization

## References

- Silver, D. et al. (2017). "Mastering Chess and Shogi by Self-Play with a General Reinforcement Learning Algorithm"
- Campbell, M. et al. (2002). "Deep Blue"
- "Computer Chess Programming Theory and Practice" by T.A. Marsland

## License

MIT License - see LICENSE file for details.