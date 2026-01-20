//! Scanner trait and types
//!
//! Defines the interface for scanning filesystems for projects and artifacts.

use super::{Project, ArtifactStats};
use crate::error::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;

/// Configuration for scanning operations
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Root directories to scan
    pub roots: Vec<PathBuf>,
    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,
    /// Number of parallel threads (None = auto based on CPU)
    pub parallelism: Option<usize>,
    /// Skip hidden files and directories
    pub skip_hidden: bool,
    /// Respect .gitignore files
    pub respect_gitignore: bool,
    /// Custom ignore patterns (glob syntax)
    pub ignore_patterns: Vec<String>,
    /// Minimum artifact size to report (bytes)
    pub min_size: Option<u64>,
    /// Maximum number of projects to return
    pub limit: Option<usize>,
    /// Include git status check for each project
    pub check_git_status: bool,
    /// Timeout for the entire scan operation
    pub timeout: Option<Duration>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            roots: vec![],
            max_depth: None,
            parallelism: None,
            skip_hidden: true,
            respect_gitignore: true,
            ignore_patterns: vec![],
            min_size: None,
            limit: None,
            check_git_status: true,
            timeout: None,
        }
    }
}

impl ScanConfig {
    /// Create a new config with a single root directory
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            roots: vec![root.into()],
            ..Default::default()
        }
    }

    /// Add a root directory
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.roots.push(root.into());
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Set parallelism
    pub fn with_parallelism(mut self, threads: usize) -> Self {
        self.parallelism = Some(threads);
        self
    }

    /// Add ignore pattern
    pub fn with_ignore(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_patterns.push(pattern.into());
        self
    }

    /// Set minimum size filter
    pub fn with_min_size(mut self, size: u64) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Disable git status checking
    pub fn without_git_check(mut self) -> Self {
        self.check_git_status = false;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Real-time scan progress information
#[derive(Debug, Default)]
pub struct ScanProgress {
    /// Number of directories scanned
    pub directories_scanned: AtomicUsize,
    /// Number of projects found
    pub projects_found: AtomicUsize,
    /// Total cleanable size found so far
    pub total_size_found: AtomicU64,
    /// Currently scanning path
    pub current_path: Mutex<PathBuf>,
    /// Errors encountered during scan
    pub errors: Mutex<Vec<ScanError>>,
    /// Whether scan is complete
    pub is_complete: std::sync::atomic::AtomicBool,
    /// Whether scan was cancelled
    pub is_cancelled: std::sync::atomic::AtomicBool,
}

impl ScanProgress {
    /// Create a new progress tracker
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Increment directories scanned
    pub fn inc_directories(&self) {
        self.directories_scanned.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment projects found
    pub fn inc_projects(&self) {
        self.projects_found.fetch_add(1, Ordering::Relaxed);
    }

    /// Add to total size
    pub fn add_size(&self, size: u64) {
        self.total_size_found.fetch_add(size, Ordering::Relaxed);
    }

    /// Update current path
    pub fn set_current_path(&self, path: PathBuf) {
        *self.current_path.lock() = path;
    }

    /// Add an error
    pub fn add_error(&self, error: ScanError) {
        self.errors.lock().push(error);
    }

    /// Mark as complete
    pub fn mark_complete(&self) {
        self.is_complete.store(true, Ordering::Release);
    }

    /// Request cancellation
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Release);
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::Acquire)
    }

    /// Get snapshot of current progress
    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            directories_scanned: self.directories_scanned.load(Ordering::Relaxed),
            projects_found: self.projects_found.load(Ordering::Relaxed),
            total_size_found: self.total_size_found.load(Ordering::Relaxed),
            current_path: self.current_path.lock().clone(),
            error_count: self.errors.lock().len(),
            is_complete: self.is_complete.load(Ordering::Acquire),
        }
    }
}

/// Snapshot of progress at a point in time
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub directories_scanned: usize,
    pub projects_found: usize,
    pub total_size_found: u64,
    pub current_path: PathBuf,
    pub error_count: usize,
    pub is_complete: bool,
}

/// Error encountered during scan
#[derive(Debug, Clone)]
pub struct ScanError {
    /// Path where error occurred
    pub path: PathBuf,
    /// Error description
    pub message: String,
    /// Whether this error is recoverable
    pub recoverable: bool,
}

impl ScanError {
    /// Create a new scan error
    pub fn new(path: PathBuf, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a non-recoverable error
    pub fn fatal(path: PathBuf, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
            recoverable: false,
        }
    }
}

/// Result of a complete scan
#[derive(Debug)]
pub struct ScanResult {
    /// Projects found
    pub projects: Vec<Project>,
    /// Total size of all artifacts
    pub total_size: u64,
    /// Total cleanable size
    pub total_cleanable: u64,
    /// Scan duration
    pub duration: Duration,
    /// Number of directories scanned
    pub directories_scanned: usize,
    /// Errors encountered
    pub errors: Vec<ScanError>,
    /// Statistics by artifact kind
    pub stats: ArtifactStats,
}

impl ScanResult {
    /// Get the number of projects
    pub fn project_count(&self) -> usize {
        self.projects.len()
    }

    /// Get total artifact count across all projects
    pub fn artifact_count(&self) -> usize {
        self.projects.iter().map(|p| p.artifacts.len()).sum()
    }

    /// Get projects sorted by cleanable size (descending)
    pub fn projects_by_size(&self) -> Vec<&Project> {
        let mut projects: Vec<_> = self.projects.iter().collect();
        projects.sort_by(|a, b| b.cleanable_size.cmp(&a.cleanable_size));
        projects
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Found {} projects with {} cleanable across {} artifacts in {:.2}s",
            self.projects.len(),
            humansize::format_size(self.total_cleanable, humansize::BINARY),
            self.artifact_count(),
            self.duration.as_secs_f64()
        )
    }
}

/// Trait for implementing scanners
pub trait Scanner: Send + Sync {
    /// Run the scan with the given configuration
    fn scan(&self, config: &ScanConfig) -> Result<ScanResult>;

    /// Get the progress tracker
    fn progress(&self) -> Arc<ScanProgress>;

    /// Cancel the ongoing scan
    fn cancel(&self) {
        self.progress().cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_config_builder() {
        let config = ScanConfig::new("/home/user")
            .with_root("/tmp")
            .with_max_depth(5)
            .with_min_size(1024)
            .with_ignore("*.bak");

        assert_eq!(config.roots.len(), 2);
        assert_eq!(config.max_depth, Some(5));
        assert_eq!(config.min_size, Some(1024));
        assert_eq!(config.ignore_patterns.len(), 1);
    }

    #[test]
    fn test_scan_progress() {
        let progress = ScanProgress::new();

        progress.inc_directories();
        progress.inc_directories();
        progress.inc_projects();
        progress.add_size(1000);

        let snapshot = progress.snapshot();
        assert_eq!(snapshot.directories_scanned, 2);
        assert_eq!(snapshot.projects_found, 1);
        assert_eq!(snapshot.total_size_found, 1000);
    }

    #[test]
    fn test_scan_progress_cancellation() {
        let progress = ScanProgress::new();
        assert!(!progress.is_cancelled());

        progress.cancel();
        assert!(progress.is_cancelled());
    }
}
