//! Electron Apps cleanup module
//!
//! Handles cleanup of Electron-based application caches:
//! - Slack
//! - Discord
//! - Spotify
//! - Microsoft Teams
//! - Notion
//! - Figma
//! - And many more Electron/Chromium apps

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Known Electron apps with their cache locations
const ELECTRON_APPS: &[(&str, &str, &str)] = &[
    // (App Name, Application Support folder name, Icon)
    ("Slack", "Slack", "💬"),
    ("Discord", "discord", "🎮"),
    ("Spotify", "Spotify", "🎵"),
    ("Microsoft Teams", "Microsoft Teams", "👥"),
    ("Microsoft Teams (Classic)", "Microsoft/Teams", "👥"),
    ("Notion", "Notion", "📝"),
    ("Figma", "Figma", "🎨"),
    ("Obsidian", "obsidian", "💎"),
    ("Postman", "Postman", "📮"),
    ("Insomnia", "Insomnia", "🌙"),
    ("Hyper", "Hyper", "⚡"),
    ("GitKraken", "GitKraken", "🐙"),
    ("Atom", "Atom", "⚛️"),
    ("Signal", "Signal", "🔒"),
    ("WhatsApp", "WhatsApp", "📱"),
    ("Telegram Desktop", "Telegram Desktop", "✈️"),
    ("Linear", "Linear", "📊"),
    ("Loom", "Loom", "🎥"),
    ("Cron", "Cron", "📅"),
    ("Raycast", "com.raycast.macos", "🔍"),
    ("1Password", "1Password", "🔐"),
    ("Bitwarden", "Bitwarden", "🔐"),
    ("Franz", "Franz", "📬"),
    ("Station", "Station", "🚉"),
    ("Skype", "Skype", "📞"),
    ("Zoom", "zoom.us", "📹"),
    ("Webex", "Cisco Webex Meetings", "🌐"),
    ("Miro", "Miro", "🖼️"),
    ("ClickUp", "ClickUp", "✅"),
    ("Todoist", "Todoist", "☑️"),
    ("Trello", "Trello", "📋"),
];

/// Electron apps cleaner
pub struct ElectronCleaner {
    home: PathBuf,
}

