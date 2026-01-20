//! Cleaner trait and types
//!
//! Defines the interface for cleaning (removing) artifacts.

use super::{Artifact, CleanResult, Project};
use crate::error::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

/// Configuration for cleaning operations
#[derive(Debug, Clone)]
pub struct CleanConfig {
    /// Use trash instead of permanent deletion
    pub use_trash: bool,
    /// Dry run - don't actually delete anything
    pub dry_run: bool,
    /// Force clean even with warnings
    pub force: bool,
    /// Skip git status checks
    pub skip_git_check: bool,
    /// Specific artifact kinds to clean (None = all)
    pub artifact_kinds: Option<Vec<super::ArtifactKind>>,
    /// Maximum concurrent delete operations
    pub parallelism: Option<usize>,
    /// Continue on errors
    pub continue_on_error: bool,
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            use_trash: true,
            dry_run: false,
            force: false,
            skip_git_check: false,
            artifact_kinds: None,
            parallelism: None,
            continue_on_error: true,
        }
    }
}

impl CleanConfig {
    /// Create a config that permanently deletes (not trash)
    pub fn permanent() -> Self {
        Self {
            use_trash: false,
            ..Default::default()
        }
    }

    /// Create a dry run config
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            ..Default::default()
        }
    }

    /// Set to force mode
    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Skip git checks
    pub fn without_git_check(mut self) -> Self {
        self.skip_git_check = true;
        self
    }

    /// Filter to specific artifact kinds
    pub fn with_kinds(mut self, kinds: Vec<super::ArtifactKind>) -> Self {
        self.artifact_kinds = Some(kinds);
        self
    }
}

/// What to clean - can be a whole project or specific artifacts
#[derive(Debug, Clone)]
pub enum CleanTarget {
    /// Clean all artifacts in a project
    Project(Project),
    /// Clean specific artifacts
    Artifacts(Vec<Artifact>),
    /// Clean specific paths
    Paths(Vec<PathBuf>),
}

impl CleanTarget {
    /// Get total size to be cleaned
    pub fn total_size(&self) -> u64 {
        match self {
            Self::Project(p) => p.cleanable_size,
            Self::Artifacts(a) => a.iter().map(|a| a.size).sum(),
            Self::Paths(_) => 0, // Unknown
        }
    }

    /// Get number of items
    pub fn count(&self) -> usize {
        match self {
            Self::Project(p) => p.artifacts.len(),
            Self::Artifacts(a) => a.len(),
            Self::Paths(p) => p.len(),
        }
    }
}

/// Progress tracking for clean operations
#[derive(Debug, Default)]
pub struct CleanProgress {
    /// Total items to clean
    pub total_items: AtomicUsize,
    /// Items completed
    pub completed_items: AtomicUsize,
    /// Bytes cleaned so far
    pub bytes_cleaned: AtomicU64,
    /// Bytes that failed to clean
    pub bytes_failed: AtomicU64,
    /// Currently cleaning item
    pub current_item: Mutex<String>,
    /// Errors encountered
    pub errors: Mutex<Vec<CleanError>>,
    /// Whether operation is complete
    pub is_complete: std::sync::atomic::AtomicBool,
    /// Whether operation was cancelled
    pub is_cancelled: std::sync::atomic::AtomicBool,
}

impl CleanProgress {
    /// Create a new progress tracker
    pub fn new(total: usize) -> Arc<Self> {
        let progress = Arc::new(Self::default());
        progress.total_items.store(total, Ordering::Relaxed);
        progress
    }

    /// Mark an item as completed
    pub fn complete_item(&self, bytes: u64) {
        self.completed_items.fetch_add(1, Ordering::Relaxed);
        self.bytes_cleaned.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Mark an item as failed
    pub fn fail_item(&self, bytes: u64, error: CleanError) {
        self.completed_items.fetch_add(1, Ordering::Relaxed);
        self.bytes_failed.fetch_add(bytes, Ordering::Relaxed);
        self.errors.lock().push(error);
    }

    /// Set current item being cleaned
    pub fn set_current(&self, item: impl Into<String>) {
        *self.current_item.lock() = item.into();
    }

    /// Get completion percentage
    pub fn percentage(&self) -> f32 {
        let total = self.total_items.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let completed = self.completed_items.load(Ordering::Relaxed);
        (completed as f32 / total as f32) * 100.0
    }

    /// Request cancellation
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Release);
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::Acquire)
    }

    /// Mark as complete
    pub fn mark_complete(&self) {
        self.is_complete.store(true, Ordering::Release);
    }

    /// Get snapshot
    pub fn snapshot(&self) -> CleanProgressSnapshot {
        CleanProgressSnapshot {
            total_items: self.total_items.load(Ordering::Relaxed),
            completed_items: self.completed_items.load(Ordering::Relaxed),
            bytes_cleaned: self.bytes_cleaned.load(Ordering::Relaxed),
            bytes_failed: self.bytes_failed.load(Ordering::Relaxed),
            current_item: self.current_item.lock().clone(),
            error_count: self.errors.lock().len(),
            is_complete: self.is_complete.load(Ordering::Acquire),
        }
    }
}

