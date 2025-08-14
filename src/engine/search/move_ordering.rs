use crate::engine::{game::r#move::Move, generate::move_generator::MAX_MOVES};

pub const MAX_KILLER_MOVE_PLY: usize = 32;

const MILLION: i32 = 1_000_000;
const HASH_MOVE_SCORE: i32 = 100 * MILLION;
const WINNING_CAPTURE_BIAS: i32 = 8 * MILLION;
const PROMOTE_BIAS: i32 = 6 * MILLION;
const KILLER_BIAS: i32 = 4 * MILLION;
const LOSING_CAPTURE_BIAS: i32 = 2 * MILLION;
const REGULAR_BIAS: i32 = 0;

pub struct MoveOrdering {
    move_scores: [i32; MAX_MOVES],
    killer_moves: [Killers; MAX_KILLER_MOVE_PLY],
    history: [[[i32; 64]; 64]; 2],
}

impl MoveOrdering {
    pub fn new() -> Self {
        Self {
            move_scores: [i32::default(); MAX_MOVES],
            killer_moves: std::array::from_fn(|_| Killers::default()),
            history: [[[i32::default(); 64]; 64]; 2],
        }
    }

    pub fn clear_history(&mut self) {
        self.history = [[[i32::default(); 64]; 64]; 2];
    }

    pub fn clear_killers(&mut self) {
        self.killer_moves = std::array::from_fn(|_| Killers::default());
    }
}

#[derive(Debug, Default)]
pub struct Killers {
    pub move_a: Move,
    pub move_b: Move,
}

impl Killers {
    pub fn new(move_a: Move, move_b: Move) -> Self {
        Self {
            move_a: move_a,
            move_b: move_b,
        }
    }

    pub fn add(&mut self, mv: Move) {
        if mv.value != self.move_a.value {
            self.move_b = self.move_a;
            self.move_a = mv;
        }
    }

    pub fn matches(&self, mv: Move) -> bool {
        mv.value == self.move_a.value || mv.value == self.move_b.value
    }
}
