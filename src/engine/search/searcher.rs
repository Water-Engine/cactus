use std::time::{Duration, Instant};

use crate::engine::{
    game::{board::Board, coord::Coord, r#move::Move, piece},
    search::{
        lu_tables::{self, RepetitionTable, TranspositionTable},
        move_ordering::{self, MoveOrdering},
    },
};

const TRANSPOSITION_TABLE_SIZE_MB: usize = 64;
const MAX_EXTENSIONS: i32 = 16;

const IMMEDIATE_MATE_SCORE: i32 = 100000;
const POSITIVE_INFINITY: i32 = 9999999;
const NEGATIVE_INFINITY: i32 = -POSITIVE_INFINITY;

const REDUCE_DEPTH: i32 = 1;
const MAX_MATE_DEPTH: i32 = 1000;
const MAX_PLY: i32 = 256;

#[derive(Debug)]
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
        }
    }

    pub fn clear_for_new_position(&mut self) {
        self.transposition_table.clear();
        self.move_orderer.clear_killers();
    }

    pub fn start_search(&mut self, board: &mut Board, time_ms: i32) {
        self.best_eval_this_iteration = 0;
        self.best_eval = 0;

        self.best_move_this_iteration = Move::null();
        self.best_move = Move::null();

        self.is_playing_white = board.white_to_move;

        self.move_orderer.clear_history();
        self.repetition_table = RepetitionTable::init(board);

        self.current_depth = 0;
        self.debug_info = format!("Starting search with FEN {}", board.current_fen(true));
        self.search_canceled = false;
        self.search_diagnostics = SearchDiagnostics::default();
        self.search_timer = SearchTimer::with_limit(time_ms);

        self.naive_search(board);

        if self.best_move.is_null() {
            let (moves, _) = board.generate_moves(false);
            self.best_move = moves[0];
        }
        self.search_canceled = false;
    }

    pub fn end_search(&mut self) {
        self.search_canceled = true;
    }

    pub fn bests(&self) -> (i32, Move) {
        (self.best_eval, self.best_move)
    }

    pub fn flush_log(&mut self) -> String {
        let log = self.debug_info.clone();
        self.debug_info = String::new();
        log
    }
}

// Helper IMPL
impl Searcher {
    fn naive_search(&mut self, board: &mut Board) {
        let start_time = Instant::now();
        let (mut moves, mg) = board.generate_moves(false);
        self.move_orderer.order_moves(
            board,
            Move::null(),
            &mut moves,
            mg.opponent_attack_map,
            mg.opponent_pawn_attack_map,
            false,
            0,
        );
        self.best_move = moves[0];
        for mv in &moves {
            if self.search_canceled || start_time.elapsed() >= self.search_timer.time_limit {
                break;
            }
            board.make_move(*mv, true);

            // Legal responses?
            let (mut opponent_moves, _) = board.generate_moves(false);
            let mut illegal = Vec::new();
            for response in &opponent_moves {
                if self.search_canceled || start_time.elapsed() >= self.search_timer.time_limit {
                    break;
                }
                board.make_move(*response, true);
                board.make_null_move(); // evaluate check state from the opponents perspective

                if board.calculate_in_check_state() {
                    illegal.push(*response);
                }

                board.unmake_null_move(); // restore move order
                board.unmake_move(*response, true);
            }

            opponent_moves.retain(|choice| !illegal.contains(choice));
            if opponent_moves.len() == 0 {
                board.unmake_move(*mv, true);
                self.best_eval = board
                    .white_to_move
                    .then(|| POSITIVE_INFINITY)
                    .unwrap_or(NEGATIVE_INFINITY);
                self.best_move = *mv;
                break;
            }

            let eval = board.evaluate();
            if self.best_eval > eval {
                self.best_eval = eval;
                self.best_move = *mv;
            }

            board.unmake_move(*mv, true);
        }

        // Verify legality of best move
        let best_move = self.best_move;
        if !best_move.is_null() {
            board.make_move(best_move, true);
            if board.calculate_in_check_state() {
                self.best_move = Move::default();
                self.best_eval = 0;
            }
            board.unmake_move(best_move, true);
        }
    }
}

pub fn is_mate_score(score: i32) -> bool {
    if score == i32::MIN {
        false
    } else {
        score.abs() > (IMMEDIATE_MATE_SCORE - MAX_MATE_DEPTH)
    }
}

pub fn num_ply_to_mate_from_score(score: i32) -> i32 {
    IMMEDIATE_MATE_SCORE - score.abs()
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
    pub time_limit: Duration
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
            time_limit: Duration::default(),
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

    pub fn with_limit(ms: i32) -> Self {
        let now = Instant::now();
        Self {
            iteration_timer: now,
            total_timer: now,
            time_limit: Duration::from_millis(ms as u64),
        }
    }
}
