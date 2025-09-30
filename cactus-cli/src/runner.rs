use crate::args::run_command::RunFlags;

pub fn run_matchup_from_cactus_toml(args: Vec<String>) {
    if let Some(opts) = RunFlags::parse_run_command_args(&args[2..]) {
        println!("{opts:#?}");
    }
}
