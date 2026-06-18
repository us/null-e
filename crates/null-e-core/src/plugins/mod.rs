//! Plugin system for null-e
//!
//! Each plugin handles detection and artifact discovery for a specific
//! language/framework ecosystem.

mod dotnet;
mod go;
mod java;
mod node;
mod python;
mod registry;
mod rust;
mod swift;

pub use dotnet::DotNetPlugin;
pub use go::GoPlugin;
pub use java::{GradlePlugin, MavenPlugin};
pub use node::NodePlugin;
pub use python::PythonPlugin;
pub use registry::*;
pub use rust::RustPlugin;
pub use swift::SwiftPlugin;

use crate::core::{Artifact, ProjectKind, ProjectMarker};
use crate::error::Result;
use std::collections::HashSet;
use std::ffi::OsString;
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

    /// Detect using a pre-computed listing of the candidate directory's immediate
    /// child names. The default delegates to [`Plugin::detect`], so plugins that
    /// do not override this keep behaving exactly as before. Plugins can override
    /// this to avoid re-stat-ing marker files that are already known to be present
    /// (or absent) from the shared listing.
    fn detect_with_listing(
        &self,
        path: &Path,
        _listing: &HashSet<OsString>,
    ) -> Option<ProjectKind> {
        self.detect(path)
    }

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

/// Calculate directory size, reporting **on-disk allocated** bytes (hardlink-deduped).
///
/// See [`crate::fsutil::measure_tree`] — allocated size predicts reclaimable space and matches
/// the deletion engine's freed accounting (apparent `st_size` would overstate compressed files).
pub fn default_calculate_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(crate::fsutil::measure_tree(path).1)
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
