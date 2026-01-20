//! IDE cleanup module
//!
//! Handles cleanup of IDE caches and data:
//! - JetBrains IDEs (IntelliJ, PyCharm, WebStorm, etc.)
//! - VS Code
//! - Sublime Text
//! - Cursor

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// IDE cleaner
pub struct IdeCleaner {
    home: PathBuf,
}

impl IdeCleaner {
    /// Create a new IDE cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all IDE cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // JetBrains
        items.extend(self.detect_jetbrains()?);

        // VS Code
        items.extend(self.detect_vscode()?);

        // Cursor (VS Code fork for AI)
        items.extend(self.detect_cursor()?);

        // Sublime Text
        items.extend(self.detect_sublime()?);

        // Zed
        items.extend(self.detect_zed()?);

        Ok(items)
    }

    /// Detect JetBrains IDE caches
    fn detect_jetbrains(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // JetBrains stores data in different locations per OS
        #[cfg(target_os = "macos")]
        {
            items.extend(self.detect_jetbrains_macos()?);
        }

        #[cfg(target_os = "linux")]
        {
            items.extend(self.detect_jetbrains_linux()?);
        }

        #[cfg(target_os = "windows")]
        {
            items.extend(self.detect_jetbrains_windows()?);
        }

        Ok(items)
    }

    #[cfg(target_os = "macos")]
    fn detect_jetbrains_macos(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let jetbrains_products = [
            "IntelliJIdea",
            "PyCharm",
            "WebStorm",
            "PhpStorm",
            "CLion",
            "GoLand",
            "Rider",
            "RubyMine",
            "DataGrip",
            "AndroidStudio",
            "Fleet",
        ];

        // Check Library/Caches
        let caches_path = self.home.join("Library/Caches/JetBrains");
        if caches_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&caches_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 50_000_000 {
                        // Skip if less than 50MB
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("JetBrains Cache: {}", name),
                        category: "IDE".to_string(),
                        subcategory: "JetBrains".to_string(),
                        icon: "🧠",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: "IDE cache and indexes. Will be rebuilt on next open.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        // Check Library/Application Support
        let support_path = self.home.join("Library/Application Support/JetBrains");
        if support_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&support_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Check if this is an old version
                    let is_old_version = jetbrains_products.iter().any(|p| {
                        name.starts_with(p) && !name.contains("2024") && !name.contains("2025")
                    });

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 100_000_000 {
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("JetBrains Data: {}", name),
                        category: "IDE".to_string(),
                        subcategory: "JetBrains".to_string(),
                        icon: "🧠",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: if is_old_version {
                            "Old IDE version data. Safe to delete if not using this version."
                        } else {
                            "IDE settings, plugins, and history."
                        },
                        safe_to_delete: if is_old_version {
                            SafetyLevel::Safe
                        } else {
                            SafetyLevel::Caution
                        },
                        clean_command: None,
                    });
                }
            }
        }

        // Check Library/Logs
        let logs_path = self.home.join("Library/Logs/JetBrains");
        if logs_path.exists() {
            let (size, file_count) = calculate_dir_size(&logs_path)?;
            if size > 10_000_000 {
                items.push(CleanableItem {
                    name: "JetBrains Logs".to_string(),
                    category: "IDE".to_string(),
                    subcategory: "JetBrains".to_string(),
                    icon: "📝",
                    path: logs_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "IDE log files. Safe to delete.",
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    #[cfg(target_os = "linux")]
    fn detect_jetbrains_linux(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Linux: ~/.cache/JetBrains and ~/.local/share/JetBrains
        let cache_path = self.home.join(".cache/JetBrains");
        if cache_path.exists() {
            let (size, file_count) = calculate_dir_size(&cache_path)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "JetBrains Cache".to_string(),
                    category: "IDE".to_string(),
                    subcategory: "JetBrains".to_string(),
                    icon: "🧠",
                    path: cache_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "IDE cache and indexes.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        let config_path = self.home.join(".config/JetBrains");
        if config_path.exists() {
            let (size, file_count) = calculate_dir_size(&config_path)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "JetBrains Config".to_string(),
                    category: "IDE".to_string(),
                    subcategory: "JetBrains".to_string(),
                    icon: "🧠",
                    path: config_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "IDE settings and plugins.",
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    #[cfg(target_os = "windows")]
    fn detect_jetbrains_windows(&self) -> Result<Vec<CleanableItem>> {
        // Windows: ~/AppData/Local/JetBrains and ~/AppData/Roaming/JetBrains
        let mut items = Vec::new();

        let local_path = self.home.join("AppData/Local/JetBrains");
        if local_path.exists() {
            let (size, file_count) = calculate_dir_size(&local_path)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "JetBrains Local Data".to_string(),
                    category: "IDE".to_string(),
                    subcategory: "JetBrains".to_string(),
                    icon: "🧠",
                    path: local_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "IDE cache and indexes.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect VS Code caches
    fn detect_vscode(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let vscode_paths = [
            ("Library/Application Support/Code/CachedData", "VS Code Cached Data"),
            ("Library/Application Support/Code/CachedExtensions", "VS Code Cached Extensions"),
            ("Library/Application Support/Code/CachedExtensionVSIXs", "VS Code Extension VSIXs"),
            ("Library/Application Support/Code/Cache", "VS Code Cache"),
            ("Library/Application Support/Code/User/workspaceStorage", "VS Code Workspace Storage"),
            ("Library/Caches/com.microsoft.VSCode", "VS Code System Cache"),
            ("Library/Caches/com.microsoft.VSCode.ShipIt", "VS Code Update Cache"),
        ];

        #[cfg(target_os = "linux")]
        let vscode_paths = [
            (".config/Code/CachedData", "VS Code Cached Data"),
            (".config/Code/CachedExtensions", "VS Code Cached Extensions"),
            (".config/Code/Cache", "VS Code Cache"),
            (".config/Code/User/workspaceStorage", "VS Code Workspace Storage"),
        ];

        #[cfg(target_os = "windows")]
        let vscode_paths = [
            ("AppData/Roaming/Code/CachedData", "VS Code Cached Data"),
            ("AppData/Roaming/Code/CachedExtensions", "VS Code Cached Extensions"),
            ("AppData/Roaming/Code/Cache", "VS Code Cache"),
            ("AppData/Roaming/Code/User/workspaceStorage", "VS Code Workspace Storage"),
        ];

        for (rel_path, name) in vscode_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                // Skip if less than 50MB
                continue;
            }

            let is_workspace = rel_path.contains("workspaceStorage");

            items.push(CleanableItem {
                name: name.to_string(),
                category: "IDE".to_string(),
                subcategory: "VS Code".to_string(),
                icon: "💻",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: if is_workspace {
                    "Workspace-specific cache. May include state for closed projects."
                } else {
                    "VS Code cache. Will be rebuilt on next open."
                },
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Cursor IDE caches (VS Code fork)
    fn detect_cursor(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let cursor_paths = [
            ("Library/Application Support/Cursor/CachedData", "Cursor Cached Data"),
            ("Library/Application Support/Cursor/Cache", "Cursor Cache"),
            ("Library/Application Support/Cursor/User/workspaceStorage", "Cursor Workspace Storage"),
            ("Library/Caches/com.todesktop.230313mzl4w4u92", "Cursor System Cache"),
        ];

        #[cfg(not(target_os = "macos"))]
        let cursor_paths: [(&str, &str); 0] = [];

        for (rel_path, name) in cursor_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "IDE".to_string(),
                subcategory: "Cursor".to_string(),
                icon: "🖱️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Cursor IDE cache. Will be rebuilt on next open.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Sublime Text caches
    fn detect_sublime(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let sublime_paths = [
            ("Library/Application Support/Sublime Text/Cache", "Sublime Cache"),
            ("Library/Application Support/Sublime Text/Index", "Sublime Index"),
            ("Library/Caches/com.sublimetext.4", "Sublime System Cache"),
        ];

        #[cfg(target_os = "linux")]
        let sublime_paths = [
            (".config/sublime-text/Cache", "Sublime Cache"),
            (".config/sublime-text/Index", "Sublime Index"),
        ];

        #[cfg(target_os = "windows")]
        let sublime_paths = [
            ("AppData/Roaming/Sublime Text/Cache", "Sublime Cache"),
            ("AppData/Roaming/Sublime Text/Index", "Sublime Index"),
        ];

        for (rel_path, name) in sublime_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 10_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "IDE".to_string(),
                subcategory: "Sublime Text".to_string(),
                icon: "📝",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Sublime Text cache and index files.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Zed editor caches
    fn detect_zed(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            let zed_cache = self.home.join("Library/Caches/dev.zed.Zed");
            if zed_cache.exists() {
                let (size, file_count) = calculate_dir_size(&zed_cache)?;
                if size > 50_000_000 {
                    items.push(CleanableItem {
                        name: "Zed Cache".to_string(),
                        category: "IDE".to_string(),
                        subcategory: "Zed".to_string(),
                        icon: "⚡",
                        path: zed_cache,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Zed editor cache files.",
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }
}

impl Default for IdeCleaner {
    fn default() -> Self {
        Self::new().expect("IdeCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_cleaner_creation() {
        let cleaner = IdeCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_ide_detection() {
        if let Some(cleaner) = IdeCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} IDE items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
