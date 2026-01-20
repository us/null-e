//! Artifact types and metadata
//!
//! Artifacts are cleanable items within a project, such as:
//! - Dependencies (node_modules, vendor)
//! - Build outputs (target, dist, build)
//! - Caches (__pycache__, .cache)
//! - Virtual environments (.venv)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// A cleanable artifact within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Full path to the artifact
    pub path: PathBuf,
    /// Type of artifact
    pub kind: ArtifactKind,
    /// Size in bytes
    pub size: u64,
    /// Number of files contained
    pub file_count: u64,
    /// Age since last modification
    pub age: Option<Duration>,
    /// Additional metadata
    pub metadata: ArtifactMetadata,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(path: PathBuf, kind: ArtifactKind) -> Self {
        Self {
            path,
            kind,
            size: 0,
            file_count: 0,
            age: None,
            metadata: ArtifactMetadata::default(),
        }
    }

    /// Get the artifact name (directory/file name)
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Check if this artifact can be safely deleted based on its kind
    pub fn is_safe_to_clean(&self) -> bool {
        match self.kind.default_safety() {
            ArtifactSafety::AlwaysSafe => true,
            ArtifactSafety::SafeIfGitClean => true, // Caller should check git
            ArtifactSafety::SafeWithLockfile => self.metadata.lockfile.is_some(),
            ArtifactSafety::RequiresConfirmation => false,
            ArtifactSafety::NeverAuto => false,
        }
    }

    /// Get a human-readable size string
    pub fn size_display(&self) -> String {
        humansize::format_size(self.size, humansize::BINARY)
    }
}

impl std::fmt::Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) - {}",
            self.name(),
            self.kind.description(),
            self.size_display()
        )
    }
}

/// Classification of artifact types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ArtifactKind {
    /// Package dependencies (node_modules, vendor, etc.)
    Dependencies,
    /// Build outputs (target, dist, build, etc.)
    BuildOutput,
    /// Cache directories (.cache, __pycache__, etc.)
    Cache,
    /// Virtual environments (.venv, venv, etc.)
    VirtualEnv,
    /// IDE/editor artifacts (.idea, .vscode local, etc.)
    IdeArtifacts,
    /// Test outputs (coverage, .nyc_output, etc.)
    TestOutput,
    /// Log files
    Logs,
    /// Temporary files
    Temporary,
    /// Lock files (generally should NOT delete)
    LockFile,
    /// Docker-related (dangling images, build cache)
    Docker,
    /// Package manager cache (npm cache, pip cache)
    PackageManagerCache,
    /// Compiled bytecode (.pyc files, .class files)
    Bytecode,
    /// Documentation builds
    DocsBuild,
    /// Custom plugin-defined artifact
    Custom(u32),
}

impl ArtifactKind {
    /// Default safety level for this artifact type
    pub fn default_safety(&self) -> ArtifactSafety {
        match self {
            Self::Cache | Self::Logs | Self::Temporary | Self::Bytecode => ArtifactSafety::AlwaysSafe,
            Self::BuildOutput | Self::TestOutput | Self::DocsBuild => ArtifactSafety::SafeIfGitClean,
            Self::Dependencies | Self::PackageManagerCache => ArtifactSafety::SafeWithLockfile,
            Self::VirtualEnv => ArtifactSafety::RequiresConfirmation,
            Self::IdeArtifacts => ArtifactSafety::RequiresConfirmation,
            Self::Docker => ArtifactSafety::RequiresConfirmation,
            Self::LockFile => ArtifactSafety::NeverAuto,
            Self::Custom(_) => ArtifactSafety::RequiresConfirmation,
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Dependencies => "dependencies",
            Self::BuildOutput => "build output",
            Self::Cache => "cache",
            Self::VirtualEnv => "virtual environment",
            Self::IdeArtifacts => "IDE artifacts",
            Self::TestOutput => "test output",
            Self::Logs => "logs",
            Self::Temporary => "temporary files",
            Self::LockFile => "lock file",
            Self::Docker => "Docker artifacts",
            Self::PackageManagerCache => "package cache",
            Self::Bytecode => "bytecode",
            Self::DocsBuild => "documentation build",
            Self::Custom(_) => "custom",
        }
    }

    /// Get the icon/emoji for this artifact kind
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Dependencies => "📦",
            Self::BuildOutput => "🔨",
            Self::Cache => "💾",
            Self::VirtualEnv => "🐍",
            Self::IdeArtifacts => "💻",
            Self::TestOutput => "🧪",
            Self::Logs => "📝",
            Self::Temporary => "🗑️",
            Self::LockFile => "🔒",
            Self::Docker => "🐳",
            Self::PackageManagerCache => "📥",
            Self::Bytecode => "⚙️",
            Self::DocsBuild => "📚",
            Self::Custom(_) => "📁",
        }
    }

    /// Get all standard artifact kinds
    pub fn all() -> &'static [ArtifactKind] {
        &[
            Self::Dependencies,
            Self::BuildOutput,
            Self::Cache,
            Self::VirtualEnv,
            Self::IdeArtifacts,
            Self::TestOutput,
            Self::Logs,
            Self::Temporary,
            Self::Docker,
            Self::PackageManagerCache,
            Self::Bytecode,
            Self::DocsBuild,
        ]
    }
}

