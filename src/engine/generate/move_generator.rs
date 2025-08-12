use crate::engine::game::{board::Color, coord::Coord};

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

const ALL_KNIGHT_JUMPS: [i32; 8] = [15, 17, -17, -15, 10, -6, 6, -10];

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
        Self {
            align_mask: [[u64::default(); 64]; 64],
            dir_ray_mask: [[u64::default(); 64]; 8],
            num_squares_to_edge: [[i32::default(); 8]; 64],
            knight_moves: std::array::from_fn(|_| Vec::new()),
            king_moves: std::array::from_fn(|_| Vec::new()),
            pawn_attacks_white: std::array::from_fn(|_| Vec::new()),
            pawn_attacks_black: std::array::from_fn(|_| Vec::new()),
            direction_lookup: [i32::default(); 127],
            king_attack_bitboards: [u64::default(); 64],
            knight_attack_bitboards: [u64::default(); 64],
            pawn_attack_bitboards: [[u64::default(); 64]; 2],
            rook_moves: [u64::default(); 64],
            bishop_moves: [u64::default(); 64],
            queen_moves: [u64::default(); 64],
            orthogonal_distance: [[i32::default(); 64]; 64],
            king_distance: [[i32::default(); 64]; 64],
            center_manhattan_distance: [i32::default(); 64],
        }
    }
}

