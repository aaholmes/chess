# Kingfisher Neural Network Training Guide

This comprehensive guide walks you through training neural networks for the Kingfisher Chess Engine, from basic setup to advanced optimization techniques.

## 🚀 Quick Start

### Prerequisites
```bash
# Install Python dependencies
pip install torch torchvision torchaudio python-chess numpy tqdm requests

# Verify PyTorch installation
python3 -c "import torch; print(f'PyTorch {torch.__version__} installed successfully')"

# Build Rust components
cargo build --release
```

### Basic Training (5 minutes)
```bash
# Train with built-in sample data
python3 python/train_chess_ai.py --data-source sample --epochs 10

# This creates: python/models/chess_model.pth
```

### Intermediate Training (30 minutes)
```bash
# Train with Lichess database sample
python3 python/train_chess_ai.py --data-source lichess --epochs 20

# Monitor training progress in terminal
```

### Advanced Training (2+ hours)
```bash
# Full training with large dataset
python3 python/train_chess_ai.py \
    --data-source lichess \
    --epochs 100 \
    --batch-size 128 \
    --device cuda  # if GPU available
```

## 📊 Training Pipeline Overview

```
Data Collection → Data Processing → Model Training → Evaluation → Integration
      ↓               ↓               ↓              ↓            ↓
  PGN Files      CSV Positions   PyTorch Model   Validation   Rust Engine
```

### Step 1: Data Collection

**Option A: Sample Data (Fastest)**
```python
# Built-in tactical and strategic positions
python3 python/data_collection.py --action sample
# Creates: data/sample_games.pgn
```

**Option B: Lichess Database (Recommended)**
```python
# Download rated games from Lichess
python3 python/data_collection.py \
    --action lichess \
    --year 2023 \
    --month 6 \
    --variant rapid \
    --min-rating 1800
```

**Option C: Custom PGN**
```python
# Filter your own game collection
python3 python/data_collection.py \
    --action filter \
    --input my_games.pgn \
    --output filtered_games.pgn \
    --min-rating 1600
```

### Step 2: Data Processing

**Generate Training Positions**:
```bash
# Use Rust engine to analyze positions
cargo run --bin generate_training_data

# Or use Python pipeline
python3 python/training_pipeline.py \
    --pgn data/sample_games.pgn \
    --output data/training_positions.csv \
    --max-positions 10000
```

**Quality Filtering Criteria**:
- Player rating ≥ 1800 (configurable)
- Game length: 20-100 moves
- Time control ≥ 3 minutes
- Exclude positions in check (too tactical)
- Minimum 12 pieces remaining
- Sample every 3rd move from middle game

### Step 3: Model Training

**Architecture Configuration**:
```python
# In chess_net.py - adjustable parameters
class ChessNet(nn.Module):
    def __init__(self, 
                 input_channels=12,     # 6 pieces × 2 colors
                 hidden_channels=256,   # ResNet channel width
                 num_res_blocks=8,      # Network depth
                 policy_output_size=4096): # 64×64 move encoding
```

**Training Parameters**:
```python
# Key hyperparameters to tune
learning_rate = 0.001
batch_size = 32        # Increase to 128+ for GPU
weight_decay = 0.0001  # L2 regularization
epochs = 50            # More for larger datasets
```

**Loss Function**:
```python
# Combined policy and value loss
policy_loss = F.cross_entropy(policy_pred, policy_target)
value_loss = F.mse_loss(value_pred, value_target)
total_loss = policy_loss + value_loss
```

## 📈 Training Monitoring

### Progress Tracking
```bash
# Training output shows:
Epoch 15/50: Loss=0.847 (Policy=0.523, Value=0.324) | Acc=41.2% | Time=2.3s
```

**Key Metrics**:
- **Total Loss**: Should decrease consistently
- **Policy Loss**: Cross-entropy on move prediction
- **Value Loss**: MSE on position evaluation
- **Accuracy**: Top-1 move prediction rate
- **Time per Epoch**: Monitor for performance issues

### Validation Monitoring
```python
# Automatic validation every 5 epochs
if epoch % 5 == 0:
    val_loss, val_acc = validate_model(model, val_loader)
    print(f"Validation: Loss={val_loss:.3f}, Accuracy={val_acc:.1f}%")
```

## 🎯 Model Integration & Testing

### Integration with Engine
```bash
# Test neural network with Rust engine
cargo run --bin neural_test

# Run strength comparison
cargo run --bin strength_test --neural-model python/models/chess_model.pth
```

### Expected Performance
**Well-Trained Model Targets**:
- Policy Accuracy: >40% top-1 move prediction
- Value Correlation: >0.7 with engine evaluation  
- MCTS Improvement: 50-100 Elo gain over classical
- Training Time: 2-4 hours for 50k positions

## 🔧 Advanced Training Techniques

### Hyperparameter Optimization

**Learning Rate Scheduling**:
```python
# Reduce learning rate when validation plateaus
scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(
    optimizer, mode='min', factor=0.5, patience=5
)
```

