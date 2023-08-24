![kingfisher_logo](https://github.com/aaholmes/chess/assets/4913443/bcca7d62-cfd0-407c-9e17-fbcbe403dfcf)
# Kingfisher

The goal is to make a reasonably good chess engine inspired by
1. The book "Neural Networks for Chess" by Dominik Klein - they say that the future best engines will continue to be alpha-beta pruning with a fast neural network evaluation function.
2. The Berserk chess engine - one of the best open source engines, and appears to be only a few years old.

## Goals for first version
1. Bitboards
2. Move generation
3. Alpha-beta pruning
4. Iterative deepening
5. Pesto evaluation function
 
## Eventual other goals
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
12. Pawn structure evaluation (doubled, isolated, backward, passed, duo, defended) (optional)
13. Tune evaluation function with linear regression (optional)
14. Minor move ordering (optional)
15. Null move pruning (optional)

## Programming language
We will use Rust for the first version. In the long term we may want to use Rust for the search, Python for the neural network, and Julia as a driver.

## Credits
Image generated using Canva
