use owo_colors::OwoColorize;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const GIT_HASH: &str = env!("GIT_HASH"); // from build.rs

pub fn display_program_info() {
    print!("{}", NAME.bright_green().bold());
    print!("{}", format!(" v{}", VERSION).bright_yellow());
    println!("{}", format!(" git({})", GIT_HASH).bright_blue());
    println!(
        "Distributed under {} by Cactus developers,",
        LICENSE.yellow()
    );
    println!("for more info: {}", REPOSITORY.green());
}
