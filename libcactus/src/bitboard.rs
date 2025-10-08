pub struct ChessPieces {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_queens: u64,
    pub white_king: u64,

    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_queens: u64,
    pub black_king: u64,
}

impl ChessPieces {
    pub fn new() -> Self {
        Self {
            white_pawns: 0x000000000000FF00,   // Rank 2
            white_knights: 0x0000000000000042, // b1, g1
            white_bishops: 0x0000000000000024, // c1, f1
            white_rooks: 0x0000000000000081,   // a1, h1
            white_queens: 0x0000000000000008,  // d1
            white_king: 0x0000000000000010,    // e1

            black_pawns: 0x00FF000000000000,   // Rank 7
            black_knights: 0x4200000000000000, // b8, g8
            black_bishops: 0x2400000000000000, // c8, f8
            black_rooks: 0x8100000000000000,   // a8, h8
            black_queens: 0x0800000000000000,  // d8
            black_king: 0x1000000000000000,    // e8
        }
    }

    pub fn empty() -> Self {
        Self {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,

            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,
        }
    }
}
