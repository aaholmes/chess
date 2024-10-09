/// Represents a chess move.
///
/// This struct contains information about the source square, destination square,
/// and any promotion that occurs as a result of the move.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    /// The index of the square the piece is moving from (0-63).
    pub from: usize,
    /// The index of the square the piece is moving to (0-63).
    pub to: usize,
    /// The type of piece to promote to, if this move results in a promotion.
    /// `None` if the move does not result in a promotion.
    pub promotion: Option<usize>
}

impl Move {
    /// Creates a new `Move` instance.
    ///
    /// # Arguments
    ///
    /// * `from` - The index of the source square (0-63).
    /// * `to` - The index of the destination square (0-63).
    /// * `promotion` - The type of piece to promote to, if applicable. Use `None` if not a promotion.
    ///
    /// # Returns
    ///
    /// A new `Move` instance with the specified parameters.
    pub fn new(from: usize, to: usize, promotion: Option<usize>) -> Move {
        // New move
        Move {
            from,
            to,
            promotion
        }
    }

    /// Creates a null move.
    ///
    /// A null move is a special move used in chess engines to pass the turn
    /// without making an actual move on the board. It's typically used in
    /// null move pruning and other search techniques.
    ///
    /// # Returns
    ///
    /// A `Move` set to 0
    /// and `promotion` set to `None`.
    pub fn null() -> Move {
        // Null move
        Move {
            from: 0,
            to: 0,
            promotion: None
        }
    }
}