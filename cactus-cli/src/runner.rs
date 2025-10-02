mod dryrun;

use crate::{args::run_command::RunFlags, runner::dryrun::dry_run_event_from_config};
use owo_colors::OwoColorize;

pub fn run_event_from_cactus_toml(args: Vec<String>) {
    if let Some(opts) = RunFlags::parse_run_command_args(&args[2..]) {
        if !opts.cwd.join("cactus.toml").exists() {
            println!(
                "{}",
                "-> Error: Current working directory is not initialized.".red()
            );
            println!(
                "{}",
                "-> Run `cactus-cli init <cwd>` to initialize it first.".green()
            );
            println!(
                "{}",
                "-> Or specify a different cwd using `cactus-cli run --cwd <cwd>`".green()
            );
        }

        if opts.dry_run {
            dry_run_event_from_config(&opts); // Pass a reference to the whole struct
            return;
        }
    }
}
