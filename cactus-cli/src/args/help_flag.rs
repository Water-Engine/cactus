use owo_colors::OwoColorize;

pub fn display_global_help() {
    // displaying cli info at top
    println!(
        "{}",
        format!(
            "{}: A CLI tool to run and manage chess engine tournaments\n",
            env!("CARGO_PKG_NAME").bright_green().bold()
        )
        .bright_white()
        .bold()
    );

    // Display Usage
    print!("{}", "Usage: ".bright_yellow().bold());
    print!("{}", "cactus-cli ".bright_green().bold());
    println!("{}", "[COMMANDS] [FLAGS]\n".green());

    // Display commands
    println!("{}", "Commands: ".bright_yellow().bold());
    // The initialization command
    print!("{}", "    init".bright_green().bold());
    println!("              Initialize a new cactus.toml template at pwd");
    // The run command
    print!("{}", "    run".bright_green().bold());
    println!("               Run the matchup defined in cactus.toml");

    // Display Flags
    println!("{}", "\nFlags: ".bright_yellow().bold());
    // Help flag
    print!("{}", "    --help".bright_green().bold());
    println!("            Show context sensitive help info");
    // Version flag
    print!("{}", "    --info".bright_green().bold());
    println!("            Show program information");
}

pub fn display_run_command_help() {
    // displaying cli info at top
    println!(
        "{}",
        format!(
            "{}: A CLI tool to run and manage chess engine tournaments\n",
            env!("CARGO_PKG_NAME").bright_green().bold()
        )
        .bright_white()
        .bold()
    );

    // Display Usage
    print!("{}", "Usage: ".bright_yellow().bold());
    print!("{}", "cactus-cli run ".bright_green().bold());
    println!("{}", "[FLAGS]\n".green());

    // Display Flags
    println!("{}", "Flags: ".bright_yellow().bold());
    // working directory flag
    print!("{}", "    --cwd".bright_green().bold());
    println!("             Set a working directory, exports will be stored here.");
    // get-config flag
    print!("{}", "    --config".bright_green().bold());
    println!("          Import config from specified cactus.toml");
    // Validate flag
    print!("{}", "    --dry".bright_green().bold());
    println!("             Dry run to prevent misconfigured runs");
    // profiles flag
    print!("{}", "    --profile".bright_green().bold());
    println!("         Run a profile specified in cactus.toml");
}
