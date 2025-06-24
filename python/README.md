# Kingfisher Chess AI Training Pipeline

This directory contains the complete training pipeline for the Kingfisher chess engine's neural network policy. The system combines Rust-based position analysis with PyTorch-based deep learning.

## Quick Start

### 1. Install Dependencies

```bash
# Install PyTorch
pip install torch torchvision torchaudio

# Install chess and ML dependencies
pip install python-chess numpy tqdm requests

# Build the Rust engine
cd ..
cargo build --release
```

### 2. Run Complete Training Pipeline

```bash
# Train with sample data (recommended for testing)
python3 train_chess_ai.py --data-source sample --epochs 10

# Train with Lichess data (requires internet)
python3 train_chess_ai.py --data-source lichess --epochs 20
```

### 3. Test the Trained Model

```bash
# Test neural network integration
cd ..
cargo run --bin neural_test
```

## Training Pipeline Components

### Core Files

- **`train_chess_ai.py`** - Complete end-to-end training pipeline
- **`chess_net.py`** - PyTorch neural network architecture
- **`training_pipeline.py`** - Data processing and model training
- **`data_collection.py`** - Data downloading and preprocessing

### Rust Integration

- **`src/training/mod.rs`** - Rust training data generation
- **`src/neural_net.rs`** - Neural network integration with engine
- **`src/bin/generate_training_data.rs`** - Training data generator binary

## Neural Network Architecture

The chess neural network uses a ResNet-style architecture:

- **Input**: 12×8×8 tensor (6 piece types × 2 colors × 8×8 board)
- **Body**: 8 residual blocks with 256 channels
- **Policy Head**: Outputs 4096 move probabilities (64×64 from-to encoding)
- **Value Head**: Outputs position evaluation [-1, 1]

```
Input (12×8×8)
     ↓
Conv2d + BatchNorm + ReLU
     ↓
8× Residual Blocks
     ↓
  ┌─────────┐ ┌─────────┐
  │ Policy  │ │ Value   │
  │ Head    │ │ Head    │
  └─────────┘ └─────────┘
    4096        1
```

## Training Data

### Data Sources

1. **Sample Data** - Built-in tactical and strategic positions
2. **Lichess Database** - Downloaded tournament games
3. **Custom PGN** - Your own game collections

### Data Format

Training data is stored in CSV format:
```csv
fen,result,best_move,engine_eval,description
rnbqkbnr/pp...w KQkq - 0 1,0.5,e2e4,25,"Opening position"
```

### Quality Filters

- Minimum player rating: 1800 (configurable)
- Game length: 20-100 moves
- Time control: ≥3 minutes
- Exclude tactical positions (in check, captures)

## Usage Examples

### Basic Training

```bash
# Quick test with sample data
python3 train_chess_ai.py

# Longer training with more epochs
python3 train_chess_ai.py --epochs 50
```

### Data Collection

```bash
# Download sample games
python3 data_collection.py --action sample

# Download Lichess rapid games
python3 data_collection.py --action lichess --year 2023 --month 6 --variant rapid

# Filter existing PGN file
python3 data_collection.py --action filter --input games.pgn --output filtered.pgn
```

### Custom Training

```bash
# Train on custom PGN file
python3 training_pipeline.py --pgn my_games.pgn --epochs 30 --batch-size 64

# Train on CSV data
python3 training_pipeline.py --csv training_data.csv --output my_model.pth
```

### Generate Training Data with Rust

```bash
# Generate analyzed positions
cargo run --bin generate_training_data

# This creates training_data.csv with engine evaluations
```

## Model Integration

### Using Trained Models

```python
from chess_net import ChessNetInterface
import numpy as np

# Load trained model
interface = ChessNetInterface("models/chess_model.pth")

# Predict on position
board_tensor = np.random.rand(12, 8, 8)  # Your board representation
policy, value = interface.predict(board_tensor)

print(f"Position value: {value:.3f}")
print(f"Best move probability: {policy.max():.3f}")
```

### Rust Integration

```rust
use kingfisher::neural_net::NeuralNetPolicy;
use kingfisher::board::Board;

// Create neural network policy
let mut nn_policy = Some(NeuralNetPolicy::new(Some("models/chess_model.pth".to_string())));

// Use in MCTS search
let best_move = neural_mcts_search(
    board,
    &move_gen,
    &pesto_eval,
    &mut nn_policy,
    2,  // mate search depth
    Some(1000),  // MCTS iterations
    Some(Duration::from_secs(5)),  // time limit
);
```

## Performance Tips

### For Better Training

1. **More Data**: Collect 10,000+ positions for meaningful training
2. **Quality Data**: Use games from strong players (2000+ rating)
3. **Balanced Dataset**: Mix tactical, positional, and endgame positions
4. **Longer Training**: 50+ epochs for convergence

### For Faster Training

1. **GPU Support**: Use CUDA if available
2. **Batch Size**: Increase to 128+ for faster throughput
3. **Parallel Data Loading**: Use multiple workers
4. **Mixed Precision**: Enable for modern GPUs

```bash
# GPU training with larger batches
python3 training_pipeline.py --device cuda --batch-size 128 --epochs 100
```

## Troubleshooting

### Common Issues

1. **PyTorch Not Found**
   ```bash
   pip install torch torchvision torchaudio
   ```

2. **python-chess Missing**
   ```bash
   pip install python-chess
   ```

3. **Rust Compilation Errors**
   ```bash
   cargo clean
   cargo build
   ```

4. **Memory Issues**
   ```bash
   # Reduce batch size
   python3 training_pipeline.py --batch-size 16
   ```

### Performance Issues

- **Slow Training**: Use GPU, increase batch size
- **Poor Convergence**: Collect more/better data, tune learning rate
- **Overfitting**: Add dropout, reduce model size, more data

## Advanced Configuration

### Custom Network Architecture

Edit `chess_net.py` to modify:
- Number of residual blocks
- Channel dimensions
- Policy output size
- Value head structure

### Custom Training Loop

Modify `training_pipeline.py` for:
- Different loss functions
- Learning rate schedules
- Data augmentation
- Regularization techniques

### Integration with Engine

Update `src/neural_net.rs` for:
- Different tensor formats
- Custom evaluation blending
- Performance optimizations
- Caching strategies

## Results and Benchmarks

A well-trained model should achieve:
- **Policy Accuracy**: >40% top-1 move prediction
- **Value Correlation**: >0.7 with engine evaluation
- **MCTS Improvement**: 50-100 Elo gain over pure classical evaluation

Monitor training with:
- Loss curves (policy + value)
- Validation accuracy
- Move prediction rate
- Value prediction error

## Next Steps

1. **Collect More Data**: Use Lichess database downloads
2. **Hyperparameter Tuning**: Optimize learning rate, batch size
3. **Architecture Experiments**: Try different network designs
4. **Self-Play Training**: Generate data from engine games
5. **Strength Testing**: Compare against baseline engine

For more advanced training, consider:
- AlphaZero-style self-play
- Knowledge distillation from stronger engines
- Multi-task learning (tactics + strategy)
- Reinforcement learning fine-tuning