//! Global developer cache detection and cleaning
//!
//! This module handles system-wide package manager caches like ~/.npm, ~/.cargo/registry, etc.
//! These are separate from project-specific artifacts (node_modules, target).

use crate::error::{DevSweepError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// A global developer cache location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCache {
    /// Human-readable name
    pub name: String,
    /// Short identifier
    pub id: &'static str,
    /// Icon/emoji for display
    pub icon: &'static str,
    /// Full path to the cache directory
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Number of files
    pub file_count: u64,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Official clean command (if available)
    pub clean_command: Option<&'static str>,
    /// Description of what this cache contains
    pub description: &'static str,
}

impl GlobalCache {
    /// Check if this cache exists
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    /// Get age in days since last modification
    pub fn age_days(&self) -> Option<u64> {
        self.last_modified
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_secs() / 86400)
    }

    /// Format the last used time
    pub fn last_used_display(&self) -> String {
        match self.age_days() {
            Some(0) => "today".to_string(),
            Some(1) => "yesterday".to_string(),
            Some(d) if d < 7 => format!("{} days ago", d),
            Some(d) if d < 30 => format!("{} weeks ago", d / 7),
            Some(d) if d < 365 => format!("{} months ago", d / 30),
            Some(d) => format!("{} years ago", d / 365),
            None => "unknown".to_string(),
        }
    }
}

/// Definition of a known cache location
#[derive(Debug, Clone)]
pub struct CacheDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    /// Paths relative to home directory
    pub paths: &'static [&'static str],
    /// Official clean command
    pub clean_command: Option<&'static str>,
    pub description: &'static str,
}

