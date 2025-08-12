use crate::engine::game::{board::Color, coord::Coord, piece};

pub const PAWNS: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const PAWNS_ENDGAME: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 80, 80, 80, 80, 80, 80, 80, 80, 50, 50, 50, 50, 50, 50, 50, 50, 30, 30,
    30, 30, 30, 30, 30, 30, 20, 20, 20, 20, 20, 20, 20, 20, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const ROOKS: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];
pub const KNIGHTS: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];
pub const BISHOPS: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];
pub const QUEENS: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];
pub const KING_START: [i32; 64] = [
    -80, -70, -70, -70, -70, -70, -70, -80, -60, -60, -60, -60, -60, -60, -60, -60, -40, -50, -50,
    -60, -60, -50, -50, -40, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, -5, -5, -5, -5, 20, 20, 20, 30, 10,
    0, 0, 10, 30, 20,
];

pub const KING_ENDGAME: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -5, 0, 5, 5, 5, 5, 0, -5, -10, -5, 20, 30, 30, 20, -5,
    -10, -15, -10, 35, 45, 45, 35, -10, -15, -20, -15, 30, 40, 40, 30, -15, -20, -25, -20, 20, 25,
    25, 20, -20, -25, -30, -25, 0, 0, 0, 0, -25, -30, -50, -30, -30, -30, -30, -30, -30, -50,
];

pub struct PieceSquareTable {
    tables: [[i32; 64]; piece::MAX_PIECE_INDEX + 1],
}

impl PieceSquareTable {
    pub fn new() -> Self {
        let mut tables = [[i32::default(); 64]; piece::MAX_PIECE_INDEX + 1];

        tables[piece::Piece::from(((piece::PAWN, piece::WHITE))).value as usize] = PAWNS;
        tables[piece::Piece::from(((piece::ROOK, piece::WHITE))).value as usize] = ROOKS;
        tables[piece::Piece::from(((piece::KNIGHT, piece::WHITE))).value as usize] = KNIGHTS;
        tables[piece::Piece::from(((piece::BISHOP, piece::WHITE))).value as usize] = BISHOPS;
        tables[piece::Piece::from(((piece::QUEEN, piece::WHITE))).value as usize] = QUEENS;

        tables[piece::Piece::from(((piece::PAWN, piece::BLACK))).value as usize] =
            get_flipped_table(PAWNS);
        tables[piece::Piece::from(((piece::ROOK, piece::BLACK))).value as usize] =
            get_flipped_table(ROOKS);
        tables[piece::Piece::from(((piece::KNIGHT, piece::BLACK))).value as usize] =
            get_flipped_table(KNIGHTS);
        tables[piece::Piece::from(((piece::BISHOP, piece::BLACK))).value as usize] =
            get_flipped_table(BISHOPS);
        tables[piece::Piece::from(((piece::QUEEN, piece::BLACK))).value as usize] =
            get_flipped_table(QUEENS);

        Self { tables: tables }
    }

    pub fn read(&self, piece: i32, square: i32) -> i32 {
        self.tables[piece as usize][square as usize]
    }
}

pub fn read(table: [i32; 64], square: i32, is_white: bool) -> i32 {
    let mut square = square;
    if is_white {
        let file = Coord::file_of_square(square);
        let rank = 7 - Coord::rank_of_square(square);
        square = Coord::from((file, rank)).index();
    }

    table[square as usize]
}

fn get_flipped_table(table: [i32; 64]) -> [i32; 64] {
    let mut flipped_table = [i32::default(); 64];

    for i in 0..64 {
        let coord = Coord::new(i);
        let flipped_coord = Coord::from((coord.file_idx, 7 - coord.rank_idx));
        flipped_table[flipped_coord.index() as usize] = table[i as usize];
    }

    flipped_table
}

pub struct PrecomputedEvalData {
    pub pawn_shield_squares: [[Vec<i32>; 64]; 2],
}

impl PrecomputedEvalData {
    pub fn new() -> Self {
        let mut data = Self {
            pawn_shield_squares: std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())),
        };
        for square_idx in 0..64 {
            data.create_pawn_shield_square(square_idx);
        }
        data
    }

    fn create_pawn_shield_square(&mut self, square_idx: i32) {
        let mut shield_indices_white = Vec::new();
        let mut shield_indices_black = Vec::new();

        let coord = Coord::new(square_idx);
        let rank = coord.rank_idx;
        let file = coord.file_idx.clamp(1, 6);

        for file_offset in -1..=1 {
            Self::add_if_valid(
                &Coord::from((file + file_offset, rank + 1)),
                &mut shield_indices_white,
            );
            Self::add_if_valid(
                &Coord::from((file + file_offset, rank - 1)),
                &mut shield_indices_black,
            );
        }

        for file_offset in -1..=1 {
            Self::add_if_valid(
                &Coord::from((file + file_offset, rank + 2)),
                &mut shield_indices_white,
            );
            Self::add_if_valid(
                &Coord::from((file + file_offset, rank - 2)),
                &mut shield_indices_black,
            );
        }

        self.pawn_shield_squares[Color::White as usize][square_idx as usize] = shield_indices_white;
        self.pawn_shield_squares[Color::Black as usize][square_idx as usize] = shield_indices_black;
    }

    fn add_if_valid(coord: &Coord, list: &mut Vec<i32>) {
        if coord.is_valid_square() {
            list.push(coord.index());
        }
    }
}
