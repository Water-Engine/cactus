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
    pub on_move_chosen: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,

    pub thinking: bool,
    pub latest_move_is_book_move: bool,

    pub searcher: Arc<Mutex<Searcher>>,
    board: Arc<Mutex<Board>>,
    book: Arc<Mutex<OpeningBook>>,
    search_request: Arc<(Mutex<SearchRequest>, Condvar)>,

    is_quitting: Arc<Mutex<bool>>,
}

impl Brain {
    pub fn new() -> Result<Self, String> {
        let mut board = Board::new();
        board.load_start_pos()?;

        let brain = Self {
            on_move_chosen: Arc::new(Mutex::new(None::<Box<dyn Fn(String) + Send + Sync>>)),

            thinking: false,
            latest_move_is_book_move: false,

            searcher: Arc::new(Mutex::new(Searcher::new())),
            board: Arc::new(Mutex::new(board)),
            book: Arc::new(Mutex::new(OpeningBook::new(opening_book::BOOK))),
            search_request: Arc::new((Mutex::new(SearchRequest::default()), Condvar::new())),

            is_quitting: Arc::new(Mutex::new(bool::default())),
        };

        spawn_search_thread(
            Arc::clone(&brain.board),
            Arc::clone(&brain.searcher),
            Arc::clone(&brain.search_request),
            Arc::clone(&brain.is_quitting),
            Arc::clone(&brain.on_move_chosen),
        );
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
        let chosen_time_ms = min_think_time.max(think_time_ms).ceil() as i32;
        Ok(chosen_time_ms)
    }

    pub fn think_timed(&mut self, time_ms: i32) -> Result<(), String> {
        self.latest_move_is_book_move = false;
        self.thinking = true;

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

    pub fn set_on_move_chosen<S: super::driver::SenderLike + Send + Sync + 'static>(
        &self,
        sender: S,
    ) {
        let on_move_chosen = Arc::clone(&self.on_move_chosen);

        let mut callback_lock = on_move_chosen.lock().unwrap();
        *callback_lock = Some(Box::new(move |mv: String| {
            sender.send(format!("bestmove {}", mv));
        }));
    }
}

// Helper IMPL
impl Brain {
    fn start_search(&mut self, time_ms: i32) -> Result<(), String> {
        let (lock, cvar) = &*self.search_request;
        let mut req = lock.lock().map_err(|_| "Lock failed")?;
        req.time_ms = time_ms;
        req.ready = true;
        cvar.notify_one();
        Ok(())
    }

    fn end_search(&self) -> Result<(), String> {
        if let Ok(mut searcher) = self.searcher.lock() {
            searcher.end_search();
        }
        Ok(())
    }

    fn on_search_complete(&mut self, mv: &Move) -> Result<(), String> {
        self.thinking = false;
        let move_string = mv.to_uci().replace("=", "");
        if let Some(action_func) = &*self.on_move_chosen.lock().unwrap() {
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

fn spawn_search_thread(
    board: Arc<Mutex<Board>>,
    searcher: Arc<Mutex<Searcher>>,
    request: Arc<(Mutex<SearchRequest>, Condvar)>,
    is_quitting: Arc<Mutex<bool>>,
    on_move_chosen: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
) {
    std::thread::spawn(move || {
        loop {
            if *is_quitting.lock().unwrap() {
                break;
            }

            let (lock, cvar) = &*request;
            let mut req = lock.lock().unwrap();
            while !req.ready && !*is_quitting.lock().unwrap() {
                req = cvar.wait(req).unwrap();
            }

            if *is_quitting.lock().unwrap() {
                break;
            }

            let time_ms = req.time_ms;
            req.ready = false;

            if let Ok(mut s) = searcher.lock() {
                if let Ok(mut b) = board.lock() {
                    s.start_search(&mut b, time_ms);

                    if let Some(callback) = &*on_move_chosen.lock().unwrap() {
                        if let Some((_, best_move)) = s.bests() {
                            callback(best_move.to_uci().replace("=", ""));
                        }
                    }
                }
            }
        }
    });
}

struct SearchRequest {
    time_ms: i32,
    ready: bool,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            time_ms: 0,
            ready: false,
        }
    }
}
