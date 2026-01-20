//! macOS-specific cleanup module
//!
//! Handles cleanup of macOS-specific files:
//! - Orphaned app containers (~/Library/Containers)
//! - Group containers
//! - Application support remnants
//! - System caches

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::collections::HashSet;
use std::path::PathBuf;

/// macOS system cleaner
#[cfg(target_os = "macos")]
pub struct MacOsCleaner {
    home: PathBuf,
}

#[cfg(target_os = "macos")]
impl MacOsCleaner {
    /// Create a new macOS cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all macOS cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Orphaned containers
        items.extend(self.detect_orphaned_containers()?);

        // Large Library caches
        items.extend(self.detect_library_caches()?);

        // Application support remnants
        items.extend(self.detect_app_support_remnants()?);

        // Font caches
        items.extend(self.detect_font_caches()?);

        Ok(items)
    }

    /// Get list of installed applications
    fn get_installed_apps(&self) -> HashSet<String> {
        let mut apps = HashSet::new();

        let app_dirs = [
            PathBuf::from("/Applications"),
            self.home.join("Applications"),
        ];

        for app_dir in app_dirs {
            if let Ok(entries) = std::fs::read_dir(&app_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".app") {
                        let app_name = name.trim_end_matches(".app").to_lowercase();
                        apps.insert(app_name);

                        // Also extract bundle identifier if possible
                        let plist_path = entry.path().join("Contents/Info.plist");
                        if let Ok(content) = std::fs::read_to_string(&plist_path) {
                            // Simple extraction of bundle identifier
                            if let Some(start) = content.find("<key>CFBundleIdentifier</key>") {
                                if let Some(value_start) = content[start..].find("<string>") {
                                    let rest = &content[start + value_start + 8..];
                                    if let Some(value_end) = rest.find("</string>") {
                                        let bundle_id = rest[..value_end].to_lowercase();
                                        apps.insert(bundle_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        apps
    }

    /// Detect orphaned containers (sandboxed app data for uninstalled apps)
    fn detect_orphaned_containers(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let containers_path = self.home.join("Library/Containers");
        if !containers_path.exists() {
            return Ok(items);
        }

        let installed_apps = self.get_installed_apps();

        if let Ok(entries) = std::fs::read_dir(&containers_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let container_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip system containers
                if container_name.starts_with("com.apple.") {
                    continue;
                }

                // Check if this container belongs to an installed app
                let is_orphaned = !installed_apps.iter().any(|app| {
                    container_name.to_lowercase().contains(app)
                });

                if !is_orphaned {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 50_000_000 {
                    continue;
                }

                items.push(CleanableItem {
                    name: format!("Orphaned: {}", container_name),
                    category: "macOS System".to_string(),
                    subcategory: "Containers".to_string(),
                    icon: "📦",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: get_mtime(&entry.path()),
                    description: "Container data for possibly uninstalled app. Verify before deleting.",
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect large Library caches
    fn detect_library_caches(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let caches_path = self.home.join("Library/Caches");
        if !caches_path.exists() {
            return Ok(items);
        }

        // Skip patterns (we handle these in other cleaners)
        let skip_patterns = [
            "com.apple.",
            "homebrew",
            "cocoapods",
            "carthage",
            "JetBrains",
            "com.microsoft.VSCode",
            "Google",
            "Firefox",
            "Spotify",
            "Slack",
            "Discord",
        ];

        if let Ok(entries) = std::fs::read_dir(&caches_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip known patterns
                let should_skip = skip_patterns.iter().any(|p| {
                    name.to_lowercase().contains(&p.to_lowercase())
                });

                if should_skip {
                    continue;
                }

                let (size, file_count) = if path.is_dir() {
                    calculate_dir_size(&path)?
                } else {
                    (std::fs::metadata(&path)?.len(), 1)
                };

                if size < 100_000_000 {
                    continue;
                }

                items.push(CleanableItem {
                    name: format!("Cache: {}", name),
                    category: "macOS System".to_string(),
                    subcategory: "Caches".to_string(),
                    icon: "🗄️",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: get_mtime(&entry.path()),
                    description: "Application cache. Usually safe to delete.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect Application Support remnants
    fn detect_app_support_remnants(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let app_support = self.home.join("Library/Application Support");
        if !app_support.exists() {
            return Ok(items);
        }

        let installed_apps = self.get_installed_apps();

        // Skip patterns (system and known apps)
        let skip_patterns = [
            "com.apple.",
            "Apple",
            "Code",
            "JetBrains",
            "Google",
            "Microsoft",
            "Slack",
            "Discord",
            "Spotify",
            "AddressBook",
            "Dock",
            "iCloud",
        ];

        if let Ok(entries) = std::fs::read_dir(&app_support) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip known patterns
                let should_skip = skip_patterns.iter().any(|p| {
                    name.to_lowercase().contains(&p.to_lowercase())
                });

                if should_skip {
                    continue;
                }

                // Check if app is installed
                let is_orphaned = !installed_apps.iter().any(|app| {
                    name.to_lowercase().contains(app) || app.contains(&name.to_lowercase())
                });

                if !is_orphaned {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 100_000_000 {
                    continue;
                }

                items.push(CleanableItem {
                    name: format!("App Support: {}", name),
                    category: "macOS System".to_string(),
                    subcategory: "Application Support".to_string(),
                    icon: "📁",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: get_mtime(&entry.path()),
                    description: "Application data for possibly uninstalled app.",
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect font caches
    fn detect_font_caches(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let font_cache_paths = [
            "Library/Caches/com.apple.FontRegistry",
            "Library/Caches/FontExplorer X",
        ];

        for rel_path in font_cache_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Font Cache".to_string(),
                category: "macOS System".to_string(),
                subcategory: "Fonts".to_string(),
                icon: "🔤",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Font rendering cache. Will be rebuilt.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: Some("sudo atsutil databases -remove".to_string()),
            });
        }

        Ok(items)
    }
}

#[cfg(target_os = "macos")]
impl Default for MacOsCleaner {
    fn default() -> Self {
        Self::new().expect("MacOsCleaner requires home directory")
    }
}

// Stub for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub struct MacOsCleaner;

#[cfg(not(target_os = "macos"))]
impl MacOsCleaner {
    pub fn new() -> Option<Self> {
        Some(Self)
    }

    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        Ok(vec![])
    }
}

#[cfg(not(target_os = "macos"))]
impl Default for MacOsCleaner {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_cleaner_creation() {
        let cleaner = MacOsCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_detection() {
        if let Some(cleaner) = MacOsCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} macOS items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
