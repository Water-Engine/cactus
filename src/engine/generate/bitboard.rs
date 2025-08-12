use crate::engine::game::{board, coord::Coord};

pub const FILE_A: u64 = 0x101010101010101;

pub const WHITE_KINGSIDE_MASK: u64 = 1 << board::F1 | 1 << board::G1;
pub const BLACK_KINGSIDE_MASK: u64 = 1 << board::F8 | 1 << board::G8;

pub const WHITE_QUEENSIDE_MASK2: u64 = 1 << board::D1 | 1 << board::C1;
pub const BLACK_QUEENSIDE_MASK2: u64 = 1 << board::D8 | 1 << board::C8;

pub const WHITE_QUEENSIDE_MASK: u64 = WHITE_QUEENSIDE_MASK2 | 1 << board::B1;
pub const BLACK_QUEENSIDE_MASK: u64 = BLACK_QUEENSIDE_MASK2 | 1 << board::B8;

pub const RANK_1: u64 = 0b11111111;
pub const RANK_2: u64 = RANK_1 << 8;
pub const RANK_3: u64 = RANK_2 << 8;
pub const RANK_4: u64 = RANK_3 << 8;
pub const RANK_5: u64 = RANK_4 << 8;
pub const RANK_6: u64 = RANK_5 << 8;
pub const RANK_7: u64 = RANK_6 << 8;
pub const RANK_8: u64 = RANK_7 << 8;

pub const NOT_A_FILE: u64 = !FILE_A;
pub const NOT_H_FILE: u64 = !(FILE_A << 7);

const ORTHO_DIR: [(i32, i32); 4] = [(-1, 0), (0, 1), (1, 0), (0, -1)];
const DIAG_DIR: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, 1), (1, -1)];
const KNIGHT_JUMPS: [(i32, i32); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, 2),
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
];

#[derive(Debug)]
pub struct BitMasks {
    pub white_passed_pawn: [u64; 64],
    pub black_passed_pawn: [u64; 64],

    pub white_pawn_support: [u64; 64],
    pub black_pawn_support: [u64; 64],

    pub file: [u64; 8],
    pub adj_file: [u64; 8],

    pub king_safety: [u64; 64],

    pub white_forward_file: [u64; 64],
    pub black_forward_file: [u64; 64],

    pub triple_file: [u64; 8],
}

impl Default for BitMasks {
    fn default() -> Self {
        BitMasks::new()
    }
}

impl BitMasks {
    pub fn new() -> Self {
        let mut file_mask = [u64::default(); 8];
        let mut adj_file_mask = [u64::default(); 8];

        for i in 0..8 {
            file_mask[i] = FILE_A << i;
            let left = if i > 0 { FILE_A << (i - 1) } else { 0 };
            let right = if i < 7 { FILE_A << (i + 1) } else { 0 };
            adj_file_mask[i] = left | right;
        }

        let mut triple_file_mask = [u64::default(); 8];
        for i in 0..8 {
            let clamped = i.clamp(1, 6);
            triple_file_mask[i] = file_mask[clamped] | adj_file_mask[clamped];
        }

        let mut white_passed_pawn_mask = [u64::default(); 64];
        let mut black_passed_pawn_mask = [u64::default(); 64];
        let mut white_pawn_support_mask = [u64::default(); 64];
        let mut black_pawn_support_mask = [u64::default(); 64];
        let mut white_forward_file_mask = [u64::default(); 64];
        let mut black_forward_file_mask = [u64::default(); 64];

        for square in 0..64 {}

        let mut king_safety_mask = [u64::default(); 64];
        for i in 0..64 {}

        Self {
            white_passed_pawn: white_passed_pawn_mask,
            black_passed_pawn: black_passed_pawn_mask,
            white_pawn_support: white_pawn_support_mask,
            black_pawn_support: black_pawn_support_mask,
            file: file_mask,
            adj_file: adj_file_mask,
            king_safety: king_safety_mask,
            white_forward_file: white_forward_file_mask,
            black_forward_file: black_forward_file_mask,
            triple_file: triple_file_mask,
        }
    }
}

#[derive(Debug)]
pub struct BitBoard {
    pub knight_attacks: [u64; 64],
    pub king_moves: [u64; 64],
    pub white_pawn_attacks: [u64; 64],
    pub black_pawn_attacks: [u64; 64],
}

