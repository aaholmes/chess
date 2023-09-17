# Kingfisher
![kingfisher_logo](https://github.com/aaholmes/chess/assets/4913443/059cd779-912b-439b-8eee-e4f513c25b01)


The goal is to make a reasonably good chess engine inspired by
1. The book "Neural Networks for Chess" by Dominik Klein - they say that the future best engines will continue to be alpha-beta pruning with a fast neural network evaluation function.
2. The Berserk chess engine - one of the best open source engines, and appears to be only a few years old.

## Goals for first version
1. Bitboards (DONE)
2. Move generation including magic bitboards (DONE)
3. Negamax search (DONE)
4. Alpha-beta pruning (DONE)
5. Iterative deepening (DONE except for move ordering)
6. Pesto evaluation function (tapered) (DONE)
7. Move ordering:
   1. Mate killer heuristic
   2. MVV-LVA (DONE)
   3. Knight forks (DONE)
   4. Non-captures ordered according to Pesto eval (DONE)
8. Quiescence search (DONE)
9. Aspiration windows (DONE)
10. Transposition table
11. Null move pruning
12. UCI protocol

I believe that this will already be enough to make a pretty strong engine.

## Goals for second version
1. Time management
2. Neural network evaluation function (DenseNet NNUE plus tapered Pesto)
3. Opening book
4. Endgame tablebases
5. Parallel search

If I get this far, the engine should be very strong.
 
## Programming language
We will use Rust for the first version. In the long term we may want to use Rust for the search, Python for the neural network, and Julia as a driver.

## Credits
Image generated using Canva
