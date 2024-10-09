// Struct representing a move
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<usize>
}

impl Move {
    pub fn new(from: usize, to: usize, promotion: Option<usize>) -> Move {
        // New move
        Move {
            from,
            to,
            promotion
        }
    }

    pub fn null() -> Move {
        // Null move
        Move {
            from: 0,
            to: 0,
            promotion: None
        }
    }
}