/// Safety classification for artifacts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactSafety {
    /// Always safe to delete (cache, temp, logs)
    AlwaysSafe,
    /// Safe if git working tree is clean
    SafeIfGitClean,
    /// Safe if lockfile exists (dependencies can be reinstalled)
    SafeWithLockfile,
    /// Requires explicit user confirmation
    RequiresConfirmation,
    /// Should never be auto-deleted
    NeverAuto,
}

/// Additional metadata about an artifact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Whether this can be restored (reinstalled/rebuilt)
    pub restorable: bool,
    /// Command to restore (e.g., "npm install", "cargo build")
    pub restore_command: Option<String>,
    /// Associated lockfile that enables restoration
    pub lockfile: Option<PathBuf>,
    /// Estimated restoration time in seconds
    pub restore_time_estimate: Option<u32>,
    /// Custom properties from plugins
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl ArtifactMetadata {
    /// Create metadata for a restorable artifact
    pub fn restorable(command: impl Into<String>) -> Self {
        Self {
            restorable: true,
            restore_command: Some(command.into()),
            ..Default::default()
        }
    }

    /// Set the lockfile
    pub fn with_lockfile(mut self, lockfile: PathBuf) -> Self {
        self.lockfile = Some(lockfile);
        self
    }

    /// Set restoration time estimate
    pub fn with_restore_time(mut self, seconds: u32) -> Self {
        self.restore_time_estimate = Some(seconds);
        self
    }
}

/// Statistics about artifacts found during scan
#[derive(Debug, Clone, Default)]
pub struct ArtifactStats {
    /// Total size of all artifacts
    pub total_size: u64,
    /// Total number of files
    pub total_files: u64,
    /// Total number of artifacts
    pub total_artifacts: usize,
    /// Stats by artifact kind
    pub by_kind: HashMap<ArtifactKind, KindStats>,
}

impl ArtifactStats {
    /// Add an artifact to the stats
    pub fn add(&mut self, artifact: &Artifact) {
        self.total_size += artifact.size;
        self.total_files += artifact.file_count;
        self.total_artifacts += 1;

        let entry = self.by_kind.entry(artifact.kind).or_default();
        entry.count += 1;
        entry.total_size += artifact.size;
        entry.file_count += artifact.file_count;
    }

    /// Get the largest artifact kind by size
    pub fn largest_kind(&self) -> Option<(ArtifactKind, u64)> {
        self.by_kind
            .iter()
            .max_by_key(|(_, stats)| stats.total_size)
            .map(|(kind, stats)| (*kind, stats.total_size))
    }
}

/// Statistics for a specific artifact kind
#[derive(Debug, Clone, Default)]
pub struct KindStats {
    /// Number of artifacts of this kind
    pub count: usize,
    /// Total size
    pub total_size: u64,
    /// Total file count
    pub file_count: u64,
}

/// Result of a cleaning operation for a single artifact
#[derive(Debug, Clone)]
pub struct CleanResult {
    /// The artifact that was (attempted to be) cleaned
    pub artifact: Artifact,
    /// Whether the clean was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Actual bytes freed (may differ from artifact.size)
    pub bytes_freed: u64,
    /// Whether it was moved to trash (vs permanent delete)
    pub trashed: bool,
}

impl CleanResult {
    /// Create a successful clean result
    pub fn success(artifact: Artifact, trashed: bool) -> Self {
        let bytes_freed = artifact.size;
        Self {
            artifact,
            success: true,
            error: None,
            bytes_freed,
            trashed,
        }
    }

    /// Create a failed clean result
    pub fn failure(artifact: Artifact, error: impl Into<String>) -> Self {
        Self {
            artifact,
            success: false,
            error: Some(error.into()),
            bytes_freed: 0,
            trashed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_kind_safety() {
        assert_eq!(
            ArtifactKind::Cache.default_safety(),
            ArtifactSafety::AlwaysSafe
        );
        assert_eq!(
            ArtifactKind::Dependencies.default_safety(),
            ArtifactSafety::SafeWithLockfile
        );
        assert_eq!(
            ArtifactKind::LockFile.default_safety(),
            ArtifactSafety::NeverAuto
        );
    }

    #[test]
    fn test_artifact_stats() {
        let mut stats = ArtifactStats::default();

        let artifact1 = Artifact {
            path: PathBuf::from("/test/node_modules"),
            kind: ArtifactKind::Dependencies,
            size: 1000,
            file_count: 100,
            age: None,
            metadata: ArtifactMetadata::default(),
        };

        let artifact2 = Artifact {
            path: PathBuf::from("/test/.cache"),
            kind: ArtifactKind::Cache,
            size: 500,
            file_count: 50,
            age: None,
            metadata: ArtifactMetadata::default(),
        };

        stats.add(&artifact1);
        stats.add(&artifact2);

        assert_eq!(stats.total_size, 1500);
        assert_eq!(stats.total_files, 150);
        assert_eq!(stats.total_artifacts, 2);
    }
}
