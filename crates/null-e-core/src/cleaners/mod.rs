//! Specialized cleanup modules for different development environments
//!
//! This module contains cleanup handlers for:
//! - Xcode (iOS/macOS development)
//! - Android Studio
//! - Docker
//! - ML/AI tools (Huggingface, Ollama, PyTorch)
//! - IDE caches (JetBrains, VS Code)
//! - System logs
//! - Homebrew
//! - iOS Dependencies (CocoaPods, Carthage, SPM)
//! - Electron apps
//! - Game Development (Unity, Unreal, Godot)
//! - Cloud CLI (AWS, GCP, Azure, kubectl)
//! - macOS System (orphaned containers, caches)
//! - Misc tools (Vagrant, Git LFS, Go, Ruby, NuGet, Gradle, Maven)
//! - Testing browsers (Playwright, Cypress, Puppeteer, Selenium)
//! - System cleanup (Trash, Downloads, Temp, Big Files)
//! - Language Runtimes (nvm, pyenv, rbenv, rustup, sdkman, gvm)

pub mod android;
pub mod binaries;
pub mod browsers_test;
pub mod cloud;
pub mod docker;
pub mod electron;
pub mod gamedev;
pub mod homebrew;
pub mod ide;
pub mod ios_deps;
pub mod logs;
pub mod macos;
pub mod misc;
pub mod ml;
pub mod runtimes;
pub mod system;
pub mod xcode;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::SystemTime;

/// Re-export of `rayon` so downstream crates (e.g. the Tauri GUI) can drive cleaner detection in
/// parallel without taking a direct dependency on `rayon`. Keeping the re-export here lets the
/// GUI's `collect_cleanable_items` fan the cleaners out across a thread pool.
pub use rayon;

/// `~/Library/Caches` child directories that are *owned* by a specialized cleaner.
///
/// Each tuple is `(dir-name-prefix, owning-cleaner-id)`. [`system::SystemCleaner`] walks the
/// per-app subdirectories of `~/Library/Caches` and would otherwise re-measure these subtrees that
/// a specialized cleaner (IDE, ML, ...) already detects — double-walking the same bytes and
/// emitting the cache twice. `SystemCleaner` skips any child whose directory name *starts with* one
/// of these prefixes, so each owned cache is detected exactly once by its specialized cleaner.
///
/// Matching is prefix-based (`name.starts_with(prefix)`) to mirror how the specialized cleaners
/// resolve these paths (e.g. the IDE cleaner's `com.microsoft.VSCode` entry also covers
/// `com.microsoft.VSCode.ShipIt`).
///
/// IMPORTANT: this list MUST be updated whenever a specialized cleaner gains a new
/// `~/Library/Caches/<name>` path, otherwise that cache would be double-counted again.
pub const OWNED_CACHE_PREFIXES: &[(&str, &str)] = &[
    // IDE cleaner (cleaners/ide.rs)
    ("JetBrains", "ide"),                     // detect_jetbrains_macos
    ("com.microsoft.VSCode", "ide"),          // detect_vscode (covers .VSCode and .VSCode.ShipIt)
    ("com.todesktop.230313mzl4w4u92", "ide"), // detect_cursor (Cursor)
    ("com.sublimetext.4", "ide"),             // detect_sublime
    ("dev.zed.Zed", "ide"),                   // detect_zed
];

/// Returns the owning-cleaner id if `name` is a `~/Library/Caches` child owned by a specialized
/// cleaner (see [`OWNED_CACHE_PREFIXES`]), otherwise `None`.
pub fn owned_cache_owner(name: &str) -> Option<&'static str> {
    OWNED_CACHE_PREFIXES
        .iter()
        .find(|(prefix, _)| name.starts_with(prefix))
        .map(|(_, owner)| *owner)
}

/// A cleanable item found by a cleaner module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanableItem {
    /// Human-readable name
    pub name: String,
    /// Category (e.g., "Xcode", "Docker")
    pub category: String,
    /// Subcategory (e.g., "DerivedData", "Simulators")
    pub subcategory: String,
    /// Icon for display
    pub icon: &'static str,
    /// Full path
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Number of files (if applicable)
    pub file_count: Option<u64>,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Description of what this is
    pub description: Cow<'static, str>,
    /// Is it safe to delete?
    pub safe_to_delete: SafetyLevel,
    /// Official clean command (if available)
    pub clean_command: Option<String>,
}

/// Safety level for deletion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Safe to delete, will be regenerated
    Safe,
    /// Safe but may slow down next build/operation
    SafeWithCost,
    /// Use caution - may lose some data
    Caution,
    /// Dangerous - may break things
    Dangerous,
}

