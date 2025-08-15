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

    pub fn start_search(&mut self, board: &mut Board) {
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
        self.search_timer = SearchTimer::new();

        self.run_iterative_deepening_search(board);

        if self.best_move.is_null() {
            let (moves, _) = board.generate_moves(false);
            self.best_move = moves[0];
        }
        self.search_canceled = false;
    }

    pub fn end_search(&mut self) {
        self.search_canceled = true;
    }

    pub fn flush_log(&mut self) -> String {
        let log = self.debug_info.clone();
        self.debug_info = String::new();
        log
    }
}

// Helper IMPL
impl Searcher {
    pub fn bests(&self) -> (i32, Move) {
        (self.best_eval, self.best_move)
    }

    fn run_iterative_deepening_search(&mut self, board: &mut Board) {
        for search_depth in 1..=MAX_PLY {
            self.has_searched_at_least_one_move = false;
            self.debug_info
                .push_str(&format!("\nStarting Iteration: {}", search_depth));
            self.search_timer.restart_iteration_timer();
            self.current_iteration_depth = search_depth;
            let _ = self.search(
                board,
                search_depth,
                0,
                NEGATIVE_INFINITY,
                POSITIVE_INFINITY,
                0,
                Move::default(),
                false,
            );

            if self.search_canceled {
                if self.has_searched_at_least_one_move {
                    self.best_move = self.best_move_this_iteration;
                    self.best_eval = self.best_eval_this_iteration;
                    let best_move_name = self.best_move.to_uci();

                    self.search_diagnostics.move_val = best_move_name.clone();
                    self.search_diagnostics.eval = self.best_eval;
                    self.search_diagnostics.move_is_from_partial_search = true;
                    self.debug_info.push_str(&format!(
                        "\nUsing partial search result: {}, Eval: {}",
                        &best_move_name, self.best_eval
                    ));
                }

                self.debug_info.push_str("\nSearch Aborted");
                break;
            } else {
                self.current_depth = search_depth;
                self.best_move = self.best_move_this_iteration;
                self.best_eval = self.best_eval_this_iteration;
                let best_move_name = self.best_move.to_uci();

                self.debug_info.push_str(&format!(
                    "\nIteration result: {}, Eval: {}",
                    &best_move_name, self.best_eval
                ));
                if is_mate_score(self.best_eval) {
                    self.debug_info.push_str(&format!(
                        ", Mate in: {}",
                        num_ply_to_mate_from_score(self.best_eval_this_iteration)
                    ));
                }

                self.best_eval_this_iteration = i32::MIN;
                self.best_move_this_iteration = Move::null();

                self.search_diagnostics.num_completed_iterations = search_depth;
                self.search_diagnostics.move_val = best_move_name;
                self.search_diagnostics.eval = self.best_eval;

                if is_mate_score(self.best_eval)
                    && num_ply_to_mate_from_score(self.best_eval) <= search_depth
                {
                    self.debug_info
                        .push_str("\nExiting search due to mate found within search depth");
                    break;
                }
            }
        }
    }

