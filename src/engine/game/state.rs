use crate::engine::game::{
    board::Board,
    piece::{self, MAX_PIECE_INDEX},
};

use rand::{RngCore, SeedableRng, rngs::StdRng};

pub const CLEAR_WHITE_KINGSIDE_MASK: i32 = 0b1110;
pub const CLEAR_WHITE_QUEENSIDE_MASK: i32 = 0b1101;
pub const CLEAR_BLACK_KINGSIDE_MASK: i32 = 0b1011;
pub const CLEAR_BLACK_QUEENSIDE_MASK: i32 = 0b0111;

const RNG_SEED: u64 = 29426028;

#[derive(Debug, Default, Clone, Copy)]
pub struct State {
    pub captured_piece_type: i32,
    pub en_passant_file: i32,
    pub castling_rights: i32,
    pub halfmove_clock: i32,
    pub zobrist: Zobrist,
}

impl State {
    pub fn new(
        captured_piece_type: i32,
        en_passant_file: i32,
        castling_rights: i32,
        halfmove_clock: i32,
        zobrist: Zobrist,
    ) -> Self {
        Self {
            captured_piece_type: captured_piece_type,
            en_passant_file: en_passant_file,
            castling_rights: castling_rights,
            halfmove_clock: halfmove_clock,
            zobrist: zobrist,
        }
    }

    pub fn can_castle_kingside(&self, white: bool) -> bool {
        let mask = if white { 1 } else { 4 };
        (self.castling_rights & mask) != 0
    }

    pub fn can_castle_queenside(&self, white: bool) -> bool {
        let mask = if white { 2 } else { 8 };
        (self.castling_rights & mask) != 0
    }
}

/**
Single 64-bit value used avoid reevaluating repeat positions
* Ref: https://en.wikipedia.org/wiki/Zobrist_hashing
*/
#[derive(Debug, Clone, Copy)]
pub struct Zobrist {
    pub key: u64,
    pub side_to_move: u64,
    pub pieces_array: [[u64; 64]; MAX_PIECE_INDEX + 1],

    /// Players have 4 possible rights: queen and/or kind side, none
    pub castling_rights: [u64; 16],

    // 0 == no en passant ignoring rank
    pub en_passant_file: [u64; 9],
}

impl Default for Zobrist {
    fn default() -> Self {
        Self {
            key: u64::default(),
            side_to_move: u64::default(),
            pieces_array: [[u64::default(); 64]; MAX_PIECE_INDEX + 1],
            castling_rights: [u64::default(); 16],
            en_passant_file: [u64::default(); 9],
        }
    }
}

impl Zobrist {
    pub fn new() -> Self {
        let mut pieces_array = [[u64::default(); 64]; MAX_PIECE_INDEX + 1];
        let mut castling_rights = [u64::default(); 16];
        let mut en_passant_file = [u64::default(); 9];

        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        for square_idx in 0..64 {
            for piece in piece::PIECE_INDICES {
                pieces_array[piece as usize][square_idx as usize] = random_u64(&mut rng);
            }
        }

        for i in 0..castling_rights.len() {
            castling_rights[i] = if i == 0 { 0 } else { random_u64(&mut rng) };
        }

        for i in 0..en_passant_file.len() {
            en_passant_file[i] = if i == 0 { 0 } else { random_u64(&mut rng) };
        }

        let side_to_move = random_u64(&mut rng);

        Self {
            key: 0,
            pieces_array: pieces_array,
            castling_rights: castling_rights,
            en_passant_file: en_passant_file,
            side_to_move: side_to_move,
        }
    }

    /// This is a costly function, use sparingly and incrementally update key when possible
    pub fn key(&mut self, board: &Board) {
        let mut key: u64 = 0;
        for square_idx in 0..64 {
            let piece = piece::Piece::from(board.squares[square_idx]);
            if piece.get_type() != piece::NONE {
                key ^= self.pieces_array[piece.value as usize][square_idx];
            }
        }

        self.key = key;
    }
}

pub fn random_u64(rng: &mut StdRng) -> u64 {
    rng.next_u64()
}
