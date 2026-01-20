//! iOS Dependency Managers cleanup module
//!
//! Handles cleanup of iOS/macOS dependency manager caches:
//! - CocoaPods cache
//! - Carthage cache and builds
//! - Swift Package Manager (SPM) cache

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// iOS Dependency caches cleaner
pub struct IosDependencyCleaner {
    home: PathBuf,
}

impl IosDependencyCleaner {
    /// Create a new iOS dependency cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all iOS dependency cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // CocoaPods
        items.extend(self.detect_cocoapods()?);

        // Carthage
        items.extend(self.detect_carthage()?);

        // Swift Package Manager
        items.extend(self.detect_spm()?);

        Ok(items)
    }

    /// Detect CocoaPods caches
    fn detect_cocoapods(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let cocoapods_paths = [
            ("Library/Caches/CocoaPods", "CocoaPods Cache"),
            ("Library/Caches/CocoaPods/Pods", "CocoaPods Pods Cache"),
            (".cocoapods/repos", "CocoaPods Repos"),
        ];

        for (rel_path, name) in cocoapods_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                continue;
            }

            let is_repos = rel_path.contains("repos");

            items.push(CleanableItem {
                name: name.to_string(),
                category: "iOS Dependencies".to_string(),
                subcategory: "CocoaPods".to_string(),
                icon: "🥥",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: if is_repos {
                    "CocoaPods spec repositories. Will be re-downloaded on next pod install."
                } else {
                    "CocoaPods download cache. Safe to delete."
                },
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("pod cache clean --all".to_string()),
            });
        }

        Ok(items)
    }

    /// Detect Carthage caches
    fn detect_carthage(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Carthage cache
        let carthage_cache = self.home.join("Library/Caches/org.carthage.CarthageKit");
        if carthage_cache.exists() {
            let (size, file_count) = calculate_dir_size(&carthage_cache)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "Carthage Cache".to_string(),
                    category: "iOS Dependencies".to_string(),
                    subcategory: "Carthage".to_string(),
                    icon: "🏛️",
                    path: carthage_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Carthage dependency cache. Will be rebuilt on next carthage bootstrap.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        // Carthage derived data (in DerivedData)
        let derived_data = self.home.join("Library/Developer/Xcode/DerivedData");
        if derived_data.exists() {
            if let Ok(entries) = std::fs::read_dir(&derived_data) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Carthage builds often have "Carthage" in the name
                    if name.contains("Carthage") {
                        let (size, file_count) = calculate_dir_size(&path)?;
                        if size > 100_000_000 {
                            items.push(CleanableItem {
                                name: format!("Carthage Build: {}", name),
                                category: "iOS Dependencies".to_string(),
                                subcategory: "Carthage".to_string(),
                                icon: "🏛️",
                                path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&entry.path()),
                                description: "Carthage build artifacts. Will be rebuilt on next build.",
                                safe_to_delete: SafetyLevel::SafeWithCost,
                                clean_command: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect Swift Package Manager caches
    fn detect_spm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let spm_paths = [
            ("Library/Caches/org.swift.swiftpm", "SPM Cache"),
            ("Library/org.swift.swiftpm", "SPM Data"),
            ("Library/Developer/Xcode/DerivedData/*/SourcePackages", "SPM Source Packages"),
        ];

        for (rel_path, name) in spm_paths {
            // Handle wildcard pattern
            if rel_path.contains('*') {
                let base = rel_path.split('*').next().unwrap_or("");
                let suffix = rel_path.split('*').nth(1).unwrap_or("");
                let base_path = self.home.join(base);

                if base_path.exists() {
                    if let Ok(entries) = std::fs::read_dir(&base_path) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let source_packages = entry.path().join(suffix.trim_start_matches('/'));
                            if source_packages.exists() {
                                let (size, file_count) = calculate_dir_size(&source_packages)?;
                                if size > 50_000_000 {
                                    let project_name = entry.file_name().to_string_lossy().to_string();
                                    items.push(CleanableItem {
                                        name: format!("SPM Packages: {}", project_name.split('-').next().unwrap_or(&project_name)),
                                        category: "iOS Dependencies".to_string(),
                                        subcategory: "Swift Package Manager".to_string(),
                                        icon: "📦",
                                        path: source_packages,
                                        size,
                                        file_count: Some(file_count),
                                        last_modified: get_mtime(&entry.path()),
                                        description: "Swift Package Manager downloaded packages. Will be re-downloaded.",
                                        safe_to_delete: SafetyLevel::SafeWithCost,
                                        clean_command: Some("swift package purge-cache".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
                continue;
            }

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
                category: "iOS Dependencies".to_string(),
                subcategory: "Swift Package Manager".to_string(),
                icon: "📦",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Swift Package Manager cache. Will be rebuilt on next build.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("swift package purge-cache".to_string()),
            });
        }

        // Xcode's SPM cache
        let xcode_spm_cache = self.home.join("Library/Caches/com.apple.dt.Xcode");
        if xcode_spm_cache.exists() {
            let (size, file_count) = calculate_dir_size(&xcode_spm_cache)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "Xcode SPM Cache".to_string(),
                    category: "iOS Dependencies".to_string(),
                    subcategory: "Swift Package Manager".to_string(),
                    icon: "📦",
                    path: xcode_spm_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Xcode's Swift Package Manager cache.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }
}

impl Default for IosDependencyCleaner {
    fn default() -> Self {
        Self::new().expect("IosDependencyCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_deps_cleaner_creation() {
        let cleaner = IosDependencyCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_ios_deps_detection() {
        if let Some(cleaner) = IosDependencyCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} iOS dependency items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
