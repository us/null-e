//! # null-e-core
//!
//! Core library for the null-e disk cleanup tool.
//!
//! This crate provides the scanning, cleaning, and plugin infrastructure.
//! It is used by both the CLI/TUI and the Tauri GUI frontends.

#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
// Several hot-path sorts use an explicit descending comparator (`b.x.cmp(&a.x)`), which clippy
// flags in favour of `sort_by_key(|e| Reverse(e.x))`. The explicit form reads clearer at the call
// sites and avoids per-element `Reverse` wrapping, so we keep it deliberately.
#![allow(clippy::unnecessary_sort_by)]

pub mod analysis;
pub mod cache;
pub mod caches;
pub mod cleaners;
pub mod config;
pub mod core;
pub mod docker;
pub mod error;
pub mod fsutil;
pub mod git;
pub mod plugins;
pub mod scanner;
pub mod trash;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::core::{
        Artifact, ArtifactKind, ArtifactMetadata, ArtifactStats, CleanConfig, CleanProgress,
        CleanResult, CleanSafety, CleanSummary, CleanTarget, Cleaner, Project, ProjectId,
        ProjectKind, ScanConfig, ScanProgress, ScanResult, Scanner,
    };
    pub use crate::error::{NullEError, Result, ResultExt};
    pub use crate::git::{get_git_status, ProtectionLevel};
    pub use crate::plugins::{Plugin, PluginRegistry};
    pub use crate::scanner::ParallelScanner;
    pub use crate::trash::{delete_artifact, delete_path, DeleteMethod};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name
pub const NAME: &str = env!("CARGO_PKG_NAME");
