//! Xcode cleanup module
//!
//! Handles cleanup of Xcode-related files:
//! - DerivedData (build artifacts)
//! - Archives (old app builds)
//! - iOS DeviceSupport (debug symbols)
//! - Simulators (iOS/watchOS/tvOS)
//! - Xcode caches

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Xcode cleaner
pub struct XcodeCleaner {
    home: PathBuf,
}

impl XcodeCleaner {
    /// Create a new Xcode cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;

        // Only available on macOS
        #[cfg(not(target_os = "macos"))]
        return None;

        #[cfg(target_os = "macos")]
        Some(Self { home })
    }

    /// Detect all Xcode cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // DerivedData
        items.extend(self.detect_derived_data()?);

        // Archives
        items.extend(self.detect_archives()?);

        // iOS DeviceSupport
        items.extend(self.detect_device_support()?);

        // Simulators
        items.extend(self.detect_simulators()?);

        // Xcode Caches
        items.extend(self.detect_caches()?);

        // Documentation cache
        items.extend(self.detect_documentation()?);

        Ok(items)
    }

    /// Detect DerivedData folders
    fn detect_derived_data(&self) -> Result<Vec<CleanableItem>> {
        let derived_data = self.home
            .join("Library/Developer/Xcode/DerivedData");

        if !derived_data.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Each subdirectory is a project's derived data
        if let Ok(entries) = std::fs::read_dir(&derived_data) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Skip the ModuleCache as it's shared
                    if name == "ModuleCache" || name == "ModuleCache.noindex" {
                        continue;
                    }

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size == 0 {
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("DerivedData: {}", name.split('-').next().unwrap_or(&name)),
                        category: "Xcode".to_string(),
                        subcategory: "DerivedData".to_string(),
                        icon: "🔨",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: "Build artifacts, indexes, logs. Safe to delete.",
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Archives
    fn detect_archives(&self) -> Result<Vec<CleanableItem>> {
        let archives = self.home
            .join("Library/Developer/Xcode/Archives");

        if !archives.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Archives are organized by date
        if let Ok(date_dirs) = std::fs::read_dir(&archives) {
            for date_dir in date_dirs.filter_map(|e| e.ok()) {
                let date_path = date_dir.path();
                if !date_path.is_dir() {
                    continue;
                }

                if let Ok(archive_files) = std::fs::read_dir(&date_path) {
                    for archive in archive_files.filter_map(|e| e.ok()) {
                        let path = archive.path();
                        if path.extension().map(|e| e == "xcarchive").unwrap_or(false) {
                            let name = path.file_stem()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            let (size, file_count) = calculate_dir_size(&path)?;

                            items.push(CleanableItem {
                                name: format!("Archive: {}", name),
                                category: "Xcode".to_string(),
                                subcategory: "Archives".to_string(),
                                icon: "📦",
                                path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&archive.path()),
                                description: "App archive with dSYM. Keep if you need crash logs.",
                                safe_to_delete: SafetyLevel::Caution,
                                clean_command: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect iOS DeviceSupport
    fn detect_device_support(&self) -> Result<Vec<CleanableItem>> {
        let device_support_paths = [
            "Library/Developer/Xcode/iOS DeviceSupport",
            "Library/Developer/Xcode/watchOS DeviceSupport",
            "Library/Developer/Xcode/tvOS DeviceSupport",
        ];

        let mut items = Vec::new();

        for support_path in device_support_paths {
            let path = self.home.join(support_path);
            if !path.exists() {
                continue;
            }

            let platform = support_path.split('/').last().unwrap_or("Device");

            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if !entry_path.is_dir() {
                        continue;
                    }

                    let version = entry_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&entry_path)?;
                    if size == 0 {
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("{}: {}", platform.replace(" DeviceSupport", ""), version),
                        category: "Xcode".to_string(),
                        subcategory: "DeviceSupport".to_string(),
                        icon: "📱",
                        path: entry_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: "Debug symbols for iOS version. Safe to delete old versions.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Simulators
    fn detect_simulators(&self) -> Result<Vec<CleanableItem>> {
        let simulators = self.home
            .join("Library/Developer/CoreSimulator/Devices");

        if !simulators.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Each UUID directory is a simulator
        if let Ok(entries) = std::fs::read_dir(&simulators) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // Read device.plist to get simulator info
                let plist_path = path.join("device.plist");
                let name = if plist_path.exists() {
                    // Try to parse plist for name
                    std::fs::read_to_string(&plist_path)
                        .ok()
                        .and_then(|content| {
                            // Simple regex-like extraction for name
                            content.find("<key>name</key>")
                                .and_then(|idx| {
                                    let after = &content[idx..];
                                    after.find("<string>")
                                        .and_then(|start| {
                                            let s = &after[start + 8..];
                                            s.find("</string>").map(|end| s[..end].to_string())
                                        })
                                })
                        })
                        .unwrap_or_else(|| "Unknown Simulator".to_string())
                } else {
                    "Unknown Simulator".to_string()
                };

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 1_000_000 {
                    // Skip tiny simulators (likely just metadata)
                    continue;
                }

                items.push(CleanableItem {
                    name: format!("Simulator: {}", name),
                    category: "Xcode".to_string(),
                    subcategory: "Simulators".to_string(),
                    icon: "📲",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: get_mtime(&entry.path()),
                    description: "iOS Simulator with apps and data.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("xcrun simctl delete".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect Xcode caches
    fn detect_caches(&self) -> Result<Vec<CleanableItem>> {
        let cache_paths = [
            ("Xcode Cache", "Library/Caches/com.apple.dt.Xcode"),
            ("Instruments Cache", "Library/Caches/com.apple.dt.instruments"),
            ("Swift Package Cache", "Library/Caches/org.swift.swiftpm"),
            ("Playgrounds Cache", "Library/Developer/XCPGDevices"),
        ];

        let mut items = Vec::new();

        for (name, rel_path) in cache_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size == 0 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Xcode".to_string(),
                subcategory: "Caches".to_string(),
                icon: "🗂️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Xcode cache files. Safe to delete.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect documentation downloads
    fn detect_documentation(&self) -> Result<Vec<CleanableItem>> {
        let doc_path = self.home
            .join("Library/Developer/Shared/Documentation/DocSets");

        if !doc_path.exists() {
            return Ok(vec![]);
        }

        let (size, file_count) = calculate_dir_size(&doc_path)?;
        if size == 0 {
            return Ok(vec![]);
        }

        Ok(vec![CleanableItem {
            name: "Documentation Sets".to_string(),
            category: "Xcode".to_string(),
            subcategory: "Documentation".to_string(),
            icon: "📚",
            path: doc_path,
            size,
            file_count: Some(file_count),
            last_modified: None,
            description: "Offline documentation. Can be re-downloaded.",
            safe_to_delete: SafetyLevel::SafeWithCost,
            clean_command: None,
        }])
    }
}

impl Default for XcodeCleaner {
    fn default() -> Self {
        Self::new().expect("XcodeCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_xcode_cleaner_creation() {
        let cleaner = XcodeCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_xcode_detection() {
        if let Some(cleaner) = XcodeCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} Xcode items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
