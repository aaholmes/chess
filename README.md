# Kingfisher

<img width="1038" alt="Screenshot 2023-08-24 at 1 44 31 PM" src="https://github.com/aaholmes/chess/assets/4913443/ceab66cf-67c8-4685-bd28-d454c38ce756">

The goal is to make a reasonably good chess engine inspired by
1. The book "Neural Networks for Chess" by Dominik Klein
2. The Berserk chess engine - one of the best chess engines in the world, and appears to be developed primarily by one person over just a few years

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
   3. Pawn and knight forks (DONE) 
   4. Non-captures ordered according to Pesto eval (DONE)
8. Mate search (DONE)
9. Quiescence search (DONE)
10. Aspiration windows (DONE)
11. Transposition table
12. Null move pruning
13. UCI protocol

I believe that this will already be enough to make an engine that's at least entertaining to play against.

## Goals for second version
1. Time management
2. Endgame tablebases
3. Opening book
4. Parallel search
5. Neural network evaluation function (start by augmenting the tapered Pesto eval with a shallow neural network that just looks at king and pawn positions, since king safety and pawn structure are the Pesto eval's biggest weaknesses)

If I get this far, the engine should be very strong and we can start staging matches against early versions of other engines.
 
## Programming language
We will use Rust for the first version. When we start using neural networks, we may want to interface with either PyTorch or Julia. 

## Credits
Image generated using Canva
