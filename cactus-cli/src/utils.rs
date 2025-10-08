use std::env;
use std::path::PathBuf;

// (creates if not exists or) finds the config directory across platforms
// for linux, the config is stored at "~/.config/cactus/gui"
pub fn get_config_dir() -> PathBuf {
    // Fetch the $HOME variable for the system
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Could not find home directory");

    // Define the cactus config directory
    let config_dir = PathBuf::from(home).join(".config").join("cactus");
    // NOTE: Still debating over how resources should be split between gui and cli
    // hence we comment this for now
    // .join("cli");

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    config_dir
}

pub fn cactus_toml_exists(working_dir: &PathBuf) -> bool {
    working_dir.join("cactus.toml").exists()
}

pub fn make_path_abs(path: &str) -> PathBuf {
    PathBuf::from(path).canonicalize().unwrap_or_else(|_| {
        // Fallback if path doesn't exist yet
        let p = PathBuf::from(path);
        if p.is_absolute() {
            p
        } else {
            env::current_dir().unwrap().join(p)
        }
    })
}
