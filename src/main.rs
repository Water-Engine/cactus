use std::time::Duration;

// #![allow(unused)]
use crate::coupling::{EngineHandle, external::ExternalEngine};
use crate::engine::brain;
use crate::engine::driver::CactusEngine;
use crate::engine::search::searcher;
use crate::engine::utils::opening_book::{self, OpeningBook};

mod core;
mod coupling;
mod engine;
mod gui;
mod moves;

fn main() {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .map(|s| s.trim().to_lowercase())
        .collect();

    let mut use_stockfish = false;
    let mut use_cactus = false;
    let mut is_engine_black = true;
    let mut run_gui = true;

    match args.as_slice() {
        [engine] if engine == "engine" => {
            run_gui = false;
        }

        [engine, engine_name] if engine == "engine" => {
            use_cactus = engine_name == "cactus";
            use_stockfish = engine_name == "stockfish";
        }

        [engine, engine_name, color] if engine == "engine" => {
            use_cactus = engine_name == "cactus";
            use_stockfish = engine_name == "stockfish";
            is_engine_black = color != "white";
        }

        _ => {}
    }

    if !run_gui {
        println!("Running cactus client...");
        CactusEngine::start();
        return;
    }

    let mut maybe_white_engine: Option<EngineHandle> = None;
    let mut maybe_black_engine: Option<EngineHandle> = None;

    if use_stockfish {
        println!("Starting GUI with external Stockfish engine...");
        let engine = ExternalEngine::spawn_threaded("stockfish/stockfish-windows-x86-64.exe").ok();
        if is_engine_black {
            maybe_black_engine = engine;
        } else {
            maybe_white_engine = engine;
        }
    } else if use_cactus {
        println!("Starting GUI with internal Cactus engine...");
        let engine = CactusEngine::spawn_threaded();
        if is_engine_black {
            maybe_black_engine = Some(engine);
        } else {
            maybe_white_engine = Some(engine);
        }
    }

    // gui::launch::launch(maybe_white_engine, maybe_black_engine);
    let mut board = crate::engine::game::board::Board::new();
    let _ = board.load_start_pos();
    let mut book = OpeningBook::new(opening_book::BOOK);
    let mv = book.try_get_book_move(&mut board, 0.5);
    dbg!(mv);
}

// Good position to test as it exceeds open: position startpos moves e2e4 e7e5 b1c3 f8c5 g1f3 d7d6