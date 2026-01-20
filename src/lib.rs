//! # null-e 🤖
//!
//! The Friendly Disk Cleanup Robot - Send your cruft to /dev/null!
//!
//! null-e helps developers reclaim disk space by finding and cleaning
//! development artifacts like `node_modules`, `target`, `.venv`, and 30+ more.
//!
//! ## Features
//!
//! - **Multi-language support**: Node.js, Rust, Python, Go, Java, .NET, Swift, and more
//! - **Git integration**: Protects uncommitted changes
//! - **Trash support**: Safe deletion with recovery option
//! - **Parallel scanning**: Fast directory traversal
//! - **Interactive TUI**: Beautiful terminal interface
//! - **Docker support**: Clean dangling images and volumes
//!
//! ## Quick Start
//!
//! ```no_run
//! use null_e::prelude::*;
//!
//! // Create a scanner with default plugins
//! let registry = PluginRegistry::with_builtins();
//! let scanner = ParallelScanner::new(std::sync::Arc::new(registry));
//!
//! // Scan a directory
//! let config = ScanConfig::new("/path/to/projects");
//! let result = scanner.scan(&config).unwrap();
//!
//! println!("Found {} projects with {} cleanable",
//!     result.projects.len(),
//!     humansize::format_size(result.total_cleanable, humansize::BINARY)
//! );
//! ```
//!
//! ## Architecture
//!
//! null-e uses a plugin-based architecture where each language/framework
//! is handled by a dedicated plugin. Plugins implement the `Plugin` trait
//! and register themselves with the `PluginRegistry`.
//!
//! ```text
//!      .---.
//!     |o   o|
//!     |  ^  |    ┌─────────────────┐
//!     | === |    │    CLI/TUI      │
//!     `-----'    └────────┬────────┘
//!      /| |\              │
//!                ┌────────▼────────┐
//!                │   Core Engine   │
//!                │  - Scanner      │
//!                │  - Cleaner      │
//!                └────────┬────────┘
//!                         │
//!                ┌────────▼────────┐
//!                │ Plugin Registry │
//!                │  - Node.js      │
//!                │  - Rust         │
//!                │  - Python       │
//!                │  - ...          │
//!                └─────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod analysis;
pub mod cache;
pub mod caches;
pub mod cleaners;
pub mod config;
pub mod core;
pub mod docker;
pub mod error;
pub mod git;
pub mod plugins;
pub mod scanner;
pub mod trash;
pub mod tui;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{
        Artifact, ArtifactKind, ArtifactMetadata, ArtifactStats,
        CleanConfig, CleanProgress, CleanResult, CleanSafety, CleanSummary, CleanTarget, Cleaner,
        Project, ProjectId, ProjectKind,
        ScanConfig, ScanProgress, ScanResult, Scanner,
    };
    pub use crate::config::Config;
    pub use crate::error::{DevSweepError, Result, ResultExt};
    pub use crate::git::{ProtectionLevel, get_git_status};
    pub use crate::plugins::{Plugin, PluginRegistry};
    pub use crate::scanner::ParallelScanner;
    pub use crate::trash::{DeleteMethod, delete_path, delete_artifact};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name
pub const NAME: &str = env!("CARGO_PKG_NAME");
