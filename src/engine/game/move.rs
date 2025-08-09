use super::piece::{Piece, ROOK, KNIGHT, BISHOP, QUEEN, NONE};

pub const NO_FLAG: i32 = 0b0000;
pub const EN_PASSANT_CAPTURE_FLAG: i32 = 0b0001;
pub const CASTLE_FLAG: i32 = 0b0010;
pub const PAWN_TWO_UP_FLAG: i32 = 0b0011;

pub const PROMOTE_TO_QUEEN_FLAG: i32 = 0b0100;
pub const PROMOTE_TO_KNIGHT_FLAG: i32 = 0b0101;
pub const PROMOTE_TO_ROOK_FLAG: i32 = 0b0110;
pub const PROMOTE_TO_BISHOP_FLAG: i32 = 0b0111;

const START_SQUARE_MASK: u16 = 0b0000000000111111;
const TARGET_SQUARE_MASK: u16 = 0b0000111111000000;

/**
Compact Move Representation (ffffttttttssssss)
* Bits 0-5: start square index
* Bits 6-11: target square index
* Bits 12-15: flag (promotion type, etc)
*/
#[derive(Debug, Eq, PartialEq)]
pub struct Move {
    pub value: u16,
}

impl From<u16> for Move {
    fn from(value: u16) -> Self {
        Self { value: value }
    }
}

impl From<(i32, i32)> for Move {
    fn from((start_square, target_square): (i32, i32)) -> Self {
        Self {
            value: (start_square | target_square << 6) as u16,
        }
    }
}

impl From<(i32, i32, i32)> for Move {
    fn from((start_square, target_square, flag): (i32, i32, i32)) -> Self {
        Self {
            value: (start_square | target_square << 6 | flag << 12) as u16,
        }
    }
}

impl Move {
    pub fn null() -> Self {
        Self { value: 0 }
    }

    pub fn is_null(&self) -> bool {
        self.value == 0
    }

    pub fn start_square(&self) -> i32 {
        (self.value & START_SQUARE_MASK) as i32
    }

    pub fn target_square(&self) -> i32 {
        (self.value & TARGET_SQUARE_MASK) as i32
    }

    pub fn move_flag(&self) -> i32 {
        (self.value >> 12) as i32
    }

    pub fn is_promotion(&self) -> bool {
        self.move_flag() >= PROMOTE_TO_QUEEN_FLAG
    }

    pub fn promotion_type(&self) -> Piece {
        Piece::from(match self.move_flag() {
            PROMOTE_TO_ROOK_FLAG => ROOK,
            PROMOTE_TO_KNIGHT_FLAG => KNIGHT,
            PROMOTE_TO_BISHOP_FLAG => BISHOP,
            PROMOTE_TO_QUEEN_FLAG => QUEEN,
            _ => NONE
        })
    }
}