impl SafetyLevel {
    /// Get a color hint for display
    pub fn color_hint(&self) -> &'static str {
        match self {
            Self::Safe => "green",
            Self::SafeWithCost => "yellow",
            Self::Caution => "red",
            Self::Dangerous => "magenta",
        }
    }

    /// Get a symbol for display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Safe => "✓",
            Self::SafeWithCost => "~",
            Self::Caution => "!",
            Self::Dangerous => "⚠",
        }
    }
}

/// How (and whether) an item can actually be reclaimed by the user.
///
/// This drives honest UI grouping and the "why can't I delete this" copy — it is the antidote to
/// the "59 GB shows but won't free" complaint: space the user *can* reclaim is separated from
/// space that needs admin, is OS-managed/purgeable, or is SIP-protected and unremovable by anyone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Reclaimability {
    /// The user's own process can delete this (the common case).
    #[default]
    UserReclaimable,
    /// Root-owned — needs admin rights (v1 surfaces this, does not auto-elevate).
    NeedsAdmin,
    /// OS-managed / purgeable (e.g. local Time Machine snapshots) — reclaim varies, not a plain delete.
    OsManagedPurgeable,
    /// SIP-protected / sealed system volume — cannot be removed by any app.
    SipProtected,
}

impl Reclaimability {
    /// Stable string id for the DTO / UI.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserReclaimable => "user_reclaimable",
            Self::NeedsAdmin => "needs_admin",
            Self::OsManagedPurgeable => "os_managed_purgeable",
            Self::SipProtected => "sip_protected",
        }
    }
}

impl CleanableItem {
    /// Check if this item exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Classify how this item can be reclaimed (drives honest UI grouping).
    pub fn reclaimability(&self) -> Reclaimability {
        // Time Machine local snapshots are OS-managed/purgeable — reclaim varies.
        let sub = self.subcategory.to_lowercase();
        if sub.contains("time machine") || sub.contains("snapshot") {
            return Reclaimability::OsManagedPurgeable;
        }
        // SIP / sealed system volume — unremovable by any app.
        if self.path.starts_with("/System") {
            return Reclaimability::SipProtected;
        }
        // Root-owned system paths need admin.
        if crate::fsutil::is_root_owned(&self.path) {
            return Reclaimability::NeedsAdmin;
        }
        Reclaimability::UserReclaimable
    }

    /// Bytes we can honestly claim are reclaimable by the user.
    ///
    /// 0 for OS-managed/purgeable and SIP-protected items (we won't promise space we can't free);
    /// the item's size otherwise (incl. NeedsAdmin, which is reclaimable *with* admin rights).
    pub fn reclaimable_bytes(&self) -> u64 {
        match self.reclaimability() {
            Reclaimability::OsManagedPurgeable | Reclaimability::SipProtected => 0,
            Reclaimability::UserReclaimable | Reclaimability::NeedsAdmin => self.size,
        }
    }

    /// Get age in days
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

/// Summary of cleanable items from all modules
#[derive(Debug, Default)]
pub struct CleanerSummary {
    pub total_items: usize,
    pub total_size: u64,
    pub by_category: std::collections::HashMap<String, CategorySummary>,
}

/// Summary for a single category
#[derive(Debug, Default, Clone)]
pub struct CategorySummary {
    pub name: String,
    pub icon: &'static str,
    pub item_count: usize,
    pub total_size: u64,
}

impl CleanerSummary {
    pub fn from_items(items: &[CleanableItem]) -> Self {
        let mut summary = Self {
            total_items: items.len(),
            total_size: items.iter().map(|i| i.size).sum(),
            ..Default::default()
        };

        for item in items {
            let entry = summary
                .by_category
                .entry(item.category.clone())
                .or_insert_with(|| CategorySummary {
                    name: item.category.clone(),
                    icon: item.icon,
                    ..Default::default()
                });
            entry.item_count += 1;
            entry.total_size += item.size;
        }

        summary
    }
}

/// Calculate directory size recursively, returning `(allocated_bytes, file_count)`.
///
/// Reports **on-disk allocated** size (`st_blocks * 512`, hardlink-deduped) rather than apparent
/// size, so the figure predicts reclaimable space and stays consistent with the deletion engine's
/// freed accounting. See [`crate::fsutil::measure_tree_counted`].
pub fn calculate_dir_size(path: &std::path::Path) -> Result<(u64, u64)> {
    if !path.exists() {
        return Ok((0, 0));
    }
    let m = crate::fsutil::measure_tree_counted(path);
    Ok((m.allocated, m.file_count))
}

/// Get last modification time of a path
pub fn get_mtime(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
