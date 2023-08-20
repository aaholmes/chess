# Chess engine
The goal is to make a reasonably good chess engine inspired by
1. The book "Neural Networks for Chess" by Dominik Klein - they say that the future best engines will continue to be alpha-beta pruning with a fast neural network evaluation function.
2. The Berserk chess engine - one of the best open source engines, and appears to be only a few years old.

## Goals for first version
1. Bitboard representation of the board
2. Magic bitboards for sliding pieces
3. Depth-first alpha-beta pruning
4. MVV-LVA move ordering
5. Negamax
6. Quiescence search
7. Transposition table
8. Iterative deepening
9. Aspiration windows
10. Tapered evaluation
11. Material, piece-square, and mobility evaluation

## Programming language
We will use Rust for the first version. In the long term we may want to use Rust for the search, Python for the neural network, and Julia as a driver.