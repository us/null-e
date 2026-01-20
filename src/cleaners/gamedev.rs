//! Game Development cleanup module
//!
//! Handles cleanup of game development artifacts:
//! - Unity (Library, Temp, Builds, Logs)
//! - Unreal Engine (Intermediate, Saved, DerivedDataCache)
//! - Godot (cache, .import)

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Game Development cleaner
pub struct GameDevCleaner {
    home: PathBuf,
}

impl GameDevCleaner {
    /// Create a new game dev cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all game development cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Unity Hub and Editor caches
        items.extend(self.detect_unity_global()?);

        // Unreal Engine global caches
        items.extend(self.detect_unreal_global()?);

        // Godot global caches
        items.extend(self.detect_godot_global()?);

        Ok(items)
    }

    /// Detect Unity global caches (Unity Hub, Editor)
    fn detect_unity_global(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let unity_paths = [
            ("Library/Application Support/Unity", "Unity Settings", "🎮"),
            ("Library/Caches/com.unity3d.UnityEditor", "Unity Editor Cache", "🎮"),
            ("Library/Unity/Asset Store-5.x", "Unity Asset Store Cache", "🛒"),
            ("Library/Unity/cache", "Unity Global Cache", "🎮"),
            ("Library/Logs/Unity", "Unity Logs", "📝"),
        ];

        #[cfg(target_os = "linux")]
        let unity_paths = [
            (".config/unity3d", "Unity Settings", "🎮"),
            (".cache/unity3d", "Unity Cache", "🎮"),
            (".local/share/unity3d/Asset Store-5.x", "Unity Asset Store Cache", "🛒"),
        ];

        #[cfg(target_os = "windows")]
        let unity_paths = [
            ("AppData/Roaming/Unity", "Unity Settings", "🎮"),
            ("AppData/Local/Unity/cache", "Unity Cache", "🎮"),
            ("AppData/Roaming/Unity/Asset Store-5.x", "Unity Asset Store Cache", "🛒"),
        ];

        for (rel_path, name, icon) in unity_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 100_000_000 {
                continue;
            }

            let is_asset_store = rel_path.contains("Asset Store");

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Game Development".to_string(),
                subcategory: "Unity".to_string(),
                icon,
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: if is_asset_store {
                    "Downloaded Asset Store packages. Can be re-downloaded."
                } else {
                    "Unity Editor cache and data. Will be rebuilt."
                },
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        // Unity Hub
        #[cfg(target_os = "macos")]
        {
            let hub_path = self.home.join("Library/Application Support/UnityHub");
            if hub_path.exists() {
                let (size, file_count) = calculate_dir_size(&hub_path)?;
                if size > 100_000_000 {
                    items.push(CleanableItem {
                        name: "Unity Hub Cache".to_string(),
                        category: "Game Development".to_string(),
                        subcategory: "Unity".to_string(),
                        icon: "🎮",
                        path: hub_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Unity Hub installer cache.",
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Unreal Engine global caches
    fn detect_unreal_global(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let unreal_paths = [
            ("Library/Application Support/Epic", "Epic Games Cache", "🎯"),
            ("Library/Caches/com.epicgames.UnrealEngine", "Unreal Engine Cache", "🎯"),
            ("Library/Application Support/Unreal Engine", "Unreal Engine Data", "🎯"),
        ];

        #[cfg(target_os = "linux")]
        let unreal_paths = [
            (".config/Epic", "Epic Games Config", "🎯"),
            (".cache/UnrealEngine", "Unreal Engine Cache", "🎯"),
        ];

        #[cfg(target_os = "windows")]
        let unreal_paths = [
            ("AppData/Local/EpicGamesLauncher", "Epic Games Launcher", "🎯"),
            ("AppData/Local/UnrealEngine", "Unreal Engine Cache", "🎯"),
        ];

        for (rel_path, name, icon) in unreal_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 100_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Game Development".to_string(),
                subcategory: "Unreal Engine".to_string(),
                icon,
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Unreal Engine cache and shader data.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        // DerivedDataCache (can be huge)
        #[cfg(target_os = "macos")]
        {
            let ddc_path = self.home.join("Library/Application Support/Unreal Engine/Common/DerivedDataCache");
            if ddc_path.exists() {
                let (size, file_count) = calculate_dir_size(&ddc_path)?;
                if size > 500_000_000 {
                    items.push(CleanableItem {
                        name: "Unreal Derived Data Cache".to_string(),
                        category: "Game Development".to_string(),
                        subcategory: "Unreal Engine".to_string(),
                        icon: "🎯",
                        path: ddc_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Shared Derived Data Cache. Can be very large. Will be rebuilt.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Godot global caches
    fn detect_godot_global(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        let godot_paths = [
            ("Library/Application Support/Godot", "Godot Data", "🤖"),
            ("Library/Caches/Godot", "Godot Cache", "🤖"),
        ];

        #[cfg(target_os = "linux")]
        let godot_paths = [
            (".config/godot", "Godot Config", "🤖"),
            (".cache/godot", "Godot Cache", "🤖"),
            (".local/share/godot", "Godot Data", "🤖"),
        ];

        #[cfg(target_os = "windows")]
        let godot_paths = [
            ("AppData/Roaming/Godot", "Godot Data", "🤖"),
            ("AppData/Local/Godot", "Godot Cache", "🤖"),
        ];

        for (rel_path, name, icon) in godot_paths {
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
                category: "Game Development".to_string(),
                subcategory: "Godot".to_string(),
                icon,
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Godot engine cache and editor data.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Scan for Unity project folders and their cleanable directories
    pub fn scan_unity_projects(&self, search_path: &PathBuf) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // This would be called during a project scan
        // Unity projects have: Library/, Temp/, Logs/, Builds/
        let unity_cleanable = ["Library", "Temp", "Logs", "Builds", "obj"];

        for dir_name in unity_cleanable {
            let path = search_path.join(dir_name);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                continue;
            }

            let safety = match dir_name {
                "Temp" | "Logs" => SafetyLevel::Safe,
                "Library" => SafetyLevel::SafeWithCost,
                "Builds" => SafetyLevel::Caution,
                _ => SafetyLevel::Safe,
            };

            items.push(CleanableItem {
                name: format!("Unity {}", dir_name),
                category: "Game Development".to_string(),
                subcategory: "Unity Project".to_string(),
                icon: "🎮",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: match dir_name {
                    "Library" => "Unity Library cache. Will be rebuilt on project open.",
                    "Temp" => "Temporary build files. Safe to delete.",
                    "Logs" => "Unity log files. Safe to delete.",
                    "Builds" => "Build output. Check if needed before deleting.",
                    _ => "Unity project files.",
                },
                safe_to_delete: safety,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Scan for Unreal project folders and their cleanable directories
    pub fn scan_unreal_projects(&self, search_path: &PathBuf) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Unreal projects have: Intermediate/, Saved/, DerivedDataCache/, Binaries/
        let unreal_cleanable = ["Intermediate", "Saved", "DerivedDataCache", "Binaries"];

        for dir_name in unreal_cleanable {
            let path = search_path.join(dir_name);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 100_000_000 {
                continue;
            }

            let safety = match dir_name {
                "Intermediate" | "DerivedDataCache" => SafetyLevel::SafeWithCost,
                "Saved" => SafetyLevel::Caution, // May contain saves
                "Binaries" => SafetyLevel::SafeWithCost,
                _ => SafetyLevel::Safe,
            };

            items.push(CleanableItem {
                name: format!("Unreal {}", dir_name),
                category: "Game Development".to_string(),
                subcategory: "Unreal Project".to_string(),
                icon: "🎯",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: match dir_name {
                    "Intermediate" => "Build intermediate files. Will be rebuilt.",
                    "DerivedDataCache" => "Shader and asset cache. Will be rebuilt.",
                    "Saved" => "Saved data including autosaves. Check before deleting.",
                    "Binaries" => "Compiled binaries. Will be rebuilt.",
                    _ => "Unreal project files.",
                },
                safe_to_delete: safety,
                clean_command: None,
            });
        }

        Ok(items)
    }
}

impl Default for GameDevCleaner {
    fn default() -> Self {
        Self::new().expect("GameDevCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamedev_cleaner_creation() {
        let cleaner = GameDevCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_gamedev_detection() {
        if let Some(cleaner) = GameDevCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} game dev items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
