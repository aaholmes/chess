# Kingfisher
![kingfisher_logo](https://github.com/aaholmes/chess/assets/4913443/059cd779-912b-439b-8eee-e4f513c25b01)


The goal is to make a reasonably good chess engine inspired by
1. The book "Neural Networks for Chess" by Dominik Klein - they say that the future best engines will continue to be alpha-beta pruning with a fast neural network evaluation function.
2. The Berserk chess engine - one of the best open source engines, and appears to be only a few years old.

## Goals for first version
1. Bitboards
2. Move generation including magic bitboards
3. Negamax search
4. Alpha-beta pruning
5. Iterative deepening
6. Pesto evaluation function (tapered)
7. Move ordering:
   1. Mate killer heuristic
   2. MVV-LVA
   3. Knight forks
   4. Non-captures ordered according to Pesto eval
8. Quiescence search
9. Aspiration windows
10. Transposition table
11. Null move pruning
12. Mate killer heuristic
13. UCI protocol
 
## Programming language
We will use Rust for the first version. In the long term we may want to use Rust for the search, Python for the neural network, and Julia as a driver.

## Credits
Image generated using Canva