impl ElectronCleaner {
    /// Create a new Electron apps cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all Electron app cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            items.extend(self.detect_macos_apps()?);
        }

        #[cfg(target_os = "linux")]
        {
            items.extend(self.detect_linux_apps()?);
        }

        #[cfg(target_os = "windows")]
        {
            items.extend(self.detect_windows_apps()?);
        }

        Ok(items)
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_apps(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let app_support = self.home.join("Library/Application Support");
        let caches = self.home.join("Library/Caches");

        for (app_name, folder_name, icon) in ELECTRON_APPS {
            let mut app_items = Vec::new();

            // Check Application Support
            let support_path = app_support.join(folder_name);
            if support_path.exists() {
                // Check for specific cache directories within
                let cache_subdirs = ["Cache", "CachedData", "GPUCache", "Code Cache", "Service Worker", "blob_storage"];

                for subdir in cache_subdirs {
                    let cache_path = support_path.join(subdir);
                    if cache_path.exists() {
                        let (size, file_count) = calculate_dir_size(&cache_path)?;
                        if size > 20_000_000 {
                            app_items.push((cache_path, size, file_count, subdir));
                        }
                    }
                }

                // Also check for old versions (common in Electron apps)
                if let Ok(entries) = std::fs::read_dir(&support_path) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Old versions often have version numbers
                        if name.starts_with("app-") || name.contains(".old") {
                            let (size, file_count) = calculate_dir_size(&entry.path())?;
                            if size > 50_000_000 {
                                app_items.push((entry.path(), size, file_count, "Old Version"));
                            }
                        }
                    }
                }
            }

            // Check Caches folder
            let cache_variants = [
                folder_name.to_string(),
                format!("com.{}.desktop", folder_name.to_lowercase()),
                format!("com.{}", folder_name.to_lowercase()),
            ];

            for variant in cache_variants {
                let cache_path = caches.join(&variant);
                if cache_path.exists() {
                    let (size, file_count) = calculate_dir_size(&cache_path)?;
                    if size > 30_000_000 {
                        app_items.push((cache_path, size, file_count, "System Cache"));
                    }
                }
            }

            // Group small caches together, list large ones separately
            let total_size: u64 = app_items.iter().map(|(_, s, _, _)| *s).sum();

            if total_size > 50_000_000 {
                if app_items.len() == 1 {
                    let (path, size, file_count, cache_type) = app_items.remove(0);
                    items.push(CleanableItem {
                        name: format!("{} {}", app_name, cache_type),
                        category: "Electron Apps".to_string(),
                        subcategory: app_name.to_string(),
                        icon,
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Electron app cache. Will be rebuilt on next launch.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                } else {
                    // Report the main Application Support folder
                    let support_path = app_support.join(folder_name);
                    if support_path.exists() {
                        items.push(CleanableItem {
                            name: format!("{} Caches", app_name),
                            category: "Electron Apps".to_string(),
                            subcategory: app_name.to_string(),
                            icon,
                            path: support_path,
                            size: total_size,
                            file_count: Some(app_items.iter().map(|(_, _, c, _)| *c).sum()),
                            last_modified: None,
                            description: "Electron app cache and data. Consider cleaning individual subdirectories.",
                            safe_to_delete: SafetyLevel::Caution,
                            clean_command: None,
                        });
                    }
                }
            }
        }

        // Also detect any other Electron-like apps by looking for Chromium cache patterns
        items.extend(self.detect_chromium_caches()?);

        Ok(items)
    }

    #[cfg(target_os = "macos")]
    fn detect_chromium_caches(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let caches = self.home.join("Library/Caches");
        if !caches.exists() {
            return Ok(items);
        }

        // Look for Chromium-style cache patterns
        if let Ok(entries) = std::fs::read_dir(&caches) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip known apps we already handle
                let already_handled = ELECTRON_APPS.iter().any(|(_, folder, _)| {
                    name.to_lowercase().contains(&folder.to_lowercase())
                });

                if already_handled {
                    continue;
                }

                // Check if it looks like an Electron app (has GPUCache or similar)
                let gpu_cache = path.join("GPUCache");
                let code_cache = path.join("Code Cache");

                if gpu_cache.exists() || code_cache.exists() {
                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size > 100_000_000 {
                        items.push(CleanableItem {
                            name: format!("App Cache: {}", name),
                            category: "Electron Apps".to_string(),
                            subcategory: "Other".to_string(),
                            icon: "📦",
                            path,
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&entry.path()),
                            description: "Electron/Chromium-based app cache.",
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: None,
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_apps(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let config_dir = self.home.join(".config");

        for (app_name, folder_name, icon) in ELECTRON_APPS {
            let app_path = config_dir.join(folder_name);
            if !app_path.exists() {
                continue;
            }

            // Check for cache subdirectories
            let cache_path = app_path.join("Cache");
            if cache_path.exists() {
                let (size, file_count) = calculate_dir_size(&cache_path)?;
                if size > 50_000_000 {
                    items.push(CleanableItem {
                        name: format!("{} Cache", app_name),
                        category: "Electron Apps".to_string(),
                        subcategory: app_name.to_string(),
                        icon,
                        path: cache_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Electron app cache.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_apps(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let appdata = self.home.join("AppData/Roaming");

        for (app_name, folder_name, icon) in ELECTRON_APPS {
            let app_path = appdata.join(folder_name);
            if !app_path.exists() {
                continue;
            }

            let cache_path = app_path.join("Cache");
            if cache_path.exists() {
                let (size, file_count) = calculate_dir_size(&cache_path)?;
                if size > 50_000_000 {
                    items.push(CleanableItem {
                        name: format!("{} Cache", app_name),
                        category: "Electron Apps".to_string(),
                        subcategory: app_name.to_string(),
                        icon,
                        path: cache_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Electron app cache.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }
}

impl Default for ElectronCleaner {
    fn default() -> Self {
        Self::new().expect("ElectronCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electron_cleaner_creation() {
        let cleaner = ElectronCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_electron_detection() {
        if let Some(cleaner) = ElectronCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} Electron app items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
