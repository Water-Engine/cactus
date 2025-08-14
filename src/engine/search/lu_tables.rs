use crate::engine::{
    game::{board::Board, r#move::Move},
    search::searcher,
};

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
            self.start_indices[self.count + 1] = reset
                .then(|| self.count)
                .unwrap_or(self.start_indices[self.count]);
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
    pub entries: Vec<Option<Entry>>,
    pub count: usize,
    pub enabled: bool,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let desired_table_size_bytes = size_mb * 1024 * 1024;
        let num_entries = desired_table_size_bytes / Entry::SIZE_BYTES;
        let entries = vec![None; num_entries];

        Self {
            entries: entries,
            count: num_entries,
            enabled: true,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.entries.len() {
            self.entries[i] = None;
        }
    }

    pub fn index(&self, board: &Board) -> usize {
        (board.state.zobrist.key % self.count as u64) as usize
    }

    pub fn try_get_stored_move(&self, board: &Board) -> Option<Move> {
        self.entries[self.index(board)].map(|e| e.mv)
    }

    pub fn lookup_evaluation(
        &self,
        board: &Board,
        depth: i32,
        ply_from_root: i32,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        if !self.enabled {
            return LOOKUP_FAILED;
        }

        if let Some(entry) = self.entries[self.index(board)] {
            // Only use stored evaluation if it has been searched to at least the same depth as would be searched now
            if entry.key == board.state.zobrist.key && entry.depth as i32 >= depth {
                let corrected_score =
                    Self::correct_retrieved_mate_score(entry.value, ply_from_root);
                return match entry.node_type as i32 {
                    // We have stored the exact evaluation for this position, so return it
                    EXACT => corrected_score,

                    // We have stored the upper bound of the eval for this position. If it's less than alpha then we don't need to
                    // search the moves in this position as they won't interest us; otherwise we will have to search to find the exact value
                    UPPER_BOUND if corrected_score <= alpha => corrected_score,

                    // We have stored the lower bound of the eval for this position. Only return if it causes a beta cut-off.
                    LOWER_BOUND if corrected_score >= beta => corrected_score,
                    _ => LOOKUP_FAILED,
                };
            }
        }
        return LOOKUP_FAILED;
    }

    pub fn store_evaluation(
        &mut self,
        mv: Move,
        board: &Board,
        depth: i32,
        num_ply_searched: i32,
        eval: i32,
        eval_type: i32,
    ) {
        if !self.enabled {
            return;
        }

        let value = Self::correct_mate_score_for_storage(eval, num_ply_searched);
        let entry = Entry::new(
            board.state.zobrist.key,
            value,
            mv,
            depth as u8,
            eval_type as u8,
        );
        let idx = self.index(board);
        self.entries[idx] = Some(entry);
    }
}

// Helper IMPL
impl TranspositionTable {
    fn correct_mate_score_for_storage(score: i32, num_ply_searched: i32) -> i32 {
        if searcher::is_mate_score(score) {
            let sign = score.signum();
            return (score * sign + num_ply_searched) * sign;
        }
        score
    }

    fn correct_retrieved_mate_score(score: i32, num_ply_searched: i32) -> i32 {
        if searcher::is_mate_score(score) {
            let sign = score.signum();
            return (score * sign - num_ply_searched) * sign;
        }
        return score;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Entry {
    pub key: u64,
    pub value: i32,
    pub mv: Move,
    pub depth: u8,
    pub node_type: u8,
}

impl Entry {
    pub const SIZE_BYTES: usize = std::mem::size_of::<Option<Entry>>();

    pub fn new(key: u64, value: i32, mv: Move, depth: u8, node_type: u8) -> Self {
        Self {
            key: key,
            value: value,
            mv: mv,
            depth: depth,
            node_type: node_type,
        }
    }
}