**Data Augmentation**:
```python
# Chess-specific augmentations
def augment_position(board_tensor):
    # Horizontal flip (mirror board)
    if random.random() < 0.5:
        board_tensor = torch.flip(board_tensor, dims=[2])
    return board_tensor
```

**Advanced Architecture**:
```python
# Attention mechanisms for long-range dependencies
class ChessAttention(nn.Module):
    def __init__(self, channels):
        self.attention = nn.MultiheadAttention(channels, num_heads=8)
        
# Squeeze-and-Excitation blocks
class SEBlock(nn.Module):
    def __init__(self, channels, reduction=16):
        self.se = nn.Sequential(
            nn.AdaptiveAvgPool2d(1),
            nn.Conv2d(channels, channels // reduction, 1),
            nn.ReLU(),
            nn.Conv2d(channels // reduction, channels, 1),
            nn.Sigmoid()
        )
```

### Large-Scale Training

**Multi-GPU Training**:
```python
# Data parallel training
if torch.cuda.device_count() > 1:
    model = nn.DataParallel(model)
    batch_size *= torch.cuda.device_count()
```

**Memory Optimization**:
```python
# Gradient accumulation for large effective batch sizes
accumulation_steps = 4
for i, batch in enumerate(dataloader):
    loss = model(batch) / accumulation_steps
    loss.backward()
    
    if (i + 1) % accumulation_steps == 0:
        optimizer.step()
        optimizer.zero_grad()
```

**Checkpointing**:
```python
# Save/resume training state
torch.save({
    'epoch': epoch,
    'model_state_dict': model.state_dict(),
    'optimizer_state_dict': optimizer.state_dict(),
    'loss': loss,
    'best_val_acc': best_val_acc
}, f'checkpoint_epoch_{epoch}.pth')
```

## 🐛 Troubleshooting

### Common Issues

**1. Poor Convergence**
```
Symptoms: Loss not decreasing, accuracy stuck around 20%
Solutions:
- Reduce learning rate (try 0.0001)
- Increase dataset size (need 10k+ positions)
- Check data quality (filter low-rated games)
- Verify loss function implementation
```

**2. Overfitting**
```
Symptoms: Training accuracy high, validation accuracy low
Solutions:
- Add dropout (0.1-0.3)
- Increase dataset size
- Reduce model complexity (fewer residual blocks)
- Add weight decay regularization
```

**3. Memory Issues**
```
Symptoms: CUDA out of memory, system RAM exhausted
Solutions:
- Reduce batch size (try 16 or 8)
- Use gradient accumulation
- Enable mixed precision training
- Reduce dataset size loaded at once
```

**4. Slow Training**
```
Symptoms: Very slow epochs, high CPU usage
Solutions:
- Use GPU if available (--device cuda)
- Increase batch size
- Use multiple DataLoader workers
- Optimize data loading pipeline
```

### Performance Debugging

**Profile Training**:
```python
# Add timing to identify bottlenecks
import time

start_time = time.time()
# ... training code ...
print(f"Epoch time: {time.time() - start_time:.2f}s")
```

**Memory Profiling**:
```python
# Monitor GPU memory usage
if torch.cuda.is_available():
    print(f"GPU Memory: {torch.cuda.memory_allocated() / 1e9:.2f}GB")
```

## 📊 Advanced Evaluation

### Custom Metrics
```python
def calculate_move_quality(predictions, targets, board_evaluations):
    """Custom metric: How much does predicted move improve position?"""
    quality_scores = []
    for pred, target, eval_before in zip(predictions, targets, board_evaluations):
        # Implementation depends on your evaluation function
        pass
    return np.mean(quality_scores)
```

### A/B Testing
```bash
# Compare model versions
cargo run --bin strength_test --neural-model models/baseline.pth > baseline_results.txt
cargo run --bin strength_test --neural-model models/improved.pth > improved_results.txt

# Analyze improvements
python3 scripts/compare_models.py baseline_results.txt improved_results.txt
```

## 🎯 Next Steps

### Immediate Improvements
1. **Collect More Data**: 50k+ positions for production models
2. **Hyperparameter Tuning**: Grid search on learning rate, architecture
3. **Data Quality**: Higher rating thresholds, better position filtering
4. **Validation**: Cross-validation for robust performance estimates

### Advanced Techniques
1. **Self-Play Training**: Generate data from engine vs engine games
2. **Curriculum Learning**: Start with simple positions, progress to complex
3. **Multi-Task Learning**: Train on multiple chess objectives simultaneously
4. **Transfer Learning**: Pre-train on large datasets, fine-tune on specific styles

### Research Directions
1. **Architecture Search**: Automated neural network design
2. **Interpretability**: Understand what the network learns
3. **Efficiency**: Faster inference for real-time play
4. **Robustness**: Performance across different playing styles

---

**Success Criteria**: A well-trained model should achieve >40% move prediction accuracy and provide meaningful Elo improvements when integrated with the MCTS search algorithm. Monitor both training metrics and engine performance to validate improvements.