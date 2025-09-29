mod help_flag;
mod info_flag;
mod init_command;
mod run_command;

use help_flag::display_global_help;
use info_flag::display_program_info;
use init_command::initialize_cactus_resources;
use run_command::run_matchup_from_cactus_toml;

use owo_colors::OwoColorize;
use std::process::exit;

pub fn argument_parser() {
    let args: Vec<String> = std::env::args().collect();

    // Handle the case of no command provided
    if args.len() < 2 {
        eprint!("No command provided.\nRun ");
        eprint!("{}", "cactus-cli --help".bright_green());
        eprintln!(" to list all commands and usage info.");
        exit(1);
    }

    let arg = &args[1];

    match arg.as_str() {
        // Call resp functions for commands
        "init" => initialize_cactus_resources(),
        "run" => run_matchup_from_cactus_toml(args),
        // Handling global flags
        "--help" => display_global_help(),
        "--info" => display_program_info(),
        _ => {
            print!("Invalid command provided: {arg}.\nRun ");
            print!("{}", "cactus-cli --help".bright_green());
            println!(" to list all commands and usage info.");
        }
    }
}
