//! Homebrew cleanup module
//!
//! Handles cleanup of Homebrew caches:
//! - Downloaded formula archives
//! - Old package versions
//! - Outdated casks

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;
use std::process::Command;

/// Homebrew cleaner
pub struct HomebrewCleaner {
    home: PathBuf,
    cache_path: PathBuf,
}

impl HomebrewCleaner {
    /// Create a new Homebrew cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;

        // Homebrew cache location
        let cache_path = home.join("Library/Caches/Homebrew");

        Some(Self { home, cache_path })
    }

    /// Check if Homebrew is installed
    pub fn is_available(&self) -> bool {
        Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Detect all Homebrew cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Main cache directory
        if self.cache_path.exists() {
            items.extend(self.detect_cache()?);
        }

        // Downloads subdirectory
        let downloads_path = self.cache_path.join("downloads");
        if downloads_path.exists() {
            let (size, file_count) = calculate_dir_size(&downloads_path)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "Homebrew Downloads".to_string(),
                    category: "Package Manager".to_string(),
                    subcategory: "Homebrew".to_string(),
                    icon: "🍺",
                    path: downloads_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Downloaded formula archives. Safe to delete.",
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some("brew cleanup".to_string()),
                });
            }
        }

        // Cask downloads
        let cask_path = self.cache_path.join("Cask");
        if cask_path.exists() {
            let (size, file_count) = calculate_dir_size(&cask_path)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "Homebrew Cask Downloads".to_string(),
                    category: "Package Manager".to_string(),
                    subcategory: "Homebrew".to_string(),
                    icon: "🍺",
                    path: cask_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Downloaded cask application archives. Safe to delete.",
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some("brew cleanup --cask".to_string()),
                });
            }
        }

        // Old formula versions in Cellar
        items.extend(self.detect_old_versions()?);

        Ok(items)
    }

    /// Detect cache files
    fn detect_cache(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.cache_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                // Skip subdirectories we handle separately
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if name == "downloads" || name == "Cask" || name == "api" {
                    continue;
                }

                // Only look at files/directories that are cached packages
                if path.is_file() && (name.ends_with(".tar.gz") || name.ends_with(".bottle.tar.gz")) {
                    let size = std::fs::metadata(&path)?.len();
                    if size > 10_000_000 {
                        items.push(CleanableItem {
                            name: format!("Cached: {}", name),
                            category: "Package Manager".to_string(),
                            subcategory: "Homebrew".to_string(),
                            icon: "🍺",
                            path,
                            size,
                            file_count: Some(1),
                            last_modified: get_mtime(&entry.path()),
                            description: "Cached package archive. Safe to delete.",
                            safe_to_delete: SafetyLevel::Safe,
                            clean_command: Some("brew cleanup".to_string()),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect old formula versions
    fn detect_old_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Check Cellar for old versions
        let cellar_paths = [
            PathBuf::from("/usr/local/Cellar"),
            PathBuf::from("/opt/homebrew/Cellar"),
        ];

        for cellar in cellar_paths {
            if !cellar.exists() {
                continue;
            }

            if let Ok(formulas) = std::fs::read_dir(&cellar) {
                for formula in formulas.filter_map(|e| e.ok()) {
                    let formula_path = formula.path();
                    if !formula_path.is_dir() {
                        continue;
                    }

                    // Count versions
                    let versions: Vec<_> = std::fs::read_dir(&formula_path)
                        .ok()
                        .map(|entries| entries.filter_map(|e| e.ok()).collect())
                        .unwrap_or_default();

                    if versions.len() <= 1 {
                        continue;
                    }

                    // Calculate size of old versions (all but the latest)
                    let mut old_versions: Vec<_> = versions;
                    old_versions.sort_by(|a, b| {
                        let a_time = a.metadata().and_then(|m| m.modified()).ok();
                        let b_time = b.metadata().and_then(|m| m.modified()).ok();
                        b_time.cmp(&a_time)
                    });

                    // Skip the newest, sum the rest
                    let mut total_size = 0u64;
                    let mut total_files = 0u64;
                    for old_version in old_versions.iter().skip(1) {
                        if let Ok((size, count)) = calculate_dir_size(&old_version.path()) {
                            total_size += size;
                            total_files += count;
                        }
                    }

                    if total_size > 50_000_000 {
                        let formula_name = formula_path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        items.push(CleanableItem {
                            name: format!("Old versions: {}", formula_name),
                            category: "Package Manager".to_string(),
                            subcategory: "Homebrew".to_string(),
                            icon: "🍺",
                            path: formula_path,
                            size: total_size,
                            file_count: Some(total_files),
                            last_modified: None,
                            description: "Old formula versions. Use 'brew cleanup' to remove.",
                            safe_to_delete: SafetyLevel::Safe,
                            clean_command: Some(format!("brew cleanup {}", formula_name)),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    /// Run brew cleanup command
    pub fn clean_all(&self, scrub: bool) -> Result<u64> {
        let mut cmd = Command::new("brew");
        cmd.arg("cleanup");

        if scrub {
            cmd.arg("-s"); // Scrub cache, even latest versions
        }

        cmd.arg("--prune=all");

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::DevSweepError::Other(
                format!("brew cleanup failed: {}", stderr)
            ));
        }

        // Estimate freed space (brew doesn't report exact bytes)
        Ok(0)
    }
}

impl Default for HomebrewCleaner {
    fn default() -> Self {
        Self::new().expect("HomebrewCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_cleaner_creation() {
        let cleaner = HomebrewCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_homebrew_detection() {
        if let Some(cleaner) = HomebrewCleaner::new() {
            if cleaner.is_available() {
                let items = cleaner.detect().unwrap();
                println!("Found {} Homebrew items", items.len());
                for item in &items {
                    println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
                }
            } else {
                println!("Homebrew not installed");
            }
        }
    }
}