impl Default for BitBoard {
    fn default() -> Self {
        BitBoard::new()
    }
}

impl BitBoard {
    pub fn new() -> Self {
        let mut bb = Self {
            knight_attacks: [u64::default(); 64],
            king_moves: [u64::default(); 64],
            white_pawn_attacks: [u64::default(); 64],
            black_pawn_attacks: [u64::default(); 64],
        };

        for y in 0..8 {
            for x in 0..8 {
                bb.process_square(x, y);
            }
        }
        bb
    }

    fn process_square(&mut self, x: i32, y: i32) {
        let c = Coord::from((x, y));
        let square_idx = c.index() as usize;

        for dir_idx in 0..4 {
            for dst in 1..8 {
                let ortho_x = x + ORTHO_DIR[dir_idx].0 * dst;
                let ortho_y = y + ORTHO_DIR[dir_idx].1 * dst;
                let diag_x = x + DIAG_DIR[dir_idx].0 * dst;
                let diag_y = y + DIAG_DIR[dir_idx].1 * dst;

                match Self::valid_square_idx(ortho_x, ortho_y) {
                    Some(target_idx) if dst == 1 => self.king_moves[square_idx] |= 1 << target_idx,
                    _ => {}
                }

                match Self::valid_square_idx(diag_x, diag_y) {
                    Some(target_idx) if dst == 1 => self.king_moves[square_idx] |= 1 << target_idx,
                    _ => {}
                }
            }

            for i in 0..KNIGHT_JUMPS.len() {
                let knight_x = x + KNIGHT_JUMPS[i].0;
                let knight_y = y + KNIGHT_JUMPS[i].1;
                if let Some(target_square) = Self::valid_square_idx(knight_x, knight_y) {
                    self.knight_attacks[square_idx] |= 1 << target_square;
                }
            }

            if let Some(white_pawn_right) = Self::valid_square_idx(x + 1, y + 1) {
                self.white_pawn_attacks[square_idx] |= 1 << white_pawn_right;
            }

            if let Some(white_pawn_left) = Self::valid_square_idx(x + 1, y + 1) {
                self.white_pawn_attacks[square_idx] |= 1 << white_pawn_left;
            }

            if let Some(black_pawn_right) = Self::valid_square_idx(x + 1, y + 1) {
                self.black_pawn_attacks[square_idx] |= 1 << black_pawn_right;
            }

            if let Some(black_pawn_left) = Self::valid_square_idx(x + 1, y + 1) {
                self.black_pawn_attacks[square_idx] |= 1 << black_pawn_left;
            }
        }
    }

    fn valid_square_idx(x: i32, y: i32) -> Option<i32> {
        let c = Coord::from((x, y));
        if c.is_valid_square() {
            Some(c.index())
        } else {
            None
        }
    }
}

// Helper IMPL
impl BitBoard {
    pub fn pop_lsb(b: &mut u64) -> u32 {
        let i = b.trailing_zeros();
        *b &= *b - 1;
        i
    }

    pub fn set_square(bb: &mut u64, square_idx: i32) {
        *bb |= 1 << square_idx;
    }

    pub fn clear_square(bb: &mut u64, square_idx: i32) {
        *bb &= !(1 << square_idx);
    }

    pub fn toggle_square(bb: &mut u64, square_idx: i32) {
        *bb ^= 1 << square_idx;
    }

    pub fn toggle_squares(bb: &mut u64, square_idxs: &[i32]) {
        *bb ^= square_idxs.iter().fold(0u64, |acc, sq| acc | (1 << sq));
    }

    pub fn contains_square(bb: u64, square_idx: i32) -> bool {
        ((bb >> square_idx) & 1) != 0
    }

    pub fn pawn_attacks(pawn_bb: u64, is_white: bool) -> u64 {
        if is_white {
            ((pawn_bb << 9) & NOT_A_FILE) | ((pawn_bb << 7) & NOT_H_FILE)
        } else {
            ((pawn_bb >> 7) & NOT_A_FILE) | ((pawn_bb >> 9) & NOT_H_FILE)
        }
    }

    pub fn shift(bb: u64, num_to_shift: i32) -> u64 {
        if num_to_shift > 0 {
            bb << num_to_shift
        } else {
            bb >> -num_to_shift
        }
    }
}