/// All known global cache locations
pub fn known_caches() -> Vec<CacheDefinition> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // JavaScript/Node.js Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "npm",
            name: "npm cache",
            icon: "📦",
            paths: &[".npm/_cacache"],
            clean_command: Some("npm cache clean --force"),
            description: "Cached npm packages and metadata",
        },
        CacheDefinition {
            id: "yarn",
            name: "Yarn cache",
            icon: "🧶",
            paths: &[".yarn/cache", ".cache/yarn"],
            clean_command: Some("yarn cache clean"),
            description: "Cached Yarn packages",
        },
        CacheDefinition {
            id: "pnpm",
            name: "pnpm store",
            icon: "📦",
            paths: &[".pnpm-store", ".local/share/pnpm/store"],
            clean_command: Some("pnpm store prune"),
            description: "Global pnpm content-addressable store",
        },
        CacheDefinition {
            id: "bun",
            name: "Bun cache",
            icon: "🥟",
            paths: &[".bun/install/cache"],
            clean_command: None,
            description: "Cached Bun packages",
        },
        CacheDefinition {
            id: "deno",
            name: "Deno cache",
            icon: "🦕",
            paths: &[".cache/deno", ".deno"],
            clean_command: Some("deno cache --reload"),
            description: "Cached Deno modules and compiled scripts",
        },

        // ═══════════════════════════════════════════════════════════════
        // Python Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "pip",
            name: "pip cache",
            icon: "🐍",
            paths: &[".cache/pip", "Library/Caches/pip"],
            clean_command: Some("pip cache purge"),
            description: "Cached pip wheels and HTTP responses",
        },
        CacheDefinition {
            id: "uv",
            name: "uv cache",
            icon: "⚡",
            paths: &[".cache/uv"],
            clean_command: Some("uv cache clean"),
            description: "Cached uv packages (fast Python installer)",
        },
        CacheDefinition {
            id: "poetry",
            name: "Poetry cache",
            icon: "📜",
            paths: &[".cache/pypoetry", "Library/Caches/pypoetry"],
            clean_command: Some("poetry cache clear --all ."),
            description: "Cached Poetry packages and virtualenvs",
        },
        CacheDefinition {
            id: "pipenv",
            name: "Pipenv cache",
            icon: "🐍",
            paths: &[".cache/pipenv"],
            clean_command: None,
            description: "Cached Pipenv packages",
        },
        CacheDefinition {
            id: "conda",
            name: "Conda cache",
            icon: "🐍",
            paths: &[".conda/pkgs", "anaconda3/pkgs", "miniconda3/pkgs"],
            clean_command: Some("conda clean --all"),
            description: "Cached Conda packages",
        },

        // ═══════════════════════════════════════════════════════════════
        // Rust Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "cargo-registry",
            name: "Cargo registry",
            icon: "🦀",
            paths: &[".cargo/registry"],
            clean_command: None, // Cargo 1.75+ has auto GC
            description: "Downloaded crate sources and indices",
        },
        CacheDefinition {
            id: "cargo-git",
            name: "Cargo git",
            icon: "🦀",
            paths: &[".cargo/git"],
            clean_command: None,
            description: "Git dependencies cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // Go Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "go-mod",
            name: "Go modules",
            icon: "🐹",
            paths: &["go/pkg/mod"],
            clean_command: Some("go clean -modcache"),
            description: "Downloaded Go module cache",
        },
        CacheDefinition {
            id: "go-build",
            name: "Go build cache",
            icon: "🐹",
            paths: &[".cache/go-build", "Library/Caches/go-build"],
            clean_command: Some("go clean -cache"),
            description: "Go build artifacts cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // JVM Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "gradle",
            name: "Gradle cache",
            icon: "🐘",
            paths: &[".gradle/caches"],
            clean_command: None, // Manual or gradle --stop && rm
            description: "Gradle dependencies and build cache",
        },
        CacheDefinition {
            id: "maven",
            name: "Maven repository",
            icon: "🪶",
            paths: &[".m2/repository"],
            clean_command: None,
            description: "Maven local repository",
        },
        CacheDefinition {
            id: "sbt",
            name: "SBT cache",
            icon: "📦",
            paths: &[".sbt", ".ivy2/cache"],
            clean_command: None,
            description: "SBT and Ivy dependency cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // .NET Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "nuget",
            name: "NuGet cache",
            icon: "🔷",
            paths: &[".nuget/packages"],
            clean_command: Some("dotnet nuget locals all --clear"),
            description: "NuGet package cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // Ruby Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "gem",
            name: "Ruby gems",
            icon: "💎",
            paths: &[".gem", ".local/share/gem"],
            clean_command: Some("gem cleanup"),
            description: "Installed Ruby gems",
        },
        CacheDefinition {
            id: "bundler",
            name: "Bundler cache",
            icon: "💎",
            paths: &[".bundle/cache"],
            clean_command: Some("bundle clean --force"),
            description: "Bundler gem cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // PHP Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "composer",
            name: "Composer cache",
            icon: "🎼",
            paths: &[".composer/cache", ".cache/composer"],
            clean_command: Some("composer clear-cache"),
            description: "Composer package cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // Mobile Development
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "cocoapods",
            name: "CocoaPods cache",
            icon: "🍫",
            paths: &["Library/Caches/CocoaPods"],
            clean_command: Some("pod cache clean --all"),
            description: "CocoaPods specs and pod cache",
        },
        CacheDefinition {
            id: "pub",
            name: "Dart/Flutter pub",
            icon: "🎯",
            paths: &[".pub-cache"],
            clean_command: None,
            description: "Dart and Flutter package cache",
        },
        CacheDefinition {
            id: "android-gradle",
            name: "Android Gradle",
            icon: "🤖",
            paths: &[".android/cache", ".android/build-cache"],
            clean_command: None,
            description: "Android build cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // ML/AI Ecosystem
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "huggingface",
            name: "Hugging Face cache",
            icon: "🤗",
            paths: &[".cache/huggingface"],
            clean_command: None,
            description: "Downloaded ML models and datasets",
        },
        CacheDefinition {
            id: "torch",
            name: "PyTorch cache",
            icon: "🔥",
            paths: &[".cache/torch"],
            clean_command: None,
            description: "PyTorch model hub cache",
        },

        // ═══════════════════════════════════════════════════════════════
        // Other Tools
        // ═══════════════════════════════════════════════════════════════
        CacheDefinition {
            id: "homebrew",
            name: "Homebrew cache",
            icon: "🍺",
            paths: &["Library/Caches/Homebrew"],
            clean_command: Some("brew cleanup --prune=all"),
            description: "Downloaded Homebrew bottles and source",
        },
        CacheDefinition {
            id: "cypress",
            name: "Cypress cache",
            icon: "🌲",
            paths: &[".cache/Cypress", "Library/Caches/Cypress"],
            clean_command: Some("cypress cache clear"),
            description: "Cypress browser binaries",
        },
        CacheDefinition {
            id: "playwright",
            name: "Playwright cache",
            icon: "🎭",
            paths: &[".cache/ms-playwright", "Library/Caches/ms-playwright"],
            clean_command: None,
            description: "Playwright browser binaries",
        },
        CacheDefinition {
            id: "electron",
            name: "Electron cache",
            icon: "⚛️",
            paths: &[".cache/electron", "Library/Caches/electron"],
            clean_command: None,
            description: "Electron framework binaries",
        },
    ]
}

