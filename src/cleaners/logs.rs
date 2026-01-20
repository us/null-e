//! Logs cleanup module
//!
//! Handles cleanup of various log files:
//! - System logs
//! - Application logs
//! - Development tool logs
//! - Crash reports

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Logs cleaner
pub struct LogsCleaner {
    home: PathBuf,
}

impl LogsCleaner {
    /// Create a new logs cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all log cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // User logs
        items.extend(self.detect_user_logs()?);

        // Development tool logs
        items.extend(self.detect_dev_logs()?);

        // Crash reports
        items.extend(self.detect_crash_reports()?);

        // npm/yarn logs
        items.extend(self.detect_package_manager_logs()?);

        Ok(items)
    }

    /// Detect user-level logs
    fn detect_user_logs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // ~/Library/Logs
            let logs_path = self.home.join("Library/Logs");
            if logs_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&logs_path) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // Skip certain system-critical logs
                        if name == "DiagnosticReports" || name == "com.apple.xpc.launchd" {
                            continue;
                        }

                        let (size, file_count) = if path.is_dir() {
                            calculate_dir_size(&path)?
                        } else if path.is_file() {
                            (std::fs::metadata(&path)?.len(), 1)
                        } else {
                            continue;
                        };

                        if size < 10_000_000 {
                            // Skip if less than 10MB
                            continue;
                        }

                        items.push(CleanableItem {
                            name: format!("Logs: {}", name),
                            category: "Logs".to_string(),
                            subcategory: "Application Logs".to_string(),
                            icon: "📝",
                            path,
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&entry.path()),
                            description: "Application log files. Usually safe to delete.",
                            safe_to_delete: SafetyLevel::Safe,
                            clean_command: None,
                        });
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // ~/.local/share/*/logs or ~/.config/*/logs
            let local_share = self.home.join(".local/share");
            if local_share.exists() {
                if let Ok(entries) = std::fs::read_dir(&local_share) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let logs_path = entry.path().join("logs");
                        if logs_path.exists() {
                            let (size, file_count) = calculate_dir_size(&logs_path)?;
                            if size > 10_000_000 {
                                let name = entry.file_name().to_string_lossy().to_string();
                                items.push(CleanableItem {
                                    name: format!("Logs: {}", name),
                                    category: "Logs".to_string(),
                                    subcategory: "Application Logs".to_string(),
                                    icon: "📝",
                                    path: logs_path,
                                    size,
                                    file_count: Some(file_count),
                                    last_modified: None,
                                    description: "Application log files.",
                                    safe_to_delete: SafetyLevel::Safe,
                                    clean_command: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect development tool logs
    fn detect_dev_logs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let dev_log_paths = [
            // Homebrew
            ("Library/Logs/Homebrew", "Homebrew Logs", "🍺"),
            // Git
            (".git/logs", "Git Logs", "📚"),
            // npm
            (".npm/_logs", "npm Logs", "📦"),
            // Yarn
            (".yarn/logs", "Yarn Logs", "🧶"),
            // pip
            (".pip/log", "pip Logs", "🐍"),
            // Gradle
            (".gradle/daemon", "Gradle Daemon Logs", "🐘"),
            // Cargo
            (".cargo/.package-cache", "Cargo Logs", "🦀"),
        ];

        for (rel_path, name, icon) in dev_log_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = if path.is_dir() {
                calculate_dir_size(&path)?
            } else {
                (std::fs::metadata(&path)?.len(), 1)
            };

            if size < 5_000_000 {
                // Skip if less than 5MB
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Logs".to_string(),
                subcategory: "Dev Tool Logs".to_string(),
                icon,
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Development tool log files. Safe to delete.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect crash reports
    fn detect_crash_reports(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            let crash_paths = [
                ("Library/Logs/DiagnosticReports", "Crash Reports"),
                ("Library/Logs/CrashReporter", "Crash Reporter"),
            ];

            for (rel_path, name) in crash_paths {
                let path = self.home.join(rel_path);
                if !path.exists() {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 5_000_000 {
                    continue;
                }

                items.push(CleanableItem {
                    name: name.to_string(),
                    category: "Logs".to_string(),
                    subcategory: "Crash Reports".to_string(),
                    icon: "💥",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Application crash reports. Safe to delete old ones.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect package manager logs
    fn detect_package_manager_logs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // npm debug logs (npm-debug.log files can accumulate)
        let npm_logs = self.home.join(".npm/_logs");
        if npm_logs.exists() {
            let (size, file_count) = calculate_dir_size(&npm_logs)?;
            if size > 1_000_000 {
                items.push(CleanableItem {
                    name: "npm Debug Logs".to_string(),
                    category: "Logs".to_string(),
                    subcategory: "Package Manager".to_string(),
                    icon: "📦",
                    path: npm_logs,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "npm installation and error logs. Safe to delete.",
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }
}

impl Default for LogsCleaner {
    fn default() -> Self {
        Self::new().expect("LogsCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_cleaner_creation() {
        let cleaner = LogsCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_logs_detection() {
        if let Some(cleaner) = LogsCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} log items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
