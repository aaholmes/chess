// Pesto evaluation function
// From CPW: Tapered eval to interpolate by current game stage between piece-square tables for
// opening and endgame, optimized by Texel tuning.

use std::cmp::min;
use crate::bitboard::{Bitboard, flip_sq_ind_vertically};

pub(crate) struct PestoEval {
    mg_table: [[i32; 64]; 12],
    eg_table: [[i32; 64]; 12]
}
const PAWN: usize = 0;
const KNIGHT: usize = 1;
const BISHOP: usize = 2;
const ROOK: usize = 3;
const QUEEN: usize = 4;
const KING: usize = 5;

const WHITE: usize = 0;
const BLACK: usize = 1;

const WHITE_PAWN: usize = 2 * PAWN + WHITE;
const BLACK_PAWN: usize = 2 * PAWN + BLACK;
const WHITE_KNIGHT: usize = 2 * KNIGHT + WHITE;
const BLACK_KNIGHT: usize = 2 * KNIGHT + BLACK;
const WHITE_BISHOP: usize = 2 * BISHOP + WHITE;
const BLACK_BISHOP: usize = 2 * BISHOP + BLACK;
const WHITE_ROOK: usize = 2 * ROOK + WHITE;
const BLACK_ROOK: usize = 2 * ROOK + BLACK;
const WHITE_QUEEN: usize = 2 * QUEEN + WHITE;
const BLACK_QUEEN: usize = 2 * QUEEN + BLACK;
const WHITE_KING: usize = 2 * KING + WHITE;
const BLACK_KING: usize = 2 * KING + BLACK;

fn player(piece: usize) -> usize {piece & 1}

// Piece values in middlegame
const MG_VALUE: [i32; 6] = [ 82, 337, 365, 477, 1025,  0];

// Piece values in endgame
const EG_VALUE: [i32; 6] = [ 94, 281, 297, 512,  936,  0];

// Piece-square tables
// Values from Rofchade: http://www.talkchess.com/forum3/viewtopic.php?f=2&t=68311&start=19
// Note that these apparently use a different indexing, so we need to flip the board vertically for white

const MG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    98, 134, 61, 95, 68, 126, 34, -11,
    -6, 7, 26, 31, 65, 56, 25, -20,
    -14, 13, 6, 21, 23, 12, 17, -23,
    -27, -2, -5, 12, 17, 6, 10, -25,
    -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const EG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    178, 173, 158, 134, 147, 132, 165, 187,
    94, 100, 85, 67, 56, 53, 82, 84,
    32, 24, 13, 5, -2, 4, 17, 17,
    13, 9, -3, -7, -7, -8, 3, -1,
    4, 7, -6, 1, 0, -5, -1, -8,
    13, 8, 8, 10, 13, 0, 2, -7,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107,
    -73, -41, 72, 36, 23, 62, 7, -17,
    -47, 60, 37, 65, 84, 129, 73, 44,
    -9, 17, 19, 53, 37, 69, 18, 22,
    -13, 4, 16, 13, 28, 19, 21, -8,
    -23, -9, 12, 10, 19, 17, 25, -16,
    -29, -53, -12, -3, -1, 18, -14, -19,
    -105, -21, -58, -33, -17, -28, -19, -23,
];

const EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25, -8, -25, -2, -9, -25, -24, -52,
    -24, -20, 10, 9, -1, -9, -19, -41,
    -17, 3, 22, 22, 22, 11, 8, -18,
    -18, -6, 16, 25, 16, 17, 4, -18,
    -23, -3, -1, 15, 10, -3, -20, -22,
    -42, -20, -10, -5, -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

const MG_BISHOP_TABLE: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8,
    -26, 16, -18, -13, 30, 59, 18, -47,
    -16, 37, 43, 40, 35, 50, 37, -2,
    -4, 5, 19, 50, 37, 37, 7, -2,
    -6, 13, 13, 26, 34, 12, 10, 4,
    0, 15, 15, 15, 14, 27, 18, 10,
    4, 15, 16, 0, 7, 21, 33, 1,
    -33, -3, -14, -21, -13, -12, -39, -21,
];

const EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24,
    -8, -4, 7, -12, -3, -13, -4, -14,
    2, -8, 0, -1, -2, 6, 0, 4,
    -3, 9, 12, 9, 14, 10, 3, 2,
    -6, 3, 13, 19, 7, 10, -3, -9,
    -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27,
    -23, -9, -23, -5, -9, -16, -5, -17,
];

const MG_ROOK_TABLE: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43,
    27, 32, 58, 62, 80, 67, 26, 44,
    -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20,
    -36, -26, -12, -1, 9, -7, 6, -23,
    -45, -25, -16, -17, 3, 0, -5, -33,
    -44, -16, -20, -9, -1, 11, -6, -71,
    -19, -13, 1, 17, 16, 7, -37, -26,
];

const EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12, 12, 8, 5,
    11, 13, 13, 11, -3, 3, 8, 3,
    7, 7, 7, 5, 4, -3, -5, -3,
    4, 3, 13, 1, 2, 1, -1, 2,
    3, 5, 8, 4, -5, -6, -8, -11,
    -4, 0, -5, -1, -7, -12, -8, -16,
    -6, -6, 0, 2, -9, -9, -11, -3,
    -9, 2, 3, -1, -5, -13, 4, -20,
];

const MG_QUEEN_TABLE: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45,
    -24, -39, -5, 1, -16, 57, 28, 54,
    -13, -17, 7, 8, 29, 56, 47, 57,
    -27, -27, -16, -16, -1, 17, -2, 1,
    -9, -26, -9, -10, -2, -4, 3, -3,
    -14, 2, -11, -2, -5, 2, 14, 5,
    -35, -8, 11, 2, 8, 15, -3, 1,
    -1, -18, -9, 10, -15, -25, -31, -50,
];

const EG_QUEEN_TABLE: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20,
    -17, 20, 32, 41, 58, 25, 30, 0,
    -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36,
    -18, 28, 19, 47, 31, 34, 39, 23,
    -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43, -5, -32, -20, -41,
];

const MG_KING_TABLE: [i32; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13,
    29, -1, -20, -7, -8, -4, -38, -29,
    -9, 24, 2, -16, -20, 6, 22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49, -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
    1, 7, -8, -64, -43, -16, 9, 8,
    -15, 36, 12, -54, 8, -28, 24, 14,
];

const EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17,
    -12, 17, 14, 17, 17, 38, 23, 11,
    10, 17, 23, 15, 20, 45, 44, 13,
    -8, 22, 24, 27, 26, 33, 26, 3,
    -18, -4, 21, 24, 27, 23, 9, -11,
    -19, -3, 11, 21, 23, 16, 7, -9,
    -27, -11, 4, 13, 14, 4, -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

const MG_PESTO_TABLE: [[i32; 64]; 6] =
    [
        MG_PAWN_TABLE,
        MG_KNIGHT_TABLE,
        MG_BISHOP_TABLE,
        MG_ROOK_TABLE,
        MG_QUEEN_TABLE,
        MG_KING_TABLE
    ];

const EG_PESTO_TABLE: [[i32; 64]; 6] =
    [
        EG_PAWN_TABLE,
        EG_KNIGHT_TABLE,
        EG_BISHOP_TABLE,
        EG_ROOK_TABLE,
        EG_QUEEN_TABLE,
        EG_KING_TABLE
    ];

const GAMEPHASE_INC: [i32; 12] = [0,0,1,1,1,1,2,2,4,4,0,0];

impl PestoEval {
    pub fn new() -> PestoEval
    {
        let mut mg_table: [[i32; 64]; 12] = [[0; 64]; 12];
        let mut eg_table: [[i32; 64]; 12] = [[0; 64]; 12];
        for p in PAWN..=KING {
            for sq in 0..64 {
                mg_table[2 * p][sq] = MG_VALUE[p] + MG_PESTO_TABLE[p][flip_sq_ind_vertically(sq as u8) as usize];
                eg_table[2 * p][sq] = EG_VALUE[p] + EG_PESTO_TABLE[p][flip_sq_ind_vertically(sq as u8) as usize];
                mg_table[2 * p + 1][sq] = MG_VALUE[p] + MG_PESTO_TABLE[p][sq];
                eg_table[2 * p + 1][sq] = EG_VALUE[p] + EG_PESTO_TABLE[p][sq];
            }
        }
        PestoEval{
            mg_table,
            eg_table
        }
    }

    pub fn eval(&self, board: Bitboard) -> i32
    // Computes the eval according to the Pesto evaluation function
    // Relative to the side to move
    {
        let mut mg: [i32; 2] = [0, 0];
        let mut eg: [i32; 2] = [0, 0];
        let mut game_phase: i32 = 0;

        // Evaluate each piece
        for piece in 0..12 {
            for sq in 0..64 {
                if board.pieces[piece as usize] & (1 << sq) != 0 {
                    mg[player(piece)] += self.mg_table[piece][sq];
                    eg[player(piece)] += self.eg_table[piece][sq];
                    game_phase += GAMEPHASE_INC[piece];
                }
            }
        }

        // Tapered eval
        let mg_score: i32;
        let eg_score: i32;
        if board.w_to_move {
            mg_score = mg[1] - mg[0];
            eg_score = eg[1] - eg[0];
        } else {
            mg_score = mg[0] - mg[1];
            eg_score = eg[0] - eg[1];
        }
        let mg_phase: i32 = min(24, game_phase); // Can exceed 24 in case of early promotion
        let eg_phase: i32 = 24 - mg_phase;
        return (mg_score * mg_phase + eg_score * eg_phase) / 24;
    }
}