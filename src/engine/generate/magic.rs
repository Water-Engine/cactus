use crate::engine::{
    game::{board, coord::Coord},
    generate::bitboard::BitBoard,
    utils::precomputed_magics::{BISHOP_MAGICS, BISHOP_SHIFTS, ROOK_MAGICS, ROOK_SHIFTS},
};

use std::sync::OnceLock;

static MAGIC: OnceLock<Magic> = OnceLock::new();

pub fn get_magic() -> &'static Magic {
    MAGIC.get_or_init(Magic::new)
}

pub struct Magic {
    pub rook_mask: [u64; 64],
    pub bishop_mask: [u64; 64],
    pub rook_attacks: [Vec<u64>; 64],
    pub bishop_attacks: [Vec<u64>; 64],
}

impl Magic {
    fn new() -> Self {
        let mut rook_mask = [u64::default(); 64];
        let mut bishop_mask = [u64::default(); 64];

        for square_idx in 0..64 {
            rook_mask[square_idx as usize] = Self::create_movement_mask(square_idx, true);
            bishop_mask[square_idx as usize] = Self::create_movement_mask(square_idx, false);
        }

        let mut rook_attacks: [Vec<u64>; 64] = std::array::from_fn(|_| Vec::new());
        let mut bishop_attacks: [Vec<u64>; 64] = std::array::from_fn(|_| Vec::new());

        for i in 0..64 {
            rook_attacks[i] = Self::create_table(i as i32, true, ROOK_MAGICS[i], ROOK_SHIFTS[i]);
            bishop_attacks[i] =
                Self::create_table(i as i32, false, BISHOP_MAGICS[i], BISHOP_SHIFTS[i]);
        }

        Self {
            rook_mask: rook_mask,
            bishop_mask: bishop_mask,
            rook_attacks: rook_attacks,
            bishop_attacks: bishop_attacks,
        }
    }

    pub fn get_slider_attacks(&self, square: i32, blockers: u64, ortho: bool) -> u64 {
        ortho
            .then(|| self.get_rook_attacks(square, blockers))
            .unwrap_or(self.get_bishop_attacks(square, blockers))
    }

    pub fn get_rook_attacks(&self, square: i32, blockers: u64) -> u64 {
        let square = square as usize;
        let key =
            (blockers & self.rook_mask[square]).wrapping_mul(ROOK_MAGICS[square]) >> ROOK_SHIFTS[square];
        self.rook_attacks[square][key as usize]
    }

    pub fn get_bishop_attacks(&self, square: i32, blockers: u64) -> u64 {
        let square = square as usize;
        let key = (blockers & self.bishop_mask[square]).wrapping_mul(BISHOP_MAGICS[square])
            >> BISHOP_SHIFTS[square];
        self.bishop_attacks[square][key as usize]
    }

    fn create_table(square: i32, rook: bool, magic: u64, left_shift: i32) -> Vec<u64> {
        let num_bits = 64 - left_shift;
        let lookup_size = 1 << num_bits;
        let mut table = vec![u64::default(); lookup_size];

        let movement_mask = Self::create_movement_mask(square, rook);
        let blocker_patterns = Self::create_all_blockers(movement_mask);

        for pattern in blocker_patterns {
            let idx = pattern.wrapping_mul(magic) >> (left_shift as u64);
            let moves = Self::legal_move_bb(square, pattern, rook);
            table[idx as usize] = moves;
        }

        table
    }
}

// Helper IMPL
impl Magic {
    pub fn create_all_blockers(movement_mask: u64) -> Vec<u64> {
        let mut move_square_indices = Vec::new();
        for i in 0..64 {
            if ((movement_mask >> i) & 1) == 1 {
                move_square_indices.push(i);
            }
        }

        let num_patterns = 1 << move_square_indices.len();
        let mut blocker_bbs = vec![u64::default(); num_patterns];

        for pattern_idx in 0..num_patterns {
            for bit_idx in 0..move_square_indices.len() {
                let bit = (pattern_idx >> bit_idx) & 1;
                blocker_bbs[pattern_idx] |= (bit << move_square_indices[bit_idx as usize]) as u64;
            }
        }

        blocker_bbs
    }

    pub fn create_movement_mask(square_idx: i32, ortho: bool) -> u64 {
        let mut mask = 0;
        let directions = ortho
            .then(|| board::ROOK_DIRECTIONS)
            .unwrap_or(board::BISHOP_DIRECTIONS);
        let start_coord = Coord::new(square_idx);

        directions.iter().for_each(|dir| {
            for dst in 1..8 {
                let coord = start_coord + dir * dst;
                let next_coord = start_coord + dir * (dst + 1);

                if next_coord.is_valid_square() {
                    BitBoard::set_square(&mut mask, coord.index());
                } else {
                    break;
                }
            }
        });

        mask
    }

    pub fn legal_move_bb(start_square: i32, blocker_bb: u64, ortho: bool) -> u64 {
        let mut bb = 0;

        let directions = ortho
            .then(|| board::ROOK_DIRECTIONS)
            .unwrap_or(board::BISHOP_DIRECTIONS);
        let start_coord = Coord::new(start_square);

        directions.iter().for_each(|dir| {
            for dst in 1..8 {
                let coord = start_coord + dir * dst;

                if coord.is_valid_square() {
                    BitBoard::set_square(&mut bb, coord.index());
                    if BitBoard::contains_square(blocker_bb, coord.index()) {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
        bb
    }
}