    fn search(
        &mut self,
        board: &mut Board,
        ply_remaining: i32,
        ply_from_root: i32,
        alpha: i32,
        beta: i32,
        num_extensions: i32,
        previous_move: Move,
        previous_move_was_capture: bool,
    ) -> i32 {
        let mut alpha = alpha;
        let mut beta = beta;

        if self.search_canceled {
            return 0;
        } else if self.search_timer.elapsed_total() >= Duration::from_secs(5) {
            self.search_canceled = true;
            return 0;
        } else if ply_from_root >= MAX_PLY {
            return board.evaluate();
        }

        if ply_from_root > 0 {
            // Detect draw by three-fold repetition.
            if board.state.halfmove_clock >= 100
                || self.repetition_table.contains(board.state.zobrist.key)
            {
                // Returns a draw score even if this position has only appeared once for sake of simplicity)
                return 0;
            }

            // Skip this position if a mating sequence has already been found earlier in the search
            alpha = alpha.max(-IMMEDIATE_MATE_SCORE + ply_from_root);
            beta = beta.min(IMMEDIATE_MATE_SCORE - ply_from_root);
            if alpha >= beta {
                return alpha;
            }
        }

        // Try looking up the current position in the transposition table.
        let tt_val = self.transposition_table.lookup_evaluation(
            board,
            ply_remaining,
            ply_from_root,
            alpha,
            beta,
        );
        if tt_val != lu_tables::LOOKUP_FAILED {
            match (
                self.transposition_table.try_get_stored_move(board),
                self.transposition_table.entries[self.transposition_table.index(board)],
            ) {
                (Some(mv), Some(eval)) if ply_from_root == 0 => {
                    self.best_move_this_iteration = mv;
                    self.best_eval_this_iteration = eval.value
                }
                _ => {}
            }
            return tt_val;
        }

        if ply_remaining <= 0 {
            return self.quiescence_search(board, alpha, beta, self.current_depth);
        }

        let (mut moves, mg) = board.generate_moves(false);
        let previous_best_move =
            (ply_from_root == 0)
                .then(|| self.best_move)
                .unwrap_or_else(|| {
                    self.transposition_table
                        .try_get_stored_move(board)
                        .unwrap_or(Move::null())
                });

        self.move_orderer.order_moves(
            board,
            previous_best_move,
            &mut moves,
            mg.opponent_attack_map,
            mg.opponent_pawn_attack_map,
            false,
            ply_from_root,
        );

        // Detect checkmate and stalemate when no legal moves are available
        if moves.len() == 0 {
            if mg.in_check {
                return -(IMMEDIATE_MATE_SCORE - ply_from_root);
            } else {
                return 0;
            }
        }

        if ply_from_root > 0 && moves.len() > 0 {
            let was_pawn_move =
                piece::Piece::from(board.squares[previous_move.target_square() as usize])
                    .get_type()
                    == piece::PAWN;
            self.repetition_table.push(
                board.state.zobrist.key,
                previous_move_was_capture || was_pawn_move,
            );
        }

        let mut evaluation_bound = lu_tables::UPPER_BOUND;
        let mut best_move_in_this_position = Move::null();

        for i in 0..moves.len() {
            let mv = moves[i];
            if board.all_piece_lists.iter().any(|list| list.count() == 1) {
                break;
            }

            let captured_piece_type =
                piece::Piece::from(board.squares[mv.target_square() as usize]).get_type();
            let is_capture = captured_piece_type != piece::NONE;
            board.make_move(mv, true);

            // Extend the depth of the search in certain interesting cases
            let mut extension = 0;
            if num_extensions < MAX_EXTENSIONS {
                let moved_piece_type =
                    piece::Piece::from(board.squares[mv.target_square() as usize]).get_type();
                let target_rank = Coord::rank_of_square(mv.target_square());

                if board.is_in_check() {
                    extension = 1;
                } else if moved_piece_type == piece::PAWN && (target_rank == 1 || target_rank == 6)
                {
                    extension = 1;
                }
            }

            let mut needs_full_search = true;
            let mut eval = 0;

            // Reduce the depth of the search for moves later in the move list as these are less likely to be good
            if extension == 0 && ply_remaining >= 3 && i >= 3 && !is_capture {
                let reduced_ply = (ply_remaining - 1 - REDUCE_DEPTH).max(0);
                eval = -self.search(
                    board,
                    reduced_ply,
                    ply_from_root + 1,
                    -alpha - 1,
                    -alpha,
                    num_extensions,
                    mv,
                    is_capture,
                );
                needs_full_search = eval > alpha;
            }

            if needs_full_search {
                let next_ply = (ply_remaining - 1 + extension).max(0);
                eval = -self.search(
                    board,
                    next_ply,
                    ply_from_root + 1,
                    -beta,
                    -alpha,
                    num_extensions + extension,
                    mv,
                    is_capture,
                );
            }
            board.unmake_move(mv, true);

            if self.search_canceled {
                return 0;
            }

            // Move was *too* good, opponent will choose a different move earlier on to avoid this position.
            if eval >= beta {
                self.transposition_table.store_evaluation(
                    mv,
                    board,
                    ply_remaining,
                    ply_from_root,
                    beta,
                    lu_tables::LOWER_BOUND,
                );

                // Update killer moves and history heuristic
                if !is_capture {
                    if ply_from_root < move_ordering::MAX_KILLER_MOVE_PLY as i32 {
                        self.move_orderer.killer_moves[ply_from_root as usize].add(mv);
                    }
                    let history_score = ply_remaining * ply_remaining;
                    self.move_orderer.history[board.move_color() as usize]
                        [mv.start_square() as usize][mv.target_square() as usize] += history_score;
                }

                if ply_from_root > 0 {
                    self.repetition_table.try_pop();
                }

                self.search_diagnostics.num_cutoffs += 1;
                return beta;
            }

            // Found a new best move in this position
            if eval > alpha {
                evaluation_bound = lu_tables::EXACT;
                best_move_in_this_position = mv;

                alpha = eval;
                if ply_from_root == 0 {
                    self.best_move_this_iteration = mv;
                    self.best_eval_this_iteration = eval;
                    self.has_searched_at_least_one_move = true;
                }
            }
        }

        if ply_from_root > 0 {
            self.repetition_table.try_pop();
        }

        self.transposition_table.store_evaluation(
            best_move_in_this_position,
            board,
            ply_remaining,
            ply_from_root,
            alpha,
            evaluation_bound,
        );

        return alpha;
    }

    fn quiescence_search(&mut self, board: &mut Board, alpha: i32, beta: i32, depth: i32) -> i32 {
        if self.search_canceled {
            return 0;
        } else if depth > MAX_PLY {
            return board.evaluate();
        }

        let mut alpha = alpha;

        // A player isn't forced to make a capture (typically), so see what the evaluation is without capturing anything.
        let mut eval = board.evaluate();
        self.search_diagnostics.num_positions_evaluated += 1;

        if eval >= beta {
            self.search_diagnostics.num_cutoffs += 1;
            return beta;
        } else if eval > alpha {
            alpha = eval;
        }

        // Search capture moves until a 'quiet' position is reached.
        let (mut moves, mg) = board.generate_moves(true);
        self.move_orderer.order_moves(
            board,
            Move::null(),
            &mut moves,
            mg.opponent_attack_map,
            mg.opponent_pawn_attack_map,
            true,
            0,
        );

        for i in 0..moves.len() {
            if board.all_piece_lists.iter().any(|list| list.count() == 1) {
                break;
            }

            board.make_move(moves[i], true);
            eval = -self.quiescence_search(board, alpha, beta, depth + 1);
            board.unmake_move(moves[i], true);

            if eval >= beta {
                self.search_diagnostics.num_cutoffs += 1;
                return beta;
            } else if eval > alpha {
                alpha = eval;
            }
        }

        return alpha;
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
