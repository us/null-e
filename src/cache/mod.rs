//! Scan result caching for faster subsequent scans
//!
//! This module provides intelligent caching of scan results with mtime-based invalidation.
//! When a directory's modification time hasn't changed, we can skip rescanning it.

use crate::core::Project;
use crate::error::{DevSweepError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cache entry for a single project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProject {
    /// The project data
    pub project: Project,
    /// Modification time of the project root when cached
    pub root_mtime: u64,
    /// When this entry was cached
    pub cached_at: u64,
}

/// Cache entry for a scanned directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDirectory {
    /// Modification time when last scanned
    pub mtime: u64,
    /// Project IDs found in this directory (empty if no project)
    pub project_roots: Vec<PathBuf>,
}

/// The scan cache
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanCache {
    /// Version for cache invalidation on format changes
    pub version: u32,
    /// When the cache was last updated
    pub updated_at: u64,
    /// Cached projects by their root path
    pub projects: HashMap<PathBuf, CachedProject>,
    /// Cached directory scan info
    pub directories: HashMap<PathBuf, CachedDirectory>,
}

impl ScanCache {
    /// Current cache version - bump this when format changes
    pub const VERSION: u32 = 1;

    /// Cache TTL - invalidate after 24 hours regardless
    pub const TTL_SECS: u64 = 24 * 60 * 60;

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            updated_at: current_timestamp(),
            projects: HashMap::new(),
            directories: HashMap::new(),
        }
    }

    /// Check if a project is still valid in cache
    pub fn get_valid_project(&self, root: &Path) -> Option<&CachedProject> {
        let cached = self.projects.get(root)?;

        // Check if mtime has changed
        let current_mtime = get_mtime(root).ok()?;
        if cached.root_mtime != current_mtime {
            return None; // Directory was modified
        }

        // Check TTL
        let now = current_timestamp();
        if now - cached.cached_at > Self::TTL_SECS {
            return None; // Cache expired
        }

        Some(cached)
    }

    /// Check if a directory needs to be rescanned
    pub fn directory_needs_rescan(&self, path: &Path) -> bool {
        let Some(cached) = self.directories.get(path) else {
            return true; // Never scanned
        };

        // Check if mtime has changed
        let Ok(current_mtime) = get_mtime(path) else {
            return true; // Can't read mtime, rescan
        };

        cached.mtime != current_mtime
    }

    /// Cache a project
    pub fn cache_project(&mut self, project: Project) {
        let root = project.root.clone();
        let mtime = get_mtime(&root).unwrap_or(0);

        self.projects.insert(root, CachedProject {
            project,
            root_mtime: mtime,
            cached_at: current_timestamp(),
        });
    }

    /// Cache a directory scan result
    pub fn cache_directory(&mut self, path: PathBuf, project_roots: Vec<PathBuf>) {
        let mtime = get_mtime(&path).unwrap_or(0);
        self.directories.insert(path, CachedDirectory {
            mtime,
            project_roots,
        });
    }

    /// Update the cache timestamp
    pub fn touch(&mut self) {
        self.updated_at = current_timestamp();
    }

    /// Check if the entire cache is valid
    pub fn is_valid(&self) -> bool {
        // Version check
        if self.version != Self::VERSION {
            return false;
        }

        // TTL check
        let now = current_timestamp();
        now - self.updated_at < Self::TTL_SECS
    }

    /// Get all valid cached projects
    pub fn get_all_valid_projects(&self) -> Vec<Project> {
        self.projects
            .iter()
            .filter_map(|(path, cached)| {
                self.get_valid_project(path).map(|c| c.project.clone())
            })
            .collect()
    }

    /// Number of cached projects
    pub fn project_count(&self) -> usize {
        self.projects.len()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.projects.clear();
        self.directories.clear();
        self.updated_at = current_timestamp();
    }
}

/// Get the default cache file path
pub fn default_cache_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| DevSweepError::Config("Could not find cache directory".into()))?;

    let devsweep_cache = cache_dir.join("devsweep");
    if !devsweep_cache.exists() {
        fs::create_dir_all(&devsweep_cache)?;
    }

    Ok(devsweep_cache.join("scan_cache.json"))
}

/// Load the cache from disk
pub fn load_cache() -> Result<ScanCache> {
    let path = default_cache_path()?;

    if !path.exists() {
        return Ok(ScanCache::new());
    }

    let content = fs::read_to_string(&path)?;
    let cache: ScanCache = serde_json::from_str(&content)
        .map_err(|e| DevSweepError::Config(format!("Invalid cache file: {}", e)))?;

    // Check version
    if cache.version != ScanCache::VERSION {
        return Ok(ScanCache::new()); // Version mismatch, start fresh
    }

    Ok(cache)
}

/// Save the cache to disk
pub fn save_cache(cache: &ScanCache) -> Result<()> {
    let path = default_cache_path()?;
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Get modification time of a path as unix timestamp
fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata.modified()?;
    Ok(mtime
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs())
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_creation() {
        let cache = ScanCache::new();
        assert_eq!(cache.version, ScanCache::VERSION);
        assert!(cache.projects.is_empty());
    }

    #[test]
    fn test_cache_save_load() {
        let temp = TempDir::new().unwrap();
        let cache_path = temp.path().join("test_cache.json");

        let mut cache = ScanCache::new();
        // Add a dummy project would go here

        let content = serde_json::to_string(&cache).unwrap();
        fs::write(&cache_path, &content).unwrap();

        let loaded: ScanCache = serde_json::from_str(&fs::read_to_string(&cache_path).unwrap()).unwrap();
        assert_eq!(loaded.version, cache.version);
    }
}
