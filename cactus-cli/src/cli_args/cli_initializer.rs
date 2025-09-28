use crate::cli_utils::cactus_toml_exists;

use owo_colors::OwoColorize;
use std::{env, fs, path::Path};

pub fn initialize_cactus_resources() {
    // Capture current directory
    let dir = env::current_dir().expect("Failed to get current directory");

    // Check if cactus.toml already exists
    if cactus_toml_exists() {
        eprintln!(
            "{}",
            format!(
                "-> cactus.toml already exists at: {}",
                dir.display().bright_yellow()
            )
            .bright_red()
        );
        eprintln!(
            "{}",
            "-> Reinitialization is not supported, please edit the existing file instead."
                .bright_red()
        );
        return;
    }

    // Use pwd as "path" to create cactus.toml
    let path = Path::new("cactus.toml");

    // Default cactus.toml template
    let template = r#"
# This is a starter template for running cactus.
# Refer to the documentation for more info on the commands.

[event]
engines = [
    "stockfish",
    "Ethereal",
    "Leela Chess"
]
tournament = "swiss"
time_control = "5+0"
"#;

    // Create cactus.toml
    fs::write(path, template).expect("Could not create cactus.toml");

    // Pretty success output
    println!(
        "{}",
        format!(
            "-> Successfully created cactus.toml at: {}",
            dir.display().bright_yellow()
        )
        .bright_green()
    );
}
