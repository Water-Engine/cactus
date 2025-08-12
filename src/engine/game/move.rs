use crate::engine::game::{board::Board, coord::Coord, piece};

use super::piece::{BISHOP, KNIGHT, NONE, Piece, QUEEN, ROOK};

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
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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
            _ => NONE,
        })
    }
}

// Helper IMPL
impl Move {
    pub fn from_uci(board: &Board, move_name: String) -> Self {
        let start_coord = Coord::from_string(move_name[0..3].to_string());
        let start_square: i32 = start_coord.index();
        let target_coord = Coord::from_string(move_name[2..5].to_string());
        let target_square: i32 = target_coord.index();

        let moved_piece_type = Piece::from(board.squares[start_square as usize]).get_type();
        let mut flag = NO_FLAG;

        if moved_piece_type == piece::PAWN {
            if move_name.len() > 4 {
                flag = match move_name.chars().nth(move_name.len() - 1) {
                    Some('q') => PROMOTE_TO_QUEEN_FLAG,
                    Some('r') => PROMOTE_TO_ROOK_FLAG,
                    Some('n') => PROMOTE_TO_KNIGHT_FLAG,
                    Some('b') => PROMOTE_TO_BISHOP_FLAG,
                    _ => NO_FLAG,
                };
            } else if (target_coord.rank_idx - start_coord.rank_idx).abs() == 2 {
                flag = PAWN_TWO_UP_FLAG;
            } else if start_coord.file_idx != target_coord.file_idx
                && board.squares[target_square as usize] == piece::NONE
            {
                flag = EN_PASSANT_CAPTURE_FLAG;
            }
        } else if moved_piece_type == piece::KING {
            if (start_coord.file_idx - target_coord.file_idx).abs() > 1 {
                flag = CASTLE_FLAG;
            }
        }

        Self::from((start_square, target_square, flag))
    }

    pub fn to_uci(&self) -> String {
        let start_square_name = Coord::new(self.start_square()).to_string();
        let target_square_name = Coord::new(self.target_square()).to_string();
        let mut move_name = start_square_name + &target_square_name;

        if self.is_promotion() {
            match self.move_flag() {
                PROMOTE_TO_ROOK_FLAG => move_name.push('r'),
                PROMOTE_TO_KNIGHT_FLAG => move_name.push('n'),
                PROMOTE_TO_BISHOP_FLAG => move_name.push('b'),
                PROMOTE_TO_QUEEN_FLAG => move_name.push('q'),
                _ => {}
            }
        }

        move_name
    }

    pub fn from_san(board: &Board, algebraic_move: String) -> Self {
        todo!("Not implemented")
    }

    pub fn to_san(&self) -> String {
        todo!("Not implemented")
    }
}
