use crate::gen_moves::MoveGen;

// Piece labels
pub(crate) const WP: usize = 0;
pub(crate) const BP: usize = 1;
pub(crate) const WN: usize = 2;
pub(crate) const BN: usize = 3;
pub(crate) const WB: usize = 4;
pub(crate) const BB: usize = 5;
pub(crate) const WR: usize = 6;
pub(crate) const BR: usize = 7;
pub(crate) const WQ: usize = 8;
pub(crate) const BQ: usize = 9;
pub(crate) const WK: usize = 10;
pub(crate) const BK: usize = 11;
pub(crate) const WOCC: usize = 12; // White occupied squares
pub(crate) const BOCC: usize = 13; // Black occupied squares
pub(crate) const OCC: usize = 14; // All occupied squares


// Define the bitboard data type.
// We will use a 64-bit unsigned integer to represent the bitboard for each piece.
#[derive(Clone)]
pub struct Bitboard {
    pub(crate) game_result: Option<i32>, // None = in progress, 1 = white wins, -1 = black wins, 0 = draw
    pub(crate) w_to_move: bool,
    pub(crate) w_castle_k: bool,
    pub(crate) w_castle_q: bool,
    pub(crate) b_castle_k: bool,
    pub(crate) b_castle_q: bool,
    pub(crate) en_passant: Option<usize>, // index of square where en passant is possible
    pub(crate) halfmove_clock: u8, // number of halfmoves since last capture or pawn advance
    pub(crate) fullmove_clock: u8, // number of fullmoves since start of game
    pub(crate) pieces: Vec<u64>
}

// Little Endian Rank Mapping

// Least Significant File
// ind = 8 * rank + file

pub fn coords_to_sq_ind(file: usize, rank: usize) -> usize {
    8 * rank + file
}

pub fn sq_ind_to_coords(sq_ind: usize) -> (usize, usize) {
    (sq_ind % 8, sq_ind / 8)
}

pub fn sq_ind_to_bit(sq_ind: usize) -> u64 {
    1 << sq_ind
}

pub fn bit_to_sq_ind(bit: u64) -> usize {
    bit.trailing_zeros() as usize
}

pub fn sq_ind_to_algebraic(sq_ind: usize) -> String {
    let (file, rank) = sq_ind_to_coords(sq_ind);
    let file = (file + 97) as u8 as char;
    let rank = (rank + 49) as u8 as char;
    format!("{}{}", file, rank)
}

pub fn algebraic_to_sq_ind(algebraic: &str) -> usize {
    let mut chars = algebraic.chars();
    let file = chars.next().unwrap() as usize - 97;
    let rank = chars.next().unwrap() as usize - 49;
    coords_to_sq_ind(file, rank)
}

pub fn algebraic_to_bit(algebraic: &str) -> u64 {
    let sq_ind = algebraic_to_sq_ind(algebraic);
    sq_ind_to_bit(sq_ind)
}

pub fn bit_to_algebraic(bit: u64) -> String {
    let sq_ind = bit_to_sq_ind(bit);
    sq_ind_to_algebraic(sq_ind)
}

pub fn flip_sq_ind_vertically(sq_ind: usize) -> usize {
    8 * (7 - sq_ind / 8) + sq_ind % 8
}

pub fn flip_vertically(bit: u64) -> u64 {
    return  ( (bit << 56)                    ) |
        ( (bit << 40) & (0x00ff000000000000) ) |
        ( (bit << 24) & (0x0000ff0000000000) ) |
        ( (bit <<  8) & (0x000000ff00000000) ) |
        ( (bit >>  8) & (0x00000000ff000000) ) |
        ( (bit >> 24) & (0x0000000000ff0000) ) |
        ( (bit >> 40) & (0x000000000000ff00) ) |
        ( (bit >> 56) );
}

