use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

use crate::args::help_flag::display_run_command_help;
use crate::utils::make_path_abs;

#[derive(Debug)]
pub struct RunFlags {
    pub cwd: PathBuf,
    pub dry_run: bool,
    pub event: String,
}

impl RunFlags {
    pub fn parse_run_command_args(args: &[String]) -> Option<Self> {
        let base = env::current_dir().ok()?;
        let mut iter = args.iter();

        let mut cwd = None;
        let mut dry_run = false;
        let mut event = None;

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--cwd" => match iter.next() {
                    Some(val) => cwd = Some(val.as_str()),
                    None => {
                        eprintln!("{}", "-> Error: --cwd requires a value".red());
                        return None;
                    }
                },
                "--dry" => dry_run = true,
                "--event" => match iter.next() {
                    Some(val) => event = Some(val.as_str()),
                    None => {
                        eprintln!("{}", "-> Error: --event requires a value".red());
                        return None;
                    }
                },
                "--help" => {
                    display_run_command_help();
                    return None;
                }
                other if other.starts_with("--") => {
                    eprintln!("{}", format!("-> Error: Unknown flag: {}", other).red());
                    return None;
                }
                other => {
                    eprintln!(
                        "{}",
                        format!("-> Error: Unexpected argument: {}", other).red()
                    );
                    return None;
                }
            }
        }

        Some(Self {
            cwd: cwd.map(make_path_abs).unwrap_or(base.clone()),
            dry_run,
            event: event
                .map(String::from)
                .unwrap_or_else(|| "default".to_string()),
        })
    }
}
