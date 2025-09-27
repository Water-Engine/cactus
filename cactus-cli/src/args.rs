mod help;
mod init;
mod run;
mod status;
mod validate;

use crate::args::help::display_help;
use crate::args::init::initialize_cactus_resources;
use crate::args::run::run_matchup_from_cactus_toml;
use crate::args::status::show_current_matchup_status;
use crate::args::validate::validate_cactus_toml;
use owo_colors::OwoColorize;
use std::process::exit;

pub fn argument_parser() {
    let args: Vec<String> = std::env::args().collect();

    // Handle the case of no command provided
    if args.len() < 2 {
        eprint!("No command provided.\nRun ");
        eprint!("{}", "cactus-cli help".bright_green());
        eprintln!(" to list all commands and usage info.");
        exit(1);
    }

    let arg = &args[1];

    match arg.as_str() {
        "help" => display_help(),
        "init" => initialize_cactus_resources(),
        "run" => run_matchup_from_cactus_toml(),
        "status" => show_current_matchup_status(),
        "validate" => validate_cactus_toml(),
        _ => {
            print!("Invalid command provided: {arg}.\nRun ");
            print!("{}", "cactus-cli help".bright_green());
            println!(" to list all commands and usage info.");
        }
    }
}
