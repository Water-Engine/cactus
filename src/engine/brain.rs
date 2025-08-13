use std::sync::{Arc, Condvar, Mutex};

use crate::engine::{
    game::board::Board, search::searcher::Searcher, utils::opening_book::{self, OpeningBook},
};

const USE_OPENING_BOOK: bool = true;
const MAX_BOOK_PLY: i32 = 16;

const USE_MAX_THINKING_TIME: bool = false;
const MAX_THINK_TIME_MS: i32 = 2500;

pub struct Brain {
    pub thinking: bool,
    pub latest_move_is_book_move: bool,

    searcher: Arc<Mutex<Searcher>>,
    board: Arc<Mutex<Board>>,
    book: Arc<Mutex<OpeningBook>>,

    search_wait_handle: Arc<(Mutex<bool>, Condvar)>,
    cancel_search_timer: Option<CancellationToken>,

    current_search_id: Arc<i32>,
    is_quitting: Arc<bool>,
}

impl Brain {
    pub fn new() -> Result<Self, String> {
        let mut board = Board::new();
        board.load_start_pos()?;
        Ok(Self {
            thinking: false,
            latest_move_is_book_move: false,

            searcher: Arc::new(Mutex::new(Searcher::new())),
            board: Arc::new(Mutex::new(board)),
            book: Arc::new(Mutex::new(OpeningBook::new(opening_book::BOOK))),

            search_wait_handle: Arc::new((Mutex::new(false), Condvar::new())),
            cancel_search_timer: None,

            current_search_id: Arc::new(i32::default()),
            is_quitting: Arc::new(bool::default()),
        })
    }

    pub fn notify_new_game(&self) -> Result<(), String>{
        let mut s = self.searcher.lock().map_err(|_| "Searcher mutex poisoned")?;
        s.clear_for_new_position();
        Ok(())
    }
}

#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<Mutex<bool>>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
        }
    }
    pub fn cancel(&self) {
        match self.cancelled.lock() {
            Ok(mut cancelled) => *cancelled = true,
            Err(poisoned) => {
                let mut cancelled = poisoned.into_inner();
                *cancelled = true
            }
        }
    }
    pub fn is_cancelled(&self) -> bool {
        match self.cancelled.lock() {
            Ok(mut cancelled) => *cancelled,
            Err(poisoned) => {
                let mut cancelled = poisoned.into_inner();
                *cancelled
            }
        }
    }
}
