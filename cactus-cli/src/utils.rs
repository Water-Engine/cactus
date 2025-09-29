use std::path::Path;

/// Check if `cactus.toml` exists in the current directory.
pub fn cactus_toml_exists() -> bool {
    Path::new("cactus.toml").exists()
}
