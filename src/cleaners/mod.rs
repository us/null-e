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

pub mod xcode;
pub mod android;
pub mod docker;
pub mod ml;
pub mod ide;
pub mod logs;
pub mod homebrew;
pub mod ios_deps;
pub mod electron;
pub mod gamedev;
pub mod cloud;
pub mod macos;
pub mod misc;
pub mod browsers_test;
pub mod system;
pub mod runtimes;
pub mod binaries;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

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
    pub description: &'static str,
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

impl CleanableItem {
    /// Check if this item exists
    pub fn exists(&self) -> bool {
        self.path.exists()
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
        let mut summary = Self::default();
        summary.total_items = items.len();
        summary.total_size = items.iter().map(|i| i.size).sum();

        for item in items {
            let entry = summary.by_category
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

/// Calculate directory size recursively
pub fn calculate_dir_size(path: &std::path::Path) -> Result<(u64, u64)> {
    use rayon::prelude::*;
    use walkdir::WalkDir;

    if !path.exists() {
        return Ok((0, 0));
    }

    let entries: Vec<_> = WalkDir::new(path)
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

    Ok((size, count))
}

/// Get last modification time of a path
pub fn get_mtime(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