impl Bitboard {
    pub(crate) fn new() -> Bitboard {
        Bitboard {
            game_result: None,
            w_to_move: true,
            w_castle_k: true,
            w_castle_q: true,
            b_castle_k: true,
            b_castle_q: true,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
            pieces: [
                0x000000000000FF00,
                0x00FF000000000000,
                0x0000000000000042,
                0x4200000000000000,
                0x0000000000000024,
                0x2400000000000000,
                0x0000000000000081,
                0x8100000000000000,
                0x0000000000000008,
                0x0800000000000000,
                0x0000000000000010,
                0x1000000000000000,
                0x000000000000FFFF,
                0xFFFF000000000000,
                0xFFFF00000000FFFF
            ].try_into().unwrap()
        }
    }

    // FEN reader
    pub(crate) fn new_from_fen(fen: &str) -> Bitboard {
        let parts = fen.split(' ').collect::<Vec<&str>>();
        let mut board = Bitboard::new();
        board.pieces = [0; 15].try_into().unwrap();
        board.w_castle_k = false;
        board.w_castle_q = false;
        board.b_castle_k = false;
        board.b_castle_q = false;
        let mut rank = 7;
        let mut file = 0;
        for c in parts[0].chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if c.is_digit(10) {
                file += c.to_digit(10).unwrap() as usize;
            } else {
                let sq_ind = coords_to_sq_ind(file, rank);
                let bit = sq_ind_to_bit(sq_ind);
                match c {
                    'P' => board.pieces[WP] ^= bit,
                    'p' => board.pieces[BP] ^= bit,
                    'N' => board.pieces[WN] ^= bit,
                    'n' => board.pieces[BN] ^= bit,
                    'B' => board.pieces[WB] ^= bit,
                    'b' => board.pieces[BB] ^= bit,
                    'R' => board.pieces[WR] ^= bit,
                    'r' => board.pieces[BR] ^= bit,
                    'Q' => board.pieces[WQ] ^= bit,
                    'q' => board.pieces[BQ] ^= bit,
                    'K' => board.pieces[WK] ^= bit,
                    'k' => board.pieces[BK] ^= bit,
                    _ => panic!("Invalid FEN")
                }
                file += 1;
            }
        }
        match parts[1] {
            "w" => board.w_to_move = true,
            "b" => board.w_to_move = false,
            _ => panic!("Invalid FEN")
        }
        match parts[2] {
            "-" => (),
            _ => {
                for c in parts[2].chars() {
                    match c {
                        'K' => board.w_castle_k = true,
                        'Q' => board.w_castle_q = true,
                        'k' => board.b_castle_k = true,
                        'q' => board.b_castle_q = true,
                        _ => panic!("Invalid FEN")
                    }
                }
            }
        }
        match parts[3] {
            "-" => (),
            _ => {
                let sq_ind = algebraic_to_sq_ind(parts[3]);
                board.en_passant = Some(sq_ind);
            }
        }
        match parts[4] {
            "0" => (),
            _ => {
                board.halfmove_clock = parts[4].parse::<u8>().unwrap();
            }
        }
        match parts[5] {
            "1" => (),
            _ => {
                board.fullmove_clock = parts[5].parse::<u8>().unwrap();
            }
        }
        board.pieces[WOCC] = board.pieces[WP] | board.pieces[WN] | board.pieces[WB] | board.pieces[WR] | board.pieces[WQ] | board.pieces[WK];
        board.pieces[BOCC] = board.pieces[BP] | board.pieces[BN] | board.pieces[BB] | board.pieces[BR] | board.pieces[BQ] | board.pieces[BK];
        board.pieces[OCC] = board.pieces[WOCC] | board.pieces[BOCC];
        board
    }

    pub fn print(&self) {
        println!("  +-----------------+");
        for rank in (0..8).rev() {
            print!("{} | ", rank + 1);
            for file in 0..8 {
                let sq_ind = coords_to_sq_ind(file, rank);
                let bit = sq_ind_to_bit(sq_ind);
                if bit & self.pieces[WK] != 0 {
                    print!("K ");
                } else if bit & self.pieces[WQ] != 0 {
                    print!("Q ");
                } else if bit & self.pieces[WR] != 0 {
                    print!("R ");
                } else if bit & self.pieces[WB] != 0 {
                    print!("B ");
                } else if bit & self.pieces[WN] != 0 {
                    print!("N ");
                } else if bit & self.pieces[WP] != 0 {
                    print!("P ");
                } else if bit & self.pieces[BK] != 0 {
                    print!("k ");
                } else if bit & self.pieces[BQ] != 0 {
                    print!("q ");
                } else if bit & self.pieces[BR] != 0 {
                    print!("r ");
                } else if bit & self.pieces[BB] != 0 {
                    print!("b ");
                } else if bit & self.pieces[BN] != 0 {
                    print!("n ");
                } else if bit & self.pieces[BP] != 0 {
                    print!("p ");
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }

    pub fn flip_vertically(&self) -> Bitboard {
        // Flip the board vertically, returning a new board.
        Bitboard {
            game_result: self.game_result,
            w_to_move: !self.w_to_move,
            w_castle_k: self.b_castle_k,
            w_castle_q: self.b_castle_q,
            b_castle_k: self.w_castle_k,
            b_castle_q: self.w_castle_q,
            en_passant: { if self.en_passant == None { None } else { Some(flip_sq_ind_vertically(self.en_passant.unwrap())) } },
            halfmove_clock: self.halfmove_clock,
            fullmove_clock: self.fullmove_clock,
            pieces: self.pieces.iter().map(|&x| flip_vertically(x)).collect()
        }
    }

    pub fn get_piece(&self, sq_ind: usize) -> Option<usize> {
        // Get the piece at a given square index.
        let bit = sq_ind_to_bit(sq_ind);
        for i in 0..12 {
            if bit & self.pieces[i] != 0 {
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn is_legal(&self, move_gen: &MoveGen) -> bool {
        // Determines whether this position is legal, i.e. the side to move cannot capture the king.
        let king_sq_ind: usize;
        if self.w_to_move {
            king_sq_ind = bit_to_sq_ind(self.pieces[BK]);
            if king_sq_ind == 64 {
                println!("No black king");
                self.print();
            }
        } else {
            king_sq_ind = bit_to_sq_ind(self.pieces[WK]);
            if king_sq_ind == 64 {
                println!("No white king");
                self.print();
            }
        }
        return !self.is_square_attacked(king_sq_ind, self.w_to_move, move_gen);
    }

    pub(crate) fn is_square_attacked(&self, sq_ind: usize, by_white: bool, move_gen: &MoveGen) -> bool {
        // Find out if the square is attacked by a given side (white if by_white is true, black if by_white is false).
        if by_white {
            // Can the king reach an enemy bishop or queen by a bishop move?
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[WB] | self.pieces[WQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[WR] | self.pieces[WQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[WN]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.bp_capture_bitboard[sq_ind] & self.pieces[WP]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[WK]) != 0 {
                return true;
            }
            false
        } else {
            // Can the king reach an enemy bishop or queen by a bishop move?
            if (move_gen.gen_bishop_potential_captures(self, sq_ind) & (self.pieces[BB] | self.pieces[BQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy rook or queen by a rook move?
            if (move_gen.gen_rook_potential_captures(self, sq_ind) & (self.pieces[BR] | self.pieces[BQ])) != 0 {
                return true;
            }
            // Can the king reach an enemy knight by a knight move?
            if (move_gen.n_move_bitboard[sq_ind] & self.pieces[BN]) != 0 {
                return true;
            }
            // Can the king reach an enemy pawn by a pawn move?
            if (move_gen.wp_capture_bitboard[sq_ind] & self.pieces[BP]) != 0 {
                return true;
            }
            // Can the king reach an enemy king by a king move?
            if (move_gen.k_move_bitboard[sq_ind] & self.pieces[BK]) != 0 {
                return true;
            }
            false
        }
    }

}
