use std::time::{Duration, Instant};

use crate::engine::{
    game::{board::Board, r#move::Move},
    generate::move_generator::MoveGenerator,
    search::{
        lu_tables::{RepetitionTable, TranspositionTable},
        move_ordering::MoveOrdering,
    },
};

const TRANSPOSITION_TABLE_SIZE_MB: usize = 64;
const MAX_EXTENSIONS: i32 = 16;

const IMMEDIATE_MATE_SCORE: i32 = 100000;
const POSITIVE_INFINITY: i32 = 9999999;
const NEGATIVE_INFINITY: i32 = -POSITIVE_INFINITY;

pub struct Searcher {
    // State
    pub current_depth: i32,
    is_playing_white: bool,
    best_move_this_iteration: Move,
    best_eval_this_iteration: i32,
    best_move: Move,
    best_eval: i32,
    has_searched_at_least_one_move: bool,
    search_canceled: bool,

    // Diagnostics
    pub search_diagnostics: SearchDiagnostics,
    current_iteration_depth: i32,
    search_timer: SearchTimer,
    debug_info: String,

    // References
    transposition_table: TranspositionTable,
    repetition_table: RepetitionTable,
    move_generator: Option<MoveGenerator>,
    move_orderer: MoveOrdering,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            current_depth: Default::default(),
            is_playing_white: Default::default(),
            best_move_this_iteration: Default::default(),
            best_eval_this_iteration: Default::default(),
            best_move: Default::default(),
            best_eval: Default::default(),
            has_searched_at_least_one_move: Default::default(),
            search_canceled: Default::default(),
            search_diagnostics: Default::default(),
            current_iteration_depth: Default::default(),
            search_timer: Default::default(),
            debug_info: Default::default(),
            transposition_table: TranspositionTable::new(TRANSPOSITION_TABLE_SIZE_MB),
            repetition_table: RepetitionTable::new(),
            move_orderer: MoveOrdering::new(),
            move_generator: None,
        }
    }

    pub fn clear_for_new_position(&mut self) {
        self.transposition_table.clear();
        self.move_orderer.clear_killers();
    }

    pub fn start_search(&mut self) {
        todo!("Not implemented")
    }

    pub fn end_search(&mut self) {
        todo!("Not implemented")
    }
}

pub fn is_mate_score(score: i32) -> bool {
    todo!("Not implemented")
}

#[derive(Debug, Default)]
pub struct SearchDiagnostics {
    pub num_completed_iterations: i32,
    pub num_positions_evaluated: i32,
    pub num_cutoffs: u64,

    pub move_val: String,
    pub mv: String,
    pub eval: i32,
    pub move_is_from_partial_search: bool,
    pub num_q_checks: i32,
    pub num_q_mates: i32,

    pub is_book: bool,
    pub max_extension_reached_in_search: i32,
}

#[derive(Debug)]
pub struct SearchTimer {
    pub iteration_timer: Instant,
    pub total_timer: Instant,
}

impl Default for SearchTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            iteration_timer: now,
            total_timer: now,
        }
    }

    pub fn restart_iteration_timer(&mut self) {
        self.iteration_timer = Instant::now();
    }

    pub fn elapsed_iteration(&self) -> Duration {
        self.iteration_timer.elapsed()
    }

    pub fn elapsed_total(&self) -> Duration {
        self.total_timer.elapsed()
    }
}
