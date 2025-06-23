# HumanlikeAgent Implementation Documentation

## Overview
The HumanlikeAgent is a chess engine that combines classical chess techniques with Monte Carlo Tree Search (MCTS). While designed with future extensions for neural networks and human-like play in mind, the current implementation uses a three-phase decision process:

1. Optional EGTB (Endgame Tablebase) lookup
2. Mate Search
3. MCTS with Pesto evaluation

## Components

### 1. Agent Structure
```rust
pub struct HumanlikeAgent<'a> {
    pub move_gen: &'a MoveGen,
    pub pesto: &'a PestoEval,
    pub egtb_prober: Option<EgtbProber>,
    pub mate_search_depth: i32,
    pub mcts_iterations: u32,
    pub mcts_time_limit_ms: u64,
}
```

### 2. Decision Flow

#### Phase 1: EGTB (Optional)
- If `egtb_prober` is Some and position has ≤ max_pieces:
  - Probes EGTB for perfect play information
  - Currently only logs results without using them for move selection
  - Future: Will use EGTB moves when available

#### Phase 2: Mate Search
- Performs classical mate search up to specified depth
- If mate is found, immediately returns the mating move
- Uses standard alpha-beta mate search algorithm

#### Phase 3: MCTS
If no mate is found, performs MCTS with the following characteristics:

##### A. Node Selection
- Uses PUCT (Polynomial Upper Confidence Trees) for tree traversal
- Exploration constant: √2 (≈1.414)
- Selection formula: Q + U where:
  - Q = average value for selector
  - U = c * P * √(N_parent) / (1 + N_child)
  - P = 1/num_legal_moves (uniform prior)

##### B. Move Prioritization
Moves are categorized and explored in strict priority order:

1. **Checks** (Highest Priority)
   - Detected by applying move and checking if opponent king is in check
   - Explored in generation order

2. **Captures** (Medium Priority)
   - Includes both captures and promotions
   - Sorted by MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
   - Includes en passant captures

3. **Quiet Moves** (Lowest Priority)
   - All other legal moves
   - Explored in generation order

##### C. Position Evaluation
Uses the Pesto evaluation function, which includes:
- Material and piece-square tables
- Pawn structure evaluation
  - Passed pawns
  - Isolated pawns
  - Pawn chains
  - Pawn duos
  - Backward pawns
- King safety
  - Pawn shield
  - King attack score
  - Castling rights
- Piece mobility
- Two bishops bonus
- Rook positioning
  - Open/half-open files
  - Rooks on 7th rank
  - Rooks behind passed pawns

The raw evaluation in centipawns is converted to a probability using a sigmoid function:
```rust
value = 1.0 / (1.0 + (-score_cp as f64 / 400.0).exp())
```

##### D. Backpropagation
- Updates visit counts and total values
- Maintains squared values for variance calculation
- Values always stored from White's perspective [0.0, 1.0]

##### E. Final Move Selection
Uses a pessimistic value selection strategy:
- Calculates Q - k*std_err for each move
- k = 1.0 (configurable pessimism factor)
- Helps avoid moves with high uncertainty
- For White: maximizes lower confidence bound
- For Black: minimizes upper confidence bound

### 3. Termination Conditions
MCTS search stops when either:
- Specified number of iterations reached
- Time limit exceeded (checked every 64 iterations)

## Current Limitations
1. EGTB integration exists but doesn't influence move selection
2. No neural network policy/evaluation
3. No rating band-specific behavior
4. Move ordering within Check and Quiet categories is not randomized
5. MVV-LVA scoring implementation needs completion

## Future Extensions
1. Complete EGTB move selection
2. Add neural network policy and value functions
3. Implement rating band-specific behavior
4. Add opening book integration
5. Randomize move selection within categories
6. Complete MVV-LVA implementation for captures

## Performance Considerations
- Mate search depth and MCTS parameters are configurable
- MCTS time management checks every 64 iterations
- Move legality checking during categorization could be optimized
- Check detection requires temporary move application 