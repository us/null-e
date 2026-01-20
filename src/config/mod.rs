//! Configuration management
//!
//! Handles loading, saving, and merging configuration from multiple sources:
//! - Default values
//! - Config file (~/.config/devsweep/config.toml)
//! - Environment variables
//! - Command line arguments

mod file;

pub use file::*;

use crate::git::ProtectionLevel;
use crate::trash::DeleteMethod;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// Scan settings
    pub scan: ScanSettings,
    /// Clean settings
    pub clean: CleanSettings,
    /// UI settings
    pub ui: UiSettings,
    /// Plugin settings
    pub plugins: PluginSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            scan: ScanSettings::default(),
            clean: CleanSettings::default(),
            ui: UiSettings::default(),
            plugins: PluginSettings::default(),
        }
    }
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default directories to scan
    pub default_paths: Vec<PathBuf>,
    /// Paths to always exclude
    pub exclude_paths: Vec<PathBuf>,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        let mut default_paths = Vec::new();

        // Add common project directories
        if let Some(home) = dirs::home_dir() {
            default_paths.push(home.join("projects"));
            default_paths.push(home.join("Projects"));
            default_paths.push(home.join("code"));
            default_paths.push(home.join("Code"));
            default_paths.push(home.join("dev"));
            default_paths.push(home.join("Developer"));
            default_paths.push(home.join("src"));
            default_paths.push(home.join("workspace"));
        }

        Self {
            default_paths,
            exclude_paths: vec![],
            log_level: "info".into(),
            verbose: false,
        }
    }
}

/// Scan settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanSettings {
    /// Maximum depth to scan
    pub max_depth: Option<usize>,
    /// Skip hidden directories
    pub skip_hidden: bool,
    /// Respect .gitignore files
    pub respect_gitignore: bool,
    /// Minimum artifact size to report (bytes)
    pub min_size: Option<u64>,
    /// Custom ignore patterns
    pub ignore_patterns: Vec<String>,
    /// Number of parallel threads (None = auto)
    pub parallelism: Option<usize>,
    /// Check git status for each project
    pub check_git_status: bool,
}

impl Default for ScanSettings {
    fn default() -> Self {
        Self {
            max_depth: None,
            skip_hidden: true,
            respect_gitignore: true,
            min_size: None, // Could set to 1MB: Some(1_000_000)
            ignore_patterns: vec![],
            parallelism: None,
            check_git_status: true,
        }
    }
}

/// Clean settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CleanSettings {
    /// Default delete method
    #[serde(with = "delete_method_serde")]
    pub delete_method: DeleteMethod,
    /// Protection level for git repos
    #[serde(with = "protection_level_serde")]
    pub protection_level: ProtectionLevel,
    /// Continue on errors
    pub continue_on_error: bool,
    /// Auto-confirm (no prompts)
    pub auto_confirm: bool,
    /// Dry run by default
    pub dry_run: bool,
}

impl Default for CleanSettings {
    fn default() -> Self {
        Self {
            delete_method: DeleteMethod::Trash,
            protection_level: ProtectionLevel::Warn,
            continue_on_error: true,
            auto_confirm: false,
            dry_run: false,
        }
    }
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// Color theme (dark, light, auto)
    pub theme: String,
    /// Show file counts
    pub show_file_counts: bool,
    /// Show last modified dates
    pub show_dates: bool,
    /// Sort by (size, name, date, kind)
    pub sort_by: String,
    /// Reverse sort order
    pub sort_reverse: bool,
    /// Use icons/emojis
    pub use_icons: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            show_file_counts: true,
            show_dates: true,
            sort_by: "size".into(),
            sort_reverse: false,
            use_icons: true,
        }
    }
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginSettings {
    /// Enabled plugins (empty = all)
    pub enabled: Vec<String>,
    /// Disabled plugins
    pub disabled: Vec<String>,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: vec![],
            disabled: vec![],
        }
    }
}

// Custom serde implementations for enums

mod delete_method_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(method: &DeleteMethod, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match method {
            DeleteMethod::Trash => "trash",
            DeleteMethod::Permanent => "permanent",
            DeleteMethod::DryRun => "dry-run",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DeleteMethod, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DeleteMethod::from_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid delete method: {}", s))
        })
    }
}

mod protection_level_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(level: &ProtectionLevel, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(level.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ProtectionLevel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ProtectionLevel::from_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid protection level: {}", s))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.general.default_paths.is_empty());
        assert!(config.scan.skip_hidden);
        assert_eq!(config.clean.delete_method, DeleteMethod::Trash);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[scan]"));
        assert!(toml_str.contains("[clean]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[general]
verbose = true
log_level = "debug"

[scan]
max_depth = 10
skip_hidden = false

[clean]
delete_method = "permanent"
protection_level = "block"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.general.verbose);
        assert_eq!(config.general.log_level, "debug");
        assert_eq!(config.scan.max_depth, Some(10));
        assert!(!config.scan.skip_hidden);
        assert_eq!(config.clean.delete_method, DeleteMethod::Permanent);
        assert_eq!(config.clean.protection_level, ProtectionLevel::Block);
    }
}
