use crate::engine::game::board::Board;

const HASHES_LEN: usize = 256;

pub struct RepetitionTable {
    pub hashes: [u64; HASHES_LEN],
    pub start_indices: [usize; HASHES_LEN + 1],
    count: usize,
}

impl RepetitionTable {
    pub fn new(board: &Board) -> Self {
        let mut table = Self {
            hashes: [u64::default(); HASHES_LEN],
            start_indices: [usize::default(); HASHES_LEN + 1],
            count: 0,
        };

        let initial_hashes: Vec<&u64> = board.repetition_history.iter().rev().collect();
        table.count = initial_hashes.len();

        for i in 0..initial_hashes.len() {
            table.hashes[i] = *initial_hashes[i];
            table.start_indices[i] = 0;
        }
        table.start_indices[table.count] = 0;

        table
    }

    pub fn push(&mut self, hash: u64, reset: bool) {
        if self.count < self.hashes.len() {
            self.hashes[self.count] = hash;
            self.start_indices[self.count + 1] = reset.then(|| self.count).unwrap_or(self.start_indices[self.count]);
        }
        self.count += 1;
    }

    pub fn try_pop(&mut self) {
        self.count = 0.max(self.count - 1);
    }

    pub fn contains(&self, hash: u64) -> bool {
        let start = self.start_indices[self.count];

        for i in start..self.count {
            if self.hashes[i] == hash {
                return true;
            }
        }

        return false;
    }
}

pub const LOOKUP_FAILED: i32 = -1;
pub const EXACT: i32 = 0;
pub const LOWER_BOUND: i32 = 1;
pub const UPPER_BOUND: i32 = 2;

/// Reference: https://web.archive.org/web/20071031100051/http://www.brucemo.com/compchess/programming/hashing.htm
pub struct TranspositionTable {

}