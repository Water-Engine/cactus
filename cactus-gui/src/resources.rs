// This file holds the functions responsible for creating the config directory and other resources for the gui.
// I've made sure to keep it modular, and easy to add other resources we may need in future (like sounds etc)

use std::path::PathBuf;

// (creates if not exists or) finds the config directory across platforms
// for linux, the config is stored at "~/.config/cactus/gui"
pub fn get_config_dir() -> PathBuf {
    // Fetch the $HOME variable for the system
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Could not find home directory");

    // Define the cactus config directory
    let config_dir = PathBuf::from(home)
        .join(".config")
        .join("cactus")
        .join("gui");

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    config_dir
}

const DARK_STYLE_DATA: &[u8] = include_bytes!("../assests/styles/style_dark.rgs");
pub fn provide_style_path() -> PathBuf {
    let style_path = get_config_dir()
        .join("assests")
        .join("styles")
        .join("style_dark.rgs");

    // Write style file if it doesn't exist
    if !style_path.exists() {
        std::fs::write(&style_path, DARK_STYLE_DATA).expect("Failed to write style file");
    }

    style_path
}