/// Snapshot of clean progress
#[derive(Debug, Clone)]
pub struct CleanProgressSnapshot {
    pub total_items: usize,
    pub completed_items: usize,
    pub bytes_cleaned: u64,
    pub bytes_failed: u64,
    pub current_item: String,
    pub error_count: usize,
    pub is_complete: bool,
}

impl CleanProgressSnapshot {
    /// Get percentage complete
    pub fn percentage(&self) -> f32 {
        if self.total_items == 0 {
            return 100.0;
        }
        (self.completed_items as f32 / self.total_items as f32) * 100.0
    }
}

/// Error during clean operation
#[derive(Debug, Clone)]
pub struct CleanError {
    /// Path that failed to clean
    pub path: PathBuf,
    /// Error message
    pub message: String,
    /// Whether the error is recoverable
    pub recoverable: bool,
}

impl CleanError {
    /// Create a new clean error
    pub fn new(path: PathBuf, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
            recoverable: true,
        }
    }
}

/// Summary of a cleaning operation
#[derive(Debug, Clone)]
pub struct CleanSummary {
    /// Total items attempted
    pub total_items: usize,
    /// Items successfully cleaned
    pub succeeded: usize,
    /// Items that failed
    pub failed: usize,
    /// Items skipped (e.g., due to warnings)
    pub skipped: usize,
    /// Total bytes freed
    pub bytes_freed: u64,
    /// Total bytes that failed to clean
    pub bytes_failed: u64,
    /// Whether trash was used
    pub used_trash: bool,
    /// Individual results
    pub results: Vec<CleanResult>,
    /// Errors encountered
    pub errors: Vec<CleanError>,
}

impl CleanSummary {
    /// Create an empty summary
    pub fn empty() -> Self {
        Self {
            total_items: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
            bytes_freed: 0,
            bytes_failed: 0,
            used_trash: false,
            results: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if all items were successful
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0 && self.skipped == 0
    }

    /// Check if any items failed
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }

    /// Get human-readable summary
    pub fn to_string(&self) -> String {
        let freed = humansize::format_size(self.bytes_freed, humansize::BINARY);

        if self.is_complete_success() {
            format!(
                "Successfully cleaned {} items, freed {}",
                self.succeeded, freed
            )
        } else {
            format!(
                "Cleaned {} items ({} freed), {} failed, {} skipped",
                self.succeeded, freed, self.failed, self.skipped
            )
        }
    }
}

/// Trait for implementing cleaners
pub trait Cleaner: Send + Sync {
    /// Clean the specified targets
    fn clean(&self, targets: &[CleanTarget], config: &CleanConfig) -> Result<CleanSummary>;

    /// Clean a single artifact
    fn clean_artifact(&self, artifact: &Artifact, config: &CleanConfig) -> Result<CleanResult>;

    /// Get the progress tracker
    fn progress(&self) -> Arc<CleanProgress>;

    /// Cancel the ongoing clean operation
    fn cancel(&self) {
        self.progress().cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_config_builder() {
        let config = CleanConfig::default().with_force().without_git_check();

        assert!(config.use_trash);
        assert!(config.force);
        assert!(config.skip_git_check);
    }

    #[test]
    fn test_clean_progress() {
        let progress = CleanProgress::new(10);

        progress.complete_item(1000);
        progress.complete_item(500);

        let snapshot = progress.snapshot();
        assert_eq!(snapshot.completed_items, 2);
        assert_eq!(snapshot.bytes_cleaned, 1500);
        assert_eq!(snapshot.percentage(), 20.0);
    }

    #[test]
    fn test_clean_summary() {
        let summary = CleanSummary {
            total_items: 10,
            succeeded: 8,
            failed: 1,
            skipped: 1,
            bytes_freed: 1024 * 1024,
            bytes_failed: 1024,
            used_trash: true,
            results: Vec::new(),
            errors: Vec::new(),
        };

        assert!(!summary.is_complete_success());
        assert!(summary.has_failures());
    }
}