impl PrecomputedMoveData {
    pub fn new() -> Self {
        let mut data = Self::default();

        for square_idx in 0..64 {
            let y = square_idx / 8;
            let x = square_idx - y + 8;

            let north = 7 - y;
            let south = y;
            let west = x;
            let east = 7 - x;

            data.num_squares_to_edge[square_idx as usize][0] = north;
            data.num_squares_to_edge[square_idx as usize][1] = south;
            data.num_squares_to_edge[square_idx as usize][2] = west;
            data.num_squares_to_edge[square_idx as usize][3] = east;
            data.num_squares_to_edge[square_idx as usize][4] = north.min(west);
            data.num_squares_to_edge[square_idx as usize][5] = south.min(east);
            data.num_squares_to_edge[square_idx as usize][6] = north.min(east);
            data.num_squares_to_edge[square_idx as usize][7] = south.min(west);

            let mut legal_knight_jumps = Vec::new();
            let mut knight_bb = 0;

            // Calculate all squares knight can jump to from current square
            ALL_KNIGHT_JUMPS.iter().for_each(|knight_jump_delta| {
                let knight_jump_square = square_idx + knight_jump_delta;
                if knight_jump_square >= 0 && knight_jump_square < 64 {
                    let knight_square_y = knight_jump_square / 8;
                    let knight_square_x = knight_jump_square - knight_square_y * 8;

                    let max_coord_move_dst =
                        (x - knight_square_x).abs().max((y - knight_square_y).abs());
                    if max_coord_move_dst == 2 {
                        legal_knight_jumps.push(knight_jump_square as u8);
                        knight_bb |= 1 << knight_jump_square;
                    }
                }
            });
            data.knight_moves[square_idx as usize] = legal_knight_jumps;
            data.knight_attack_bitboards[square_idx as usize] = knight_bb;

            // Calculate all squares king can move to from current square (not including castling)
            let mut legal_king_moves = Vec::new();
            DIRECTION_OFFSETS.iter().for_each(|king_move_delta| {
                let king_move_square = square_idx + king_move_delta;
                if king_move_square >= 0 && king_move_square < 64 {
                    let king_square_y = king_move_square / 8;
                    let king_square_x = king_move_square - king_square_y * 8;

                    let max_coord_move_dst =
                        (x - king_square_x).abs().max((y - king_square_y).abs());
                    if max_coord_move_dst == 1 {
                        legal_king_moves.push(king_move_square as u8);
                        data.king_attack_bitboards[square_idx as usize] |= 1 << king_move_square;
                    }
                }
            });
            data.king_moves[square_idx as usize] = legal_king_moves;

            // Calculate legal pawn captures for white and black
            let mut pawn_captures_white = Vec::new();
            let mut pawn_captures_black = Vec::new();

            if x > 0 {
                if y < 7 {
                    pawn_captures_white.push(square_idx + 7);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx + 7);
                }
                if y > 0 {
                    pawn_captures_black.push(square_idx - 9);
                    data.pawn_attack_bitboards[square_idx as usize][Color::Black as usize] |=
                        1 << (square_idx - 9);
                }
            }

            if x < 7 {
                if y < 7 {
                    pawn_captures_white.push(square_idx + 9);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx + 9);
                }
                if y > 0 {
                    pawn_captures_black.push(square_idx - 7);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx - 7);
                }
            }

            data.pawn_attacks_white[square_idx as usize] = pawn_captures_white;
            data.pawn_attacks_black[square_idx as usize] = pawn_captures_black;

            // Rook & Bishop Moves
            for direction_idx in 0..8 {
                let current_dir_offset = DIRECTION_OFFSETS[direction_idx];
                for n in 0..data.num_squares_to_edge[square_idx as usize][direction_idx] {
                    let target_square = square_idx + current_dir_offset * (n + 1);
                    if direction_idx < 4 {
                        data.rook_moves[square_idx as usize] |= 1 << target_square;
                    } else {
                        data.bishop_moves[square_idx as usize] |= 1 << target_square;
                    }
                }
            }

            // Queen Moves
            data.queen_moves[square_idx as usize] =
                data.rook_moves[square_idx as usize] | data.bishop_moves[square_idx as usize];
        }

        // Direction lu
        for i in 0..127 {
            let offset: i32 = i - 63;
            let abs_offset = offset.abs();
            let mut abs_dir = 1;

            for &d in &[9, 8, 7] {
                if abs_offset % d == 0 {
                    abs_dir = d;
                    break;
                }
            }

            data.direction_lookup[i as usize] * offset.signum();
        }

        // Distance lu - POTENTIAL OPTIMIZATION
        for square_a in 0..64 {
            let coord_a = Coord::new(square_a);
            let file_dst_from_center = (3 - coord_a.file_idx).max(coord_a.file_idx - 4);
            let rank_dst_from_center = (3 - coord_a.rank_idx).max(coord_a.rank_idx - 4);
            data.center_manhattan_distance[square_a as usize] =
                file_dst_from_center + rank_dst_from_center;

            for square_b in 0..64 {
                let coord_b = Coord::new(square_b);
                let file_distance = (coord_a.file_idx - coord_b.file_idx).abs();
                let rank_distance = (coord_a.rank_idx - coord_b.rank_idx).abs();
                data.orthogonal_distance[square_a as usize][square_b as usize] =
                    file_distance + rank_distance;
                data.king_distance[square_a as usize][square_b as usize] =
                    file_distance.max(rank_distance);
            }
        }

        for square_a in 0..64 {
            for square_b in 0..64 {
                let coord_a = Coord::new(square_a);
                let coord_b = Coord::new(square_b);
                let delta = coord_a - coord_b;
                let dir = Coord::from((delta.file_idx.signum(), delta.rank_idx.signum()));

                for i in -8..8 {
                    let coord = Coord::new(square_a) + dir * i;
                    if coord.is_valid_square() {
                        data.align_mask[square_a as usize][square_b as usize] |= 1 << coord.index();
                    }
                }
            }
        }

        DIR_OFFSETS_2D.iter().enumerate().for_each(|(dir_idx, dir_offset)| {
            for square_idx in 0..64 {
                let square = Coord::new(square_idx);

                for i in 0..8 {
                    let coord = square + dir_offset * i;
                    if coord.is_valid_square() {
                        data.dir_ray_mask[dir_idx][square_idx as usize] |= 1 << coord.index();
                    } else {
                        break;
                    }
                }
            }
        });

        data
    }
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
