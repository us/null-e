//! Android Studio cleanup module
//!
//! Handles cleanup of Android development files:
//! - AVD (Android Virtual Devices / Emulators)
//! - SDK components
//! - Gradle caches
//! - Android Studio caches

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Android cleaner
pub struct AndroidCleaner {
    home: PathBuf,
}

impl AndroidCleaner {
    /// Create a new Android cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all Android cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // AVD Emulators
        items.extend(self.detect_avd()?);

        // SDK System Images
        items.extend(self.detect_system_images()?);

        // Gradle caches (Android-specific)
        items.extend(self.detect_gradle_caches()?);

        // Android build caches
        items.extend(self.detect_android_caches()?);

        // Android Studio caches
        items.extend(self.detect_studio_caches()?);

        Ok(items)
    }

    /// Detect AVD (Android Virtual Devices)
    fn detect_avd(&self) -> Result<Vec<CleanableItem>> {
        let avd_path = self.home.join(".android/avd");

        if !avd_path.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&avd_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                // AVD directories end with .avd
                if path.is_dir() && path.extension().map(|e| e == "avd").unwrap_or(false) {
                    let name = path.file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size == 0 {
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("Emulator: {}", name),
                        category: "Android".to_string(),
                        subcategory: "AVD".to_string(),
                        icon: "🤖",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: "Android Virtual Device with user data.",
                        safe_to_delete: SafetyLevel::Caution,
                        clean_command: Some(format!("avdmanager delete avd -n {}", name)),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect SDK System Images
    fn detect_system_images(&self) -> Result<Vec<CleanableItem>> {
        // Try common SDK locations
        let sdk_paths = [
            self.home.join("Library/Android/sdk"),  // macOS default
            self.home.join("Android/Sdk"),          // Linux default
            self.home.join(".android/sdk"),         // Alternative
        ];

        let mut items = Vec::new();

        for sdk_path in sdk_paths {
            let sys_images = sdk_path.join("system-images");
            if !sys_images.exists() {
                continue;
            }

            // system-images/<android-version>/<variant>/<arch>
            if let Ok(versions) = std::fs::read_dir(&sys_images) {
                for version in versions.filter_map(|e| e.ok()) {
                    let version_path = version.path();
                    if !version_path.is_dir() {
                        continue;
                    }

                    let version_name = version_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if let Ok(variants) = std::fs::read_dir(&version_path) {
                        for variant in variants.filter_map(|e| e.ok()) {
                            let variant_path = variant.path();
                            if !variant_path.is_dir() {
                                continue;
                            }

                            let variant_name = variant_path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();

                            let (size, file_count) = calculate_dir_size(&variant_path)?;
                            if size == 0 {
                                continue;
                            }

                            items.push(CleanableItem {
                                name: format!("System Image: {} {}", version_name, variant_name),
                                category: "Android".to_string(),
                                subcategory: "SDK".to_string(),
                                icon: "💿",
                                path: variant_path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&variant.path()),
                                description: "Android system image for emulator.",
                                safe_to_delete: SafetyLevel::SafeWithCost,
                                clean_command: Some("sdkmanager --uninstall".to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect Gradle caches
    fn detect_gradle_caches(&self) -> Result<Vec<CleanableItem>> {
        let gradle_paths = [
            ("Gradle Caches", ".gradle/caches"),
            ("Gradle Wrapper", ".gradle/wrapper"),
            ("Gradle Daemon Logs", ".gradle/daemon"),
            ("Gradle Native", ".gradle/native"),
        ];

        let mut items = Vec::new();

        for (name, rel_path) in gradle_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 10_000_000 {
                // Skip if less than 10MB
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Android".to_string(),
                subcategory: "Gradle".to_string(),
                icon: "🐘",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Gradle build cache. Will be rebuilt on next build.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Android build caches
    fn detect_android_caches(&self) -> Result<Vec<CleanableItem>> {
        let cache_paths = [
            ("Android Cache", ".android/cache"),
            ("Android Build Cache", ".android/build-cache"),
            ("ADB Keys", ".android/.adb_keys_backup"),
        ];

        let mut items = Vec::new();

        for (name, rel_path) in cache_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = if path.is_dir() {
                calculate_dir_size(&path)?
            } else {
                let meta = std::fs::metadata(&path)?;
                (meta.len(), 1)
            };

            if size == 0 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Android".to_string(),
                subcategory: "Cache".to_string(),
                icon: "🗂️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Android build cache files.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Android Studio caches
    fn detect_studio_caches(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Android Studio versions (macOS)
        #[cfg(target_os = "macos")]
        {
            let cache_base = self.home.join("Library/Caches");
            if cache_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&cache_base) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("Google.AndroidStudio") {
                            let path = entry.path();
                            let (size, file_count) = calculate_dir_size(&path)?;
                            if size == 0 {
                                continue;
                            }

                            items.push(CleanableItem {
                                name: format!("Android Studio Cache: {}", name.replace("Google.", "")),
                                category: "Android".to_string(),
                                subcategory: "IDE Cache".to_string(),
                                icon: "💻",
                                path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&entry.path()),
                                description: "Android Studio IDE cache.",
                                safe_to_delete: SafetyLevel::Safe,
                                clean_command: None,
                            });
                        }
                    }
                }
            }

            // Application Support
            let support_base = self.home.join("Library/Application Support");
            if support_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&support_base) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("Google") && name.contains("AndroidStudio") {
                            let path = entry.path();
                            let (size, file_count) = calculate_dir_size(&path)?;
                            if size < 100_000_000 {
                                // Skip if less than 100MB
                                continue;
                            }

                            items.push(CleanableItem {
                                name: format!("Android Studio Data: {}", name.replace("Google/", "")),
                                category: "Android".to_string(),
                                subcategory: "IDE Data".to_string(),
                                icon: "💻",
                                path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&entry.path()),
                                description: "Android Studio settings and plugins.",
                                safe_to_delete: SafetyLevel::Caution,
                                clean_command: None,
                            });
                        }
                    }
                }
            }
        }

        // Linux locations
        #[cfg(target_os = "linux")]
        {
            let config_base = self.home.join(".config");
            if config_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&config_base) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("Google") && name.contains("AndroidStudio") {
                            let path = entry.path();
                            let (size, file_count) = calculate_dir_size(&path)?;
                            if size == 0 {
                                continue;
                            }

                            items.push(CleanableItem {
                                name: format!("Android Studio: {}", name),
                                category: "Android".to_string(),
                                subcategory: "IDE".to_string(),
                                icon: "💻",
                                path,
                                size,
                                file_count: Some(file_count),
                                last_modified: get_mtime(&entry.path()),
                                description: "Android Studio configuration.",
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
}

impl Default for AndroidCleaner {
    fn default() -> Self {
        Self::new().expect("AndroidCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android_cleaner_creation() {
        let cleaner = AndroidCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_android_detection() {
        if let Some(cleaner) = AndroidCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} Android items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
