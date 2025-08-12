use std::ops::{Index, IndexMut};

use crate::engine::game::board::Color;

pub const NONE: i32 = 0;
pub const PAWN: i32 = 1;
pub const KNIGHT: i32 = 2;
pub const BISHOP: i32 = 3;
pub const ROOK: i32 = 4;
pub const QUEEN: i32 = 5;
pub const KING: i32 = 6;

pub const WHITE: i32 = 0;
pub const BLACK: i32 = 8;

pub const WHITE_PAWN: i32 = PAWN | WHITE; // 1
pub const WHITE_KNIGHT: i32 = KNIGHT | WHITE; // 2
pub const WHITE_BISHOP: i32 = BISHOP | WHITE; // 3
pub const WHITE_ROOK: i32 = ROOK | WHITE; // 4
pub const WHITE_QUEEN: i32 = QUEEN | WHITE; // 5
pub const WHITE_KING: i32 = KING | WHITE; // 6

pub const BLACK_PAWN: i32 = PAWN | BLACK; // 9
pub const BLACK_KNIGHT: i32 = KNIGHT | BLACK; // 10
pub const BLACK_BISHOP: i32 = BISHOP | BLACK; // 11
pub const BLACK_ROOK: i32 = ROOK | BLACK; // 12
pub const BLACK_QUEEN: i32 = QUEEN | BLACK; // 13
pub const BLACK_KING: i32 = KING | BLACK; // 14

pub const MAX_PIECE_INDEX: usize = BLACK_KING as usize;

pub const PIECE_INDICES: [i32; 12] = [
    WHITE_PAWN,
    WHITE_KNIGHT,
    WHITE_BISHOP,
    WHITE_ROOK,
    WHITE_QUEEN,
    WHITE_KING,
    BLACK_PAWN,
    BLACK_KNIGHT,
    BLACK_BISHOP,
    BLACK_ROOK,
    BLACK_QUEEN,
    BLACK_KING,
];

const TYPE_MASK: i32 = 0b0111;
const COLOR_MASK: i32 = 0b1000;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Piece {
    pub value: i32,
}

impl From<i32> for Piece {
    fn from(value: i32) -> Self {
        Self { value: value }
    }
}

impl From<(i32, i32)> for Piece {
    fn from((r#type, color): (i32, i32)) -> Self {
        Self {
            value: r#type | color,
        }
    }
}

impl From<(i32, bool)> for Piece {
    fn from((r#type, is_white): (i32, bool)) -> Self {
        let color = if is_white { WHITE } else { BLACK };
        Self::from((r#type, color))
    }
}

impl From<char> for Piece {
    fn from(value: char) -> Self {
        Self {
            value: match value {
                'R' | 'r' => ROOK,
                'N' | 'n' => KNIGHT,
                'B' | 'b' => BISHOP,
                'Q' | 'q' => QUEEN,
                'K' | 'k' => KING,
                'P' | 'p' => PAWN,
                _ => NONE,
            },
        }
    }
}

impl Into<char> for Piece {
    fn into(self) -> char {
        self.get_symbol()
    }
}

impl Piece {
    pub fn is_color(&self, color: i32) -> bool {
        (self.value & COLOR_MASK) == color && (self.value != 0)
    }

    pub fn is_white(&self) -> bool {
        self.is_color(WHITE)
    }

    pub fn get_color(&self) -> i32 {
        self.value & COLOR_MASK
    }

    pub fn get_type(&self) -> i32 {
        self.value & TYPE_MASK
    }

    pub fn get_symbol(&self) -> char {
        match (self.get_type(), self.get_color()) {
            (ROOK, BLACK) => 'r',
            (KNIGHT, BLACK) => 'n',
            (BISHOP, BLACK) => 'b',
            (QUEEN, BLACK) => 'q',
            (KING, BLACK) => 'k',
            (PAWN, BLACK) => 'p',
            (ROOK, WHITE) => 'R',
            (KNIGHT, WHITE) => 'N',
            (BISHOP, WHITE) => 'B',
            (QUEEN, WHITE) => 'Q',
            (KING, WHITE) => 'K',
            (PAWN, WHITE) => 'P',
            _ => ' ',
        }
    }

    pub fn can_ortho_slide(&self) -> bool {
        match self.get_type() {
            QUEEN | ROOK => true,
            _ => false,
        }
    }

    pub fn can_diag_slide(&self) -> bool {
        match self.get_type() {
            QUEEN | BISHOP => true,
            _ => false,
        }
    }

    pub fn can_slide(&self) -> bool {
        match self.get_type() {
            QUEEN | BISHOP | ROOK => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PieceList {
    pub occupied_squares: [i32; 16],
    map: [usize; 64],
    num_pieces: usize,
}

impl Default for PieceList {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceList {
    pub fn new() -> Self {
        Self {
            occupied_squares: [0; 16],
            map: [0; 64],
            num_pieces: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.num_pieces
    }

    pub fn add_piece(&mut self, square: i32) {
        self.occupied_squares[self.num_pieces] = square;
        self.map[square as usize] = self.num_pieces;
        self.num_pieces += 1;
    }

    pub fn remove_piece(&mut self, square: i32) {
        let idx = self.map[square as usize] as usize;
        self.occupied_squares[idx] = self.occupied_squares[self.num_pieces - 1];
        self.map[self.occupied_squares[idx] as usize] = idx;
        self.num_pieces -= 1;
    }

    pub fn move_piece(&mut self, start_square: i32, target_square: i32) {
        let idx = self.map[start_square as usize];
        self.occupied_squares[idx] = target_square;
        self.map[target_square as usize] = idx;
    }
}

impl Index<usize> for PieceList {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.occupied_squares[index]
    }
}

impl IndexMut<usize> for PieceList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.occupied_squares[index]
    }
}
