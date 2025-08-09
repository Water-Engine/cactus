use crate::engine::game::coord::Coord;

pub const ROOK_DIRECTIONS: [Coord; 4] = [
    Coord::from((-1, 0)),
    Coord::from((1, 0)),
    Coord::from((0, 1)),
    Coord::from((0, -1)),
];
pub const BISHOP_DIRECTIONS: [Coord; 4] = [
    Coord::from((-1, 1)),
    Coord::from((1, 1)),
    Coord::from((1, -1)),
    Coord::from((-1, -1)),
];

pub const FILE_NAMES: &[char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANK_NAMES: &[char] = &['1', '2', '3', '4', '5', '6', '7', '8'];

pub const A1: usize = 0;
pub const B1: usize = 1;
pub const C1: usize = 2;
pub const D1: usize = 3;
pub const E1: usize = 4;
pub const F1: usize = 5;
pub const G1: usize = 6;
pub const H1: usize = 7;

pub const A8: usize = 56;
pub const B8: usize = 57;
pub const C8: usize = 58;
pub const D8: usize = 59;
pub const E8: usize = 60;
pub const F8: usize = 61;
pub const G8: usize = 62;
pub const H8: usize = 63;
