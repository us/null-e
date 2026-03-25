//! Miscellaneous development tools cleanup module
//!
//! Handles cleanup of various development tools:
//! - Vagrant boxes
//! - Git LFS cache
//! - Go modules
//! - Ruby gems
//! - NuGet packages (.NET)
//! - Composer (PHP)

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::borrow::Cow;
use std::path::PathBuf;

/// Miscellaneous tools cleaner
pub struct MiscCleaner {
    home: PathBuf,
}

impl MiscCleaner {
    /// Create a new misc cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Vagrant
        items.extend(self.detect_vagrant()?);

        // Git LFS
        items.extend(self.detect_git_lfs()?);

        // Go
        items.extend(self.detect_go()?);

        // Ruby
        items.extend(self.detect_ruby()?);

        // NuGet (.NET)
        items.extend(self.detect_nuget()?);

        // Composer (PHP)
        items.extend(self.detect_composer()?);

        // Coursier (Scala)
        items.extend(self.detect_coursier()?);

        // Gradle (global)
        items.extend(self.detect_gradle()?);

        // Maven
        items.extend(self.detect_maven()?);

        // SBT (Scala)
        items.extend(self.detect_sbt()?);

        Ok(items)
    }

    /// Detect Vagrant boxes
    fn detect_vagrant(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let vagrant_home = std::env::var("VAGRANT_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".vagrant.d"));

        // Vagrant boxes
        let boxes_path = vagrant_home.join("boxes");
        if boxes_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&boxes_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let (size, file_count) = calculate_dir_size(&path)?;
                        if size < 100_000_000 {
                            // Skip small boxes
                            continue;
                        }

                        items.push(CleanableItem {
                            name: format!("Vagrant Box: {}", name.replace("-VAGRANTSLASH-", "/")),
                            category: "Vagrant".to_string(),
                            subcategory: "Boxes".to_string(),
                            icon: "📦",
                            path,
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&entry.path()),
                            description: Cow::Borrowed(
                                "Vagrant base box. Can be re-downloaded if needed.",
                            ),
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: Some(format!("vagrant box remove {}", name)),
                        });
                    }
                }
            }
        }

        // Vagrant temp files
        let tmp_path = vagrant_home.join("tmp");
        if tmp_path.exists() {
            let (size, file_count) = calculate_dir_size(&tmp_path)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "Vagrant Temp Files".to_string(),
                    category: "Vagrant".to_string(),
                    subcategory: "Cache".to_string(),
                    icon: "📦",
                    path: tmp_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed("Temporary Vagrant files. Safe to delete."),
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect Git LFS cache
    fn detect_git_lfs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let lfs_paths = [
            self.home.join(".git-lfs"),
            self.home.join("AppData/Local/git-lfs"), // Windows
        ];

        for lfs_path in lfs_paths {
            if !lfs_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&lfs_path)?;
            if size < 100_000_000 {
                // 100MB minimum
                continue;
            }

            items.push(CleanableItem {
                name: "Git LFS Cache".to_string(),
                category: "Git".to_string(),
                subcategory: "LFS".to_string(),
                icon: "📁",
                path: lfs_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(
                    "Git Large File Storage cache. Will be re-downloaded when needed.",
                ),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("git lfs prune".to_string()),
            });
        }

        Ok(items)
    }

    /// Detect Go module cache
    fn detect_go(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // GOPATH defaults to ~/go
        let gopath = std::env::var("GOPATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join("go"));

        // Module cache
        let mod_cache = gopath.join("pkg/mod/cache");
        if mod_cache.exists() {
            let (size, file_count) = calculate_dir_size(&mod_cache)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "Go Module Cache".to_string(),
                    category: "Go".to_string(),
                    subcategory: "Modules".to_string(),
                    icon: "🐹",
                    path: mod_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Downloaded Go modules. Will be re-downloaded when needed.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("go clean -modcache".to_string()),
                });
            }
        }

        // Build cache
        let gocache = std::env::var("GOCACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(target_os = "macos")]
                return self.home.join("Library/Caches/go-build");
                #[cfg(target_os = "linux")]
                return self.home.join(".cache/go-build");
                #[cfg(target_os = "windows")]
                return self.home.join("AppData/Local/go-build");
                #[cfg(not(any(
                    target_os = "macos",
                    target_os = "linux",
                    target_os = "windows"
                )))]
                return self.home.join(".cache/go-build");
            });

        if gocache.exists() {
            let (size, file_count) = calculate_dir_size(&gocache)?;
            if size > 500_000_000 {
                // 500MB
                items.push(CleanableItem {
                    name: "Go Build Cache".to_string(),
                    category: "Go".to_string(),
                    subcategory: "Build".to_string(),
                    icon: "🐹",
                    path: gocache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Go build cache. Will slow down first build after deletion.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("go clean -cache".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect Ruby gems
    fn detect_ruby(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Global gems
        let gem_paths = [
            self.home.join(".gem"),
            self.home.join(".local/share/gem"),  // Linux
            self.home.join("AppData/Local/gem"), // Windows
        ];

        for gem_path in gem_paths {
            if !gem_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&gem_path)?;
            if size < 100_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Ruby Gems".to_string(),
                category: "Ruby".to_string(),
                subcategory: "Gems".to_string(),
                icon: "💎",
                path: gem_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Installed Ruby gems. Will be reinstalled when needed."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("gem cleanup".to_string()),
            });
        }

        // Bundler cache
        let bundler_cache = self.home.join(".bundle/cache");
        if bundler_cache.exists() {
            let (size, file_count) = calculate_dir_size(&bundler_cache)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "Bundler Cache".to_string(),
                    category: "Ruby".to_string(),
                    subcategory: "Bundler".to_string(),
                    icon: "💎",
                    path: bundler_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed("Bundler download cache. Safe to delete."),
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some("bundle clean --force".to_string()),
                });
            }
        }

        // rbenv versions
        let rbenv_versions = self.home.join(".rbenv/versions");
        if rbenv_versions.exists() {
            if let Ok(entries) = std::fs::read_dir(&rbenv_versions) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let (size, file_count) = calculate_dir_size(&path)?;
                        if size < 100_000_000 {
                            continue;
                        }

                        items.push(CleanableItem {
                            name: format!("Ruby {}", name),
                            category: "Ruby".to_string(),
                            subcategory: "rbenv".to_string(),
                            icon: "💎",
                            path,
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&entry.path()),
                            description: Cow::Borrowed("Installed Ruby version via rbenv."),
                            safe_to_delete: SafetyLevel::Caution,
                            clean_command: Some(format!("rbenv uninstall {}", name)),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    /// Detect NuGet packages (.NET)
    fn detect_nuget(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // NuGet global packages
        let nuget_paths = [
            self.home.join(".nuget/packages"),
            self.home.join("AppData/Local/NuGet/Cache"), // Windows
        ];

        for nuget_path in nuget_paths {
            if !nuget_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&nuget_path)?;
            if size < 500_000_000 {
                // 500MB
                continue;
            }

            items.push(CleanableItem {
                name: "NuGet Global Packages".to_string(),
                category: ".NET".to_string(),
                subcategory: "NuGet".to_string(),
                icon: "🔷",
                path: nuget_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(
                    "NuGet package cache. Will be re-downloaded when needed.",
                ),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("dotnet nuget locals all --clear".to_string()),
            });
        }

        // .NET SDK workloads
        let workload_paths = [
            PathBuf::from("/usr/local/share/dotnet/metadata"), // macOS/Linux
            self.home.join("AppData/Local/Microsoft/dotnet"),  // Windows
        ];

        for workload_path in workload_paths {
            if !workload_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&workload_path)?;
            if size < 500_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: ".NET Workloads/SDK".to_string(),
                category: ".NET".to_string(),
                subcategory: "SDK".to_string(),
                icon: "🔷",
                path: workload_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(".NET SDK workloads. May affect installed SDKs."),
                safe_to_delete: SafetyLevel::Caution,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Composer (PHP) cache
    fn detect_composer(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let composer_paths = [
            self.home.join(".composer/cache"),
            self.home.join(".cache/composer"),              // Linux
            self.home.join("AppData/Local/Composer/cache"), // Windows
        ];

        for composer_path in composer_paths {
            if !composer_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&composer_path)?;
            if size < 100_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Composer Cache".to_string(),
                category: "PHP".to_string(),
                subcategory: "Composer".to_string(),
                icon: "🐘",
                path: composer_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(
                    "PHP Composer package cache. Will be re-downloaded when needed.",
                ),
                safe_to_delete: SafetyLevel::Safe,
                clean_command: Some("composer clear-cache".to_string()),
            });
        }

        Ok(items)
    }

    /// Detect Coursier (Scala) cache
    fn detect_coursier(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let coursier_paths = [
            self.home.join(".cache/coursier"),
            self.home.join("Library/Caches/Coursier"), // macOS
            self.home.join("AppData/Local/Coursier/Cache"), // Windows
        ];

        for coursier_path in coursier_paths {
            if !coursier_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&coursier_path)?;
            if size < 500_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Coursier Cache".to_string(),
                category: "Scala".to_string(),
                subcategory: "Coursier".to_string(),
                icon: "⚡",
                path: coursier_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(
                    "Scala dependency cache. Will be re-downloaded when needed.",
                ),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Gradle global cache
    fn detect_gradle(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let gradle_home = std::env::var("GRADLE_USER_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".gradle"));

        // Caches
        let cache_path = gradle_home.join("caches");
        if cache_path.exists() {
            let (size, file_count) = calculate_dir_size(&cache_path)?;
            if size > 1_000_000_000 {
                // 1GB
                items.push(CleanableItem {
                    name: "Gradle Cache".to_string(),
                    category: "Java/Kotlin".to_string(),
                    subcategory: "Gradle".to_string(),
                    icon: "🐘",
                    path: cache_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Gradle dependency cache. Will be re-downloaded when needed.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("gradle --stop && rm -rf ~/.gradle/caches".to_string()),
                });
            }
        }

        // Wrapper distributions
        let wrapper_path = gradle_home.join("wrapper/dists");
        if wrapper_path.exists() {
            let (size, file_count) = calculate_dir_size(&wrapper_path)?;
            if size > 500_000_000 {
                // 500MB
                items.push(CleanableItem {
                    name: "Gradle Wrapper Distributions".to_string(),
                    category: "Java/Kotlin".to_string(),
                    subcategory: "Gradle".to_string(),
                    icon: "🐘",
                    path: wrapper_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Downloaded Gradle distributions. Will be re-downloaded when needed.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        // Daemon files
        let daemon_path = gradle_home.join("daemon");
        if daemon_path.exists() {
            let (size, file_count) = calculate_dir_size(&daemon_path)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "Gradle Daemon Files".to_string(),
                    category: "Java/Kotlin".to_string(),
                    subcategory: "Gradle".to_string(),
                    icon: "🐘",
                    path: daemon_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed("Gradle daemon logs and state. Safe to delete."),
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some("gradle --stop".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect Maven cache
    fn detect_maven(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let m2_repo = self.home.join(".m2/repository");
        if m2_repo.exists() {
            let (size, file_count) = calculate_dir_size(&m2_repo)?;
            if size > 1_000_000_000 {
                // 1GB
                items.push(CleanableItem {
                    name: "Maven Repository".to_string(),
                    category: "Java/Kotlin".to_string(),
                    subcategory: "Maven".to_string(),
                    icon: "☕",
                    path: m2_repo,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Maven local repository. Dependencies will be re-downloaded.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("mvn dependency:purge-local-repository".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect SBT (Scala) cache
    fn detect_sbt(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let sbt_path = self.home.join(".sbt");
        if sbt_path.exists() {
            let (size, file_count) = calculate_dir_size(&sbt_path)?;
            if size > 500_000_000 {
                // 500MB
                items.push(CleanableItem {
                    name: "SBT Cache".to_string(),
                    category: "Scala".to_string(),
                    subcategory: "SBT".to_string(),
                    icon: "⚡",
                    path: sbt_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "SBT cache and plugins. Will slow down first build after deletion.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        // Ivy cache (used by SBT)
        let ivy_path = self.home.join(".ivy2/cache");
        if ivy_path.exists() {
            let (size, file_count) = calculate_dir_size(&ivy_path)?;
            if size > 500_000_000 {
                items.push(CleanableItem {
                    name: "Ivy Cache".to_string(),
                    category: "Scala".to_string(),
                    subcategory: "Ivy".to_string(),
                    icon: "⚡",
                    path: ivy_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Ivy dependency cache (used by SBT). Will be re-downloaded.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }
}
