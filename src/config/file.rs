//! Configuration file loading and saving

use super::Config;
use crate::error::{DevSweepError, Result};
use std::path::{Path, PathBuf};

/// Get the default config file path
pub fn default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        DevSweepError::Config("Cannot determine config directory".into())
    })?;

    Ok(config_dir.join("devsweep").join("config.toml"))
}

/// Load configuration from file
pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        DevSweepError::ConfigParse {
            path: path.to_path_buf(),
            reason: e.to_string(),
        }
    })?;

    let config: Config = toml::from_str(&content).map_err(|e| {
        DevSweepError::ConfigParse {
            path: path.to_path_buf(),
            reason: e.to_string(),
        }
    })?;

    Ok(config)
}

/// Load configuration from default location
pub fn load_default_config() -> Result<Config> {
    let path = default_config_path()?;
    load_config(&path)
}

/// Save configuration to file
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;

    Ok(())
}

/// Save configuration to default location
pub fn save_default_config(config: &Config) -> Result<()> {
    let path = default_config_path()?;
    save_config(config, &path)
}

/// Generate a sample configuration file
pub fn generate_sample_config() -> String {
    r#"# DevSweep Configuration
# Location: ~/.config/devsweep/config.toml

[general]
# Default directories to scan (leave empty to require explicit path)
default_paths = [
    "~/projects",
    "~/code",
]

# Paths to always exclude
exclude_paths = [
    "~/projects/important-project",
]

# Log level: error, warn, info, debug, trace
log_level = "info"

# Enable verbose output
verbose = false

[scan]
# Maximum depth to scan (null = unlimited)
# max_depth = 10

# Skip hidden directories (starting with .)
skip_hidden = true

# Respect .gitignore files
respect_gitignore = true

# Minimum artifact size to report in bytes (null = no minimum)
# min_size = 1000000  # 1 MB

# Custom ignore patterns (glob syntax)
ignore_patterns = []

# Number of parallel threads (null = auto based on CPU)
# parallelism = 4

# Check git status for each project
check_git_status = true

[clean]
# Delete method: trash, permanent, dry-run
delete_method = "trash"

# Protection level: none, warn, block, paranoid
protection_level = "warn"

# Continue on errors
continue_on_error = true

# Auto-confirm without prompts (use with caution!)
auto_confirm = false

# Dry run by default
dry_run = false

[ui]
# Color theme: dark, light, auto
theme = "auto"

# Show file counts for artifacts
show_file_counts = true

# Show last modified dates
show_dates = true

# Sort results by: size, name, date, kind
sort_by = "size"

# Reverse sort order
sort_reverse = false

# Use icons/emojis
use_icons = true

[plugins]
# Enabled plugins (empty = all)
enabled = []

# Disabled plugins
disabled = []
"#.to_string()
}

/// Initialize config directory with sample config
pub fn init_config() -> Result<PathBuf> {
    let path = default_config_path()?;

    if path.exists() {
        return Err(DevSweepError::Config(format!(
            "Config file already exists at {}",
            path.display()
        )));
    }

    // Create parent directory
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write sample config
    let sample = generate_sample_config();
    std::fs::write(&path, sample)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_config() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent.toml");

        let config = load_config(&path).unwrap();
        // Should return default config
        assert_eq!(config.clean.delete_method, crate::trash::DeleteMethod::Trash);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.general.verbose = true;
        config.scan.max_depth = Some(5);

        save_config(&config, &path).unwrap();

        let loaded = load_config(&path).unwrap();
        assert!(loaded.general.verbose);
        assert_eq!(loaded.scan.max_depth, Some(5));
    }

    #[test]
    fn test_sample_config_is_valid() {
        let sample = generate_sample_config();
        let result: std::result::Result<Config, _> = toml::from_str(&sample);
        assert!(result.is_ok(), "Sample config should be valid TOML");
    }
}
