use owo_colors::OwoColorize;

pub fn display_help() {
    // cli info
    println!(
        "{}",
        format!(
            "{}: A CLI tool to run and manage chess engine matchups",
            env!("CARGO_PKG_NAME").bright_green().bold()
        )
        .bright_white()
        .bold()
    );
    // version info
    println!(
        "{}",
        format!("(version {})\n", env!("CARGO_PKG_VERSION")).green()
    );

    // Display Usage
    print!("{}", "Usage: ".bright_yellow().bold());
    print!("{}", "cactus-cli ".bright_green().bold());
    println!("{}", "[COMMANDS] \n".green());

    // Display commands
    println!("{}", "Commands: ".bright_yellow().bold());

    // The initialization command
    print!("{}", "    init".bright_green().bold());
    println!("              Initialize a new cactus.toml template");

    // The run command
    print!("{}", "    run".bright_green().bold());
    println!("               Run the matchup defined in cactus.toml");

    // The status command
    print!("{}", "    status".bright_green().bold());
    println!("            Show current matchup status");

    // The validate command
    print!("{}", "    validate".bright_green().bold());
    println!("          Validate cactus.toml to prevent misconfigured runs");

    // Help command
    print!("{}", "    help".bright_green().bold());
    println!("              Show this help message and version info");
}
