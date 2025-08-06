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

    if !args.is_empty() {
        match (args.get(0), args.get(1)) {
            (Some(first), None) => cactus = first == "engine",
            (Some(first), Some(second)) => stockfish = first == "engine" && second == "stockfish",
            _ => {}
        }
    }

    if stockfish {
        println!("Starting with external Stockfish engine...");
    } else if cactus {
        println!("Starting with internal Cactus engine...");
    } else {
        gui::launch::launch();
    }
}
