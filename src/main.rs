use crate::engine::external::ExternalEngine;

mod core;
mod engine;
mod gui;
mod moves;

fn main() {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .map(|s| s.trim().to_lowercase())
        .collect();

    let mut stockfish = false;
    let mut cactus = false;
    let mut engine_black = Some(true);

    if !args.is_empty() {
        match (args.get(0), args.get(1)) {
            (Some(first), None) => cactus = first == "engine",
            (Some(first), Some(second)) => stockfish = first == "engine" && second == "stockfish",
            _ => {}
        }
    }

    if let Some(engine_color) = args.last() {
        if engine_color == &"black".to_string() {
            engine_black = Some(true);
        }
    }

    if stockfish {
        println!("Starting with external Stockfish engine...");
        gui::launch::launch(
            ExternalEngine::spawn_threaded("stockfish/stockfish-windows-x86-64.exe").ok(),
            engine_black,
        );
    } else if cactus {
        println!("Starting with internal Cactus engine...");
    } else {
        gui::launch::launch(None, None);
    }
}
