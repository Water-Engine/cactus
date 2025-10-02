mod help_flag;
mod info_flag;
pub mod run_command;

use crate::args::{help_flag::display_global_help, info_flag::display_program_info};
use crate::initializer::initialize_cactus_resources;
use crate::runner::run_event_from_cactus_toml;

use owo_colors::OwoColorize;
use std::process::exit;

pub fn argument_parser() {
    let args: Vec<String> = std::env::args().collect();

    // Handle the case of no command provided
    if args.len() < 2 {
        eprint!("{}", "-> Error: No command provided".red());
        eprint!("\n-> Run ");
        eprint!("{}", "cactus-cli --help".bright_green());
        eprintln!(" to list all commands and usage info.");
        exit(1);
    }

    let arg = &args[1];

    match arg.as_str() {
        // Call resp functions for commands
        "init" => initialize_cactus_resources(args),
        "run" => run_event_from_cactus_toml(args),
        // Handling global flags
        "--help" => display_global_help(),
        "--info" => display_program_info(),
        _ => {
            print!(
                "{}",
                format!("-> Error: Invalid command provided: {}", arg).red()
            );
            print!("\n-> Run ");
            print!("{}", "cactus-cli --help".bright_green());
            println!(" to list all commands and usage info.");
        }
    }
}
