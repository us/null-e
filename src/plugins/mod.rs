//! Plugin system for DevSweep
//!
//! Each plugin handles detection and artifact discovery for a specific
//! language/framework ecosystem.

mod registry;
mod node;
mod rust;
mod python;
mod go;
mod java;
mod dotnet;
mod swift;

pub use registry::*;
pub use node::NodePlugin;
pub use rust::RustPlugin;
pub use python::PythonPlugin;
pub use go::GoPlugin;
pub use java::{MavenPlugin, GradlePlugin};
pub use dotnet::DotNetPlugin;
pub use swift::SwiftPlugin;

use crate::core::{Artifact, ProjectKind, ProjectMarker};
use crate::error::Result;
use std::path::Path;

/// Trait that all language/framework plugins must implement
pub trait Plugin: Send + Sync {
    /// Unique identifier for this plugin
    fn id(&self) -> &'static str;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Project kinds this plugin handles
    fn supported_kinds(&self) -> &[ProjectKind];

    /// Markers that identify projects this plugin handles
    fn markers(&self) -> Vec<ProjectMarker>;

    /// Detect if path is a project root for this plugin
    fn detect(&self, path: &Path) -> Option<ProjectKind>;

    /// Find cleanable artifacts in a project directory
    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>>;

    /// Custom size calculation (override for special cases)
    fn calculate_size(&self, artifact: &Artifact) -> Result<u64> {
        default_calculate_size(&artifact.path)
    }

    /// Pre-clean hook (e.g., stop running processes)
    fn pre_clean(&self, _artifact: &Artifact) -> Result<()> {
        Ok(())
    }

    /// Post-clean hook (e.g., update state files)
    fn post_clean(&self, _artifact: &Artifact) -> Result<()> {
        Ok(())
    }

    /// Priority when multiple plugins match (higher = preferred)
    fn priority(&self) -> u8 {
        50
    }

    /// Get cleanable directory names for fast scanning
    fn cleanable_dirs(&self) -> &[&'static str] {
        &[]
    }
}

/// Calculate directory size using parallel walk
pub fn default_calculate_size(path: &Path) -> Result<u64> {
    use rayon::prelude::*;
    use walkdir::WalkDir;

    if !path.exists() {
        return Ok(0);
    }

    // For small directories, use simple walk
    let entries: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let size: u64 = entries
        .par_iter()
        .filter_map(|entry| entry.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum();

    Ok(size)
}

/// Count files in a directory
pub fn count_files(path: &Path) -> Result<u64> {
    use walkdir::WalkDir;

    if !path.exists() {
        return Ok(0);
    }

    let count = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as u64;

    Ok(count)
}

/// Get all built-in plugins
pub fn builtin_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(NodePlugin),
        Box::new(RustPlugin),
        Box::new(PythonPlugin),
        Box::new(GoPlugin),
        Box::new(MavenPlugin),
        Box::new(GradlePlugin),
        Box::new(DotNetPlugin),
        Box::new(SwiftPlugin),
    ]
}
