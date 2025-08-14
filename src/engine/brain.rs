use std::sync::{Arc, Condvar, Mutex};

use crate::engine::{
    game::{
        board::Board,
        r#move::{self, Move},
    },
    search::searcher::Searcher,
    utils::opening_book::{self, OpeningBook},
};

const USE_OPENING_BOOK: bool = true;
const MAX_BOOK_PLY: i32 = 16;

const USE_MAX_THINKING_TIME: bool = false;
const MAX_THINK_TIME_MS: i32 = 2500;

pub struct Brain {
    pub on_move_chosen: Option<Box<dyn Fn(String) + Send + Sync>>,

    pub thinking: bool,
    pub latest_move_is_book_move: bool,

    searcher: Arc<Mutex<Searcher>>,
    board: Arc<Mutex<Board>>,
    book: Arc<Mutex<OpeningBook>>,

    search_wait_handle: Arc<(Mutex<bool>, Condvar)>,
    cancel_search_timer: Option<CancellationToken>,

    current_search_id: Arc<Mutex<i32>>,
    is_quitting: Arc<Mutex<bool>>,
}

impl Brain {
    pub fn new() -> Result<Self, String> {
        let mut board = Board::new();
        board.load_start_pos()?;

        let brain = Self {
            on_move_chosen: None,

            thinking: false,
            latest_move_is_book_move: false,

            searcher: Arc::new(Mutex::new(Searcher::new())),
            board: Arc::new(Mutex::new(board)),
            book: Arc::new(Mutex::new(OpeningBook::new(opening_book::BOOK))),

            search_wait_handle: Arc::new((Mutex::new(false), Condvar::new())),
            cancel_search_timer: None,

            current_search_id: Arc::new(Mutex::new(i32::default())),
            is_quitting: Arc::new(Mutex::new(bool::default())),
        };
        brain.spawn_search_thread();
        Ok(brain)
    }

    pub fn notify_new_game(&self) -> Result<(), String> {
        let mut searcher = self
            .searcher
            .lock()
            .map_err(|_| "Searcher mutex poisoned")?;
        searcher.clear_for_new_position();
        Ok(())
    }

    pub fn set_position(&self, fen: &str) -> Result<(), String> {
        let mut board = self.board.lock().map_err(|_| "Board mutex poisoned")?;
        board.load_from_fen(fen)?;
        Ok(())
    }

    pub fn make_move(&self, move_string: &str) -> Result<(), String> {
        let mut board = self.board.lock().map_err(|_| "Board mutex poisoned")?;
        let mv = r#move::Move::from_uci(&board, move_string);
        board.make_move(mv, false);
        Ok(())
    }

    pub fn choose_think_time(
        &self,
        time_remaining_white_ms: i32,
        time_remaining_black_ms: i32,
        increment_white_ms: i32,
        increment_black_ms: i32,
    ) -> Result<i32, String> {
        let board = self.board.lock().map_err(|_| "Board mutex poisoned")?;
        let my_time_remaining_ms = board
            .white_to_move
            .then(|| time_remaining_white_ms)
            .unwrap_or(time_remaining_black_ms);
        let my_increment_ms = board
            .white_to_move
            .then(|| increment_white_ms)
            .unwrap_or(increment_black_ms);
        let mut think_time_ms = my_time_remaining_ms as f32 / 40.0;

        if USE_MAX_THINKING_TIME {
            think_time_ms = (MAX_THINK_TIME_MS as f32).min(think_time_ms);
        }

        if my_time_remaining_ms > my_increment_ms * 2 {
            think_time_ms += my_increment_ms as f32 * 0.8;
        }

        let min_think_time = 50.0_f32.min(my_time_remaining_ms as f32 * 0.25);
        Ok(min_think_time.max(think_time_ms).ceil() as i32)
    }

