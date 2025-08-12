use crate::engine::game::coord::Coord;

// Orthogonal and diagonal: N, S, W, E, NW, SE, NE, SW
pub const DIRECTION_OFFSETS: [i32; 8] = [8, -8, -1, 1, 7, -7, 9, -9];

const DIR_OFFSETS_2D: [Coord; 8] = [
    Coord::from((0, 1)),
    Coord::from((0, -1)),
    Coord::from((-1, 0)),
    Coord::from((1, 0)),
    Coord::from((-1, 1)),
    Coord::from((1, -1)),
    Coord::from((1, 1)),
    Coord::from((-1, -1)),
];

pub const PAWN_ATTACK_DIRECTIONS: [[u8; 2]; 2] = [[4, 6], [7, 5]];

#[derive(Debug)]
pub struct PrecomputedMoveData {
    pub align_mask: [[u64; 64]; 64],
    pub dir_ray_mask: [[u64; 64]; 8],

    pub num_squares_to_edge: [[i32; 8]; 64],

    pub knight_moves: [Vec<u8>; 64],
    pub king_moves: [Vec<u8>; 64],

    pub pawn_attacks_white: [Vec<i32>; 64],
    pub pawn_attacks_black: [Vec<i32>; 64],
    pub direction_lookup: [i32; 127],

    pub king_attack_bitboards: [u64; 64],
    pub knight_attack_bitboards: [u64; 64],
    pub pawn_attack_bitboards: [[u64; 64]; 2],

    pub rook_moves: [u64; 64],
    pub bishop_moves: [u64; 64],
    pub queen_moves: [u64; 64],

    pub orthogonal_distance: [[i32; 64]; 64],
    pub king_distance: [[i32; 64]; 64],
    pub center_manhattan_distance: [i32; 64],
}

impl Default for PrecomputedMoveData {
    fn default() -> Self {
        
    }
}

impl PrecomputedMoveData {

}

// Helper IMPL
impl PrecomputedMoveData {
    pub fn num_rook_moves_to_reach_square(&self, start_square: i32, target_square: i32) -> i32 {
        self.orthogonal_distance[start_square as usize][target_square as usize]
    }

    pub fn num_king_moves_to_reach_square(&self, start_square: i32, target_square: i32) -> i32 {
        self.king_distance[start_square as usize][target_square as usize]
    }
}