/// Detect all existing global caches
pub fn detect_caches() -> Result<Vec<GlobalCache>> {
    let home = dirs::home_dir()
        .ok_or_else(|| DevSweepError::Config("Could not find home directory".into()))?;

    let definitions = known_caches();
    let mut caches = Vec::new();

    for def in definitions {
        // Try each possible path for this cache
        for rel_path in def.paths {
            let full_path = home.join(rel_path);

            if full_path.exists() && full_path.is_dir() {
                // Found this cache!
                let mut cache = GlobalCache {
                    name: def.name.to_string(),
                    id: def.id,
                    icon: def.icon,
                    path: full_path.clone(),
                    size: 0,
                    file_count: 0,
                    last_modified: None,
                    clean_command: def.clean_command,
                    description: def.description,
                };

                // Get last modified time
                if let Ok(meta) = std::fs::metadata(&full_path) {
                    cache.last_modified = meta.modified().ok();
                }

                caches.push(cache);
                break; // Found one path, don't check others
            }
        }
    }

    Ok(caches)
}

/// Calculate size for a single cache (can be slow for large caches)
pub fn calculate_cache_size(cache: &mut GlobalCache) -> Result<()> {
    use rayon::prelude::*;
    use walkdir::WalkDir;

    if !cache.path.exists() {
        return Ok(());
    }

    let entries: Vec<_> = WalkDir::new(&cache.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let (size, count): (u64, u64) = entries
        .par_iter()
        .filter_map(|entry| entry.metadata().ok())
        .filter(|m| m.is_file())
        .fold(
            || (0u64, 0u64),
            |(size, count), m| (size + m.len(), count + 1),
        )
        .reduce(|| (0, 0), |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2));

    cache.size = size;
    cache.file_count = count;

    Ok(())
}

/// Calculate sizes for all caches in parallel
pub fn calculate_all_sizes(caches: &mut [GlobalCache]) -> Result<()> {
    use rayon::prelude::*;

    // Calculate sizes in parallel
    caches.par_iter_mut().for_each(|cache| {
        let _ = calculate_cache_size(cache);
    });

    Ok(())
}

/// Clean a cache using the official command if available, otherwise rm -rf
pub fn clean_cache(cache: &GlobalCache, use_official_command: bool) -> Result<CleanResult> {
    if !cache.path.exists() {
        return Ok(CleanResult {
            success: true,
            bytes_freed: 0,
            method: CleanMethod::NotFound,
        });
    }

    let size_before = cache.size;

    // Try official command first if requested
    if use_official_command {
        if let Some(cmd) = cache.clean_command {
            let result = run_clean_command(cmd);
            if result.is_ok() {
                return Ok(CleanResult {
                    success: true,
                    bytes_freed: size_before,
                    method: CleanMethod::OfficialCommand(cmd.to_string()),
                });
            }
            // Fall through to manual deletion if command fails
        }
    }

    // Manual deletion
    match crate::trash::delete_path(&cache.path, crate::trash::DeleteMethod::Permanent) {
        Ok(_) => Ok(CleanResult {
            success: true,
            bytes_freed: size_before,
            method: CleanMethod::ManualDelete,
        }),
        Err(e) => Err(e),
    }
}

/// Run an official clean command
fn run_clean_command(cmd: &str) -> Result<()> {
    use std::process::Command;

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(DevSweepError::Config("Empty clean command".into()));
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map_err(|e| DevSweepError::Io(e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(DevSweepError::CleanFailed {
            path: PathBuf::from(cmd),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

/// Result of cleaning a cache
#[derive(Debug)]
pub struct CleanResult {
    pub success: bool,
    pub bytes_freed: u64,
    pub method: CleanMethod,
}

/// How a cache was cleaned
#[derive(Debug)]
pub enum CleanMethod {
    OfficialCommand(String),
    ManualDelete,
    NotFound,
}

/// Summary of all cache operations
#[derive(Debug, Default)]
pub struct CachesSummary {
    pub total_caches: usize,
    pub total_size: u64,
    pub total_files: u64,
    pub cleaned_count: usize,
    pub bytes_freed: u64,
}

impl CachesSummary {
    pub fn from_caches(caches: &[GlobalCache]) -> Self {
        Self {
            total_caches: caches.len(),
            total_size: caches.iter().map(|c| c.size).sum(),
            total_files: caches.iter().map(|c| c.file_count).sum(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_caches_not_empty() {
        let caches = known_caches();
        assert!(!caches.is_empty());
        assert!(caches.len() > 20); // We defined 25+ caches
    }

    #[test]
    fn test_cache_age_display() {
        let mut cache = GlobalCache {
            name: "test".into(),
            id: "test",
            icon: "📦",
            path: PathBuf::from("/tmp/test"),
            size: 0,
            file_count: 0,
            last_modified: Some(SystemTime::now()),
            clean_command: None,
            description: "test",
        };

        assert_eq!(cache.last_used_display(), "today");
    }

    #[test]
    fn test_detect_caches() {
        // This will detect real caches on the system
        let caches = detect_caches().unwrap();
        // At minimum, most dev machines have at least one cache
        // But don't assert > 0 as CI might not have any
        println!("Found {} caches", caches.len());
    }
}