    pub fn think_timed(&mut self, time_ms: i32) -> Result<(), String> {
        self.latest_move_is_book_move = false;
        self.thinking = true;
        if let Some(t) = &self.cancel_search_timer {
            t.cancel();
        }

        if let Some(book_mv) = self.try_get_opening_move()? {
            self.latest_move_is_book_move = true;
            self.on_search_complete(&book_mv)
        } else {
            self.start_search(time_ms)
        }
    }

    pub fn quit(&self) -> Result<(), String> {
        if let Ok(mut is_quitting) = self.is_quitting.lock() {
            *is_quitting = true;
        }
        self.end_search()
    }

    pub fn stop_thinking(&self) -> Result<(), String> {
        self.end_search()
    }

    pub fn display_board(&self) -> Result<String, String> {
        let mut board = self.board.lock().map_err(|_| "Board mutex poisoned")?;
        Ok(board.to_string())
    }
}

// Helper IMPL
impl Brain {
    pub fn spawn_search_thread(&self) {
        let handle = Arc::clone(&self.search_wait_handle);
        let is_quitting = Arc::clone(&self.is_quitting);
        let searcher = Arc::clone(&self.searcher);

        std::thread::spawn(move || {
            loop {
                if *is_quitting.lock().unwrap() {
                    break;
                }

                let (lock, cvar) = &*handle;
                let mut ready = lock.lock().unwrap();
                while !*ready && !*is_quitting.lock().unwrap() {
                    ready = cvar.wait(ready).unwrap();
                }
                *ready = false;

                // Run the search
                if let Ok(mut searcher) = searcher.lock() {
                    searcher.start_search()
                }
            }
        });
    }

    fn start_search(&mut self, time_ms: i32) -> Result<(), String> {
        {
            let mut id = self
                .current_search_id
                .lock()
                .map_err(|_| "Current search id mutex poisoned")?;
            *id += 1;
        }

        {
            let (lock, cvar) = &*self.search_wait_handle;
            if let Ok(mut ready) = lock.lock() {
                *ready = true;
                cvar.notify_one();
            } else {
                return Err("Search wait handle mutex poisoned".into());
            }
        }

        let token = CancellationToken::new();
        self.cancel_search_timer = Some(token.clone());

        let this_search_id = *self
            .current_search_id
            .lock()
            .map_err(|_| "Current search id mutex poisoned")?;
        let searcher = Arc::clone(&self.searcher);
        let is_quitting = Arc::clone(&self.is_quitting);

        std::thread::spawn(move || {
            use std::time::Duration;
            std::thread::sleep(Duration::from_millis(time_ms as u64));

            if token.is_cancelled() {
                return;
            }
            if *is_quitting.lock().unwrap() {
                return;
            }

            if let Ok(mut s) = searcher.lock() {
                s.end_search();
            }
        });

        Ok(())
    }

    /// Ends the current search
    fn end_search(&self) -> Result<(), String> {
        if let Some(cancellation_token) = &self.cancel_search_timer {
            cancellation_token.cancel();
        }

        if let Ok(mut searcher) = self.searcher.lock() {
            searcher.end_search();
        }
        Ok(())
    }

    fn on_search_complete(&mut self, mv: &Move) -> Result<(), String> {
        self.thinking = false;
        let move_string = mv.to_uci().replace("=", "");
        if let Some(action_func) = &self.on_move_chosen {
            action_func(move_string);
        }
        Ok(())
    }

    fn try_get_opening_move(&self) -> Result<Option<Move>, String> {
        let mut board = self.board.lock().map_err(|_| "Board mutex poisoned")?;
        let mut book = self.book.lock().map_err(|_| "Book mutex poisoned")?;
        if USE_OPENING_BOOK && board.ply_count <= MAX_BOOK_PLY {
            if let Some(move_string) = book.try_get_book_move(&mut board, 0.5) {
                return Ok(Some(r#move::Move::from_uci(&board, &move_string)));
            }
        }
        Ok(None)
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
