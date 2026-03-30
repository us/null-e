//! System cleanup module
//!
//! Handles cleanup of system-level items:
//! - Trash/Recycle Bin
//! - Downloads folder (old archives)
//! - Temporary files
//! - Time Machine local snapshots (macOS)
//! - Windows temp files

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// System cleaner
pub struct SystemCleaner {
    home: PathBuf,
}

impl SystemCleaner {
    /// Create a new system cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Trash
        items.extend(self.detect_trash()?);

        // Downloads (old archives)
        items.extend(self.detect_downloads()?);

        // Temp files
        items.extend(self.detect_temp()?);

        // Time Machine local snapshots (macOS)
        #[cfg(target_os = "macos")]
        items.extend(self.detect_time_machine()?);

        // System caches
        items.extend(self.detect_system_caches()?);

        Ok(items)
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_child_dirs(
        &self,
        root: &std::path::Path,
        label_prefix: &str,
        subcategory: &str,
        icon: &'static str,
        description: &'static str,
        safe_to_delete: SafetyLevel,
        min_size: u64,
    ) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        if !root.exists() {
            return Ok(items);
        }

        let Ok(entries) = std::fs::read_dir(root) else {
            return Ok(items);
        };

        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if !path.is_dir() || std::fs::read_dir(&path).is_err() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < min_size {
                continue;
            }

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            items.push(CleanableItem {
                name: format!("{label_prefix} ({name})"),
                category: "System".to_string(),
                subcategory: subcategory.to_string(),
                icon,
                path: path.clone(),
                size,
                file_count: Some(file_count),
                last_modified: get_mtime(&path),
                description: Cow::Borrowed(description),
                safe_to_delete,
                clean_command: None,
            });
        }

        Ok(items)
    }

    #[cfg(target_os = "macos")]
    fn macos_var_folders_dir(name: &str) -> Option<PathBuf> {
        let mut dir = std::env::temp_dir();
        if dir.file_name()?.to_str()? != "T" {
            return None;
        }
        dir.pop();
        dir.push(name);
        Some(dir)
    }

    /// Detect Trash contents
    fn detect_trash(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Trash locations per OS
        #[cfg(target_os = "macos")]
        let trash_paths = vec![self.home.join(".Trash")];

        #[cfg(target_os = "linux")]
        let trash_paths = vec![
            self.home.join(".local/share/Trash/files"),
            self.home.join(".Trash"),
        ];

        #[cfg(target_os = "windows")]
        let trash_paths = vec![
            // Windows Recycle Bin is more complex to access
            // Using SHQueryRecycleBin API would be better
            PathBuf::from("C:\\$Recycle.Bin"),
        ];

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let trash_paths: Vec<PathBuf> = vec![];

        for trash_path in trash_paths {
            if !trash_path.exists() {
                continue;
            }

            // Try to calculate size, skip if permission denied
            let (size, file_count) = match calculate_dir_size(&trash_path) {
                Ok(result) => result,
                Err(_) => continue, // Permission denied or other error
            };

            // Show if at least 1MB
            if size < 1_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Trash".to_string(),
                category: "System".to_string(),
                subcategory: "Trash".to_string(),
                icon: "🗑️",
                path: trash_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Files in trash. Permanently deleted when cleaned."),
                safe_to_delete: SafetyLevel::Caution,
                #[cfg(target_os = "macos")]
                clean_command: Some("rm -rf ~/.Trash/*".to_string()),
                #[cfg(target_os = "linux")]
                clean_command: Some("trash-empty".to_string()),
                #[cfg(target_os = "windows")]
                clean_command: Some("Clear-RecycleBin -Force".to_string()),
                #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect old archives in Downloads
    fn detect_downloads(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let downloads = dirs::download_dir().unwrap_or_else(|| self.home.join("Downloads"));

        if !downloads.exists() {
            return Ok(items);
        }

        // Archive extensions to look for
        let archive_extensions = [
            "zip", "tar", "tar.gz", "tgz", "tar.bz2", "tar.xz", "7z", "rar", "dmg", "iso", "pkg",
            "deb", "rpm", "msi", "exe",
        ];

        // Threshold: files older than 30 days
        let age_threshold = Duration::from_secs(30 * 24 * 60 * 60);
        let now = SystemTime::now();

        let mut total_size = 0u64;
        let mut file_count = 0u64;
        let mut old_archives: Vec<PathBuf> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&downloads) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                // Check extension
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // Handle .tar.* extensions
                let is_archive = archive_extensions.contains(&ext.as_str())
                    || path.to_string_lossy().ends_with(".tar.gz")
                    || path.to_string_lossy().ends_with(".tar.bz2")
                    || path.to_string_lossy().ends_with(".tar.xz");

                if !is_archive {
                    continue;
                }

                // Check age
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > age_threshold {
                                total_size += metadata.len();
                                file_count += 1;
                                old_archives.push(path);
                            }
                        }
                    }
                }
            }
        }

        if total_size > 100_000_000 && file_count > 0 {
            // 100MB minimum
            items.push(CleanableItem {
                name: format!("Old Downloads ({} files)", file_count),
                category: "System".to_string(),
                subcategory: "Downloads".to_string(),
                icon: "📥",
                path: downloads,
                size: total_size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Archive files older than 30 days in Downloads folder."),
                safe_to_delete: SafetyLevel::Caution,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect temporary files
    fn detect_temp(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Temp locations
        #[cfg(target_os = "macos")]
        {
            items.extend(self.detect_macos_child_dirs(
                &self.home.join("Library/Caches/TemporaryItems"),
                "Temp Files",
                "Temp",
                "🔥",
                "Temporary files. May contain files in use.",
                SafetyLevel::Caution,
                100_000_000,
            )?);
            items.extend(self.detect_macos_child_dirs(
                &std::env::temp_dir(),
                "Temp Files",
                "Temp",
                "🔥",
                "Temporary files. May contain files in use.",
                SafetyLevel::Caution,
                100_000_000,
            )?);

            return Ok(items);
        }

        #[cfg(target_os = "linux")]
        let temp_paths = vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/var/tmp"),
            self.home.join(".cache"),
        ];

        #[cfg(target_os = "windows")]
        let temp_paths = vec![std::env::temp_dir(), self.home.join("AppData/Local/Temp")];

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let temp_paths: Vec<PathBuf> = vec![];

        for temp_path in temp_paths {
            if !temp_path.exists() {
                continue;
            }

            // Skip if not readable (permissions)
            if std::fs::read_dir(&temp_path).is_err() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&temp_path)?;
            if size < 500_000_000 {
                // 500MB minimum for temp
                continue;
            }

            // Don't suggest cleaning main system cache on Linux
            #[cfg(target_os = "linux")]
            if temp_path == self.home.join(".cache") {
                continue;
            }

            items.push(CleanableItem {
                name: format!(
                    "Temp Files ({})",
                    temp_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "tmp".to_string())
                ),
                category: "System".to_string(),
                subcategory: "Temp".to_string(),
                icon: "🔥",
                path: temp_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Temporary files. May contain files in use."),
                safe_to_delete: SafetyLevel::Caution,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Time Machine local snapshots (macOS only)
    #[cfg(target_os = "macos")]
    fn detect_time_machine(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Check if tmutil is available and get snapshot info
        let output = std::process::Command::new("tmutil")
            .args(["listlocalsnapshotdates", "/"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let snapshot_count = stdout.lines().count().saturating_sub(1); // First line is header

                if snapshot_count > 0 {
                    // Estimate size (typically 1-10GB per snapshot)
                    // We can't easily get exact size without parsing more output
                    let estimated_size = (snapshot_count as u64) * 2_000_000_000; // ~2GB per snapshot

                    items.push(CleanableItem {
                        name: format!("Time Machine Snapshots ({} snapshots)", snapshot_count),
                        category: "System".to_string(),
                        subcategory: "Time Machine".to_string(),
                        icon: "⏰",
                        path: PathBuf::from("/"),
                        size: estimated_size,
                        file_count: Some(snapshot_count as u64),
                        last_modified: None,
                        description: Cow::Borrowed(
                            "Local Time Machine snapshots. Deleting frees space.",
                        ),
                        safe_to_delete: SafetyLevel::Caution,
                        clean_command: Some("tmutil deletelocalsnapshots /".to_string()),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect system caches
    fn detect_system_caches(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // User caches
            let user_cache = self.home.join("Library/Caches");
            if user_cache.exists() {
                let (size, file_count) = calculate_dir_size(&user_cache)?;
                if size > 1_000_000_000 {
                    // 1GB
                    items.push(CleanableItem {
                        name: "User Caches".to_string(),
                        category: "System".to_string(),
                        subcategory: "Caches".to_string(),
                        icon: "🗄️",
                        path: user_cache,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: Cow::Borrowed("Application caches. Apps will rebuild them."),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }

            if let Some(var_folders_cache) = Self::macos_var_folders_dir("C") {
                items.extend(self.detect_macos_child_dirs(
                    &var_folders_cache,
                    "System Cache",
                    "Caches",
                    "🗄️",
                    "Per-user macOS cache. Apps may recreate it.",
                    SafetyLevel::SafeWithCost,
                    100_000_000,
                )?);
            }

            // Font caches
            let font_caches = [
                self.home.join("Library/Caches/com.apple.FontRegistry"),
                PathBuf::from("/System/Library/Caches/com.apple.IntlDataCache.le*"),
            ];

            for font_cache in font_caches {
                if !font_cache.exists() {
                    continue;
                }
                let (size, file_count) = calculate_dir_size(&font_cache)?;
                if size > 50_000_000 {
                    items.push(CleanableItem {
                        name: "Font Caches".to_string(),
                        category: "System".to_string(),
                        subcategory: "Fonts".to_string(),
                        icon: "🔤",
                        path: font_cache,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: Cow::Borrowed("Font caches. System will rebuild on restart."),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: Some("sudo atsutil databases -remove".to_string()),
                    });
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Thumbnail cache
            let thumbs = self.home.join(".cache/thumbnails");
            if thumbs.exists() {
                let (size, file_count) = calculate_dir_size(&thumbs)?;
                if size > 500_000_000 {
                    items.push(CleanableItem {
                        name: "Thumbnail Cache".to_string(),
                        category: "System".to_string(),
                        subcategory: "Thumbnails".to_string(),
                        icon: "🖼️",
                        path: thumbs,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: Cow::Borrowed(
                            "Image thumbnails. Will be regenerated when needed.",
                        ),
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: None,
                    });
                }
            }

            // Journal logs
            let journal = PathBuf::from("/var/log/journal");
            if journal.exists() {
                if let Ok((size, file_count)) = calculate_dir_size(&journal) {
                    if size > 1_000_000_000 {
                        // 1GB
                        items.push(CleanableItem {
                            name: "Journal Logs".to_string(),
                            category: "System".to_string(),
                            subcategory: "Logs".to_string(),
                            icon: "📋",
                            path: journal,
                            size,
                            file_count: Some(file_count),
                            last_modified: None,
                            description: Cow::Borrowed("Systemd journal logs. Can be vacuumed."),
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: Some("sudo journalctl --vacuum-size=500M".to_string()),
                        });
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows Update cache
            let wu_cache = PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download");
            if wu_cache.exists() {
                if let Ok((size, file_count)) = calculate_dir_size(&wu_cache) {
                    if size > 1_000_000_000 {
                        items.push(CleanableItem {
                            name: "Windows Update Cache".to_string(),
                            category: "System".to_string(),
                            subcategory: "Windows Update".to_string(),
                            icon: "🪟",
                            path: wu_cache,
                            size,
                            file_count: Some(file_count),
                            last_modified: None,
                            description: Cow::Borrowed("Windows Update download cache."),
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: None,
                        });
                    }
                }
            }

            // Prefetch
            let prefetch = PathBuf::from("C:\\Windows\\Prefetch");
            if prefetch.exists() {
                if let Ok((size, file_count)) = calculate_dir_size(&prefetch) {
                    if size > 200_000_000 {
                        items.push(CleanableItem {
                            name: "Prefetch Files".to_string(),
                            category: "System".to_string(),
                            subcategory: "Prefetch".to_string(),
                            icon: "⚡",
                            path: prefetch,
                            size,
                            file_count: Some(file_count),
                            last_modified: None,
                            description: Cow::Borrowed(
                                "Windows prefetch data. May slow first app launches.",
                            ),
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: None,
                        });
                    }
                }
            }
        }

        Ok(items)
    }
}

/// Find big files in a directory
pub fn find_big_files(min_size_mb: u64) -> Result<Vec<CleanableItem>> {
    let mut items = Vec::new();
    let min_size = min_size_mb * 1_000_000;

    let home = dirs::home_dir()
        .ok_or_else(|| crate::error::NullEError::Config("Could not find home directory".into()))?;

    // Walk home directory, but skip certain paths
    let skip_paths = [
        ".git",
        "node_modules",
        "target",
        ".cargo",
        ".npm",
        ".gradle",
        "venv",
        ".venv",
        "__pycache__",
        "Library/Caches",
        "AppData/Local",
        ".cache",
    ];

    let walker = walkdir::WalkDir::new(&home)
        .max_depth(5) // Don't go too deep
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            // Skip hidden system directories and known cache locations
            !skip_paths
                .iter()
                .any(|skip| path.to_string_lossy().contains(skip))
        });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Ok(metadata) = path.metadata() {
            let size = metadata.len();
            if size >= min_size {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                items.push(CleanableItem {
                    name,
                    category: "Big Files".to_string(),
                    subcategory: "Files".to_string(),
                    icon: "📄",
                    path: path.to_path_buf(),
                    size,
                    file_count: Some(1),
                    last_modified: get_mtime(path),
                    description: Cow::Borrowed("Large file found in home directory."),
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: None,
                });
            }
        }
    }

    // Sort by size (largest first)
    items.sort_by(|a, b| b.size.cmp(&a.size));

    // Return top 50
    items.truncate(50);

    Ok(items)
}
