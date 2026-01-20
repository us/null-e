//! Project detection and representation
//!
//! A Project represents a detected development project directory,
//! including its type, artifacts, and safety status.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Artifact;

/// Unique identifier for a detected project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

impl ProjectId {
    /// Create a project ID from a path
    pub fn from_path(path: &Path) -> Self {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// The type/ecosystem of a detected project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProjectKind {
    // ═══════════════════════════════════════════════════════════════
    // JavaScript/TypeScript Ecosystem
    // ═══════════════════════════════════════════════════════════════
    NodeNpm,
    NodeYarn,
    NodePnpm,
    NodeBun,
    Deno,

    // ═══════════════════════════════════════════════════════════════
    // Systems Languages
    // ═══════════════════════════════════════════════════════════════
    Rust,
    Go,
    Cpp,
    C,
    Zig,

    // ═══════════════════════════════════════════════════════════════
    // JVM Ecosystem
    // ═══════════════════════════════════════════════════════════════
    JavaMaven,
    JavaGradle,
    Kotlin,
    Scala,
    Clojure,

    // ═══════════════════════════════════════════════════════════════
    // .NET Ecosystem
    // ═══════════════════════════════════════════════════════════════
    DotNet,
    FSharp,

    // ═══════════════════════════════════════════════════════════════
    // Python Ecosystem
    // ═══════════════════════════════════════════════════════════════
    PythonPip,
    PythonPoetry,
    PythonPipenv,
    PythonConda,
    PythonUv,

    // ═══════════════════════════════════════════════════════════════
    // Ruby Ecosystem
    // ═══════════════════════════════════════════════════════════════
    RubyBundler,
    RubyRails,

    // ═══════════════════════════════════════════════════════════════
    // PHP Ecosystem
    // ═══════════════════════════════════════════════════════════════
    PhpComposer,
    PhpLaravel,

    // ═══════════════════════════════════════════════════════════════
    // Mobile Development
    // ═══════════════════════════════════════════════════════════════
    SwiftSpm,
    SwiftXcode,
    Flutter,
    ReactNative,
    Android,

    // ═══════════════════════════════════════════════════════════════
    // Other Languages
    // ═══════════════════════════════════════════════════════════════
    Elixir,
    Haskell,
    OCaml,
    Julia,
    R,
    Lua,
    Perl,

    // ═══════════════════════════════════════════════════════════════
    // Infrastructure as Code
    // ═══════════════════════════════════════════════════════════════
    Terraform,
    Pulumi,

    // ═══════════════════════════════════════════════════════════════
    // Containers
    // ═══════════════════════════════════════════════════════════════
    Docker,

    /// Custom plugin-defined project type
    Custom(u32),
}

impl ProjectKind {
    /// Get a human-readable name for this project kind
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::NodeNpm => "Node.js (npm)",
            Self::NodeYarn => "Node.js (Yarn)",
            Self::NodePnpm => "Node.js (pnpm)",
            Self::NodeBun => "Bun",
            Self::Deno => "Deno",
            Self::Rust => "Rust (Cargo)",
            Self::Go => "Go",
            Self::Cpp => "C++",
            Self::C => "C",
            Self::Zig => "Zig",
            Self::JavaMaven => "Java (Maven)",
            Self::JavaGradle => "Java (Gradle)",
            Self::Kotlin => "Kotlin",
            Self::Scala => "Scala",
            Self::Clojure => "Clojure",
            Self::DotNet => ".NET",
            Self::FSharp => "F#",
            Self::PythonPip => "Python (pip)",
            Self::PythonPoetry => "Python (Poetry)",
            Self::PythonPipenv => "Python (Pipenv)",
            Self::PythonConda => "Python (Conda)",
            Self::PythonUv => "Python (uv)",
            Self::RubyBundler => "Ruby (Bundler)",
            Self::RubyRails => "Ruby on Rails",
            Self::PhpComposer => "PHP (Composer)",
            Self::PhpLaravel => "PHP (Laravel)",
            Self::SwiftSpm => "Swift (SPM)",
            Self::SwiftXcode => "Swift (Xcode)",
            Self::Flutter => "Flutter",
            Self::ReactNative => "React Native",
            Self::Android => "Android",
            Self::Elixir => "Elixir",
            Self::Haskell => "Haskell",
            Self::OCaml => "OCaml",
            Self::Julia => "Julia",
            Self::R => "R",
            Self::Lua => "Lua",
            Self::Perl => "Perl",
            Self::Terraform => "Terraform",
            Self::Pulumi => "Pulumi",
            Self::Docker => "Docker",
            Self::Custom(_) => "Custom",
        }
    }

    /// Get the icon/emoji for this project kind
    pub fn icon(&self) -> &'static str {
        match self {
            Self::NodeNpm | Self::NodeYarn | Self::NodePnpm | Self::NodeBun | Self::Deno => "📦",
            Self::Rust => "🦀",
            Self::Go => "🐹",
            Self::Cpp | Self::C => "⚙️",
            Self::Zig => "⚡",
            Self::JavaMaven | Self::JavaGradle | Self::Kotlin | Self::Scala | Self::Clojure => "☕",
            Self::DotNet | Self::FSharp => "🔷",
            Self::PythonPip | Self::PythonPoetry | Self::PythonPipenv | Self::PythonConda | Self::PythonUv => "🐍",
            Self::RubyBundler | Self::RubyRails => "💎",
            Self::PhpComposer | Self::PhpLaravel => "🐘",
            Self::SwiftSpm | Self::SwiftXcode => "🍎",
            Self::Flutter => "🦋",
            Self::ReactNative => "⚛️",
            Self::Android => "🤖",
            Self::Elixir => "💧",
            Self::Haskell => "λ",
            Self::OCaml => "🐫",
            Self::Julia => "📊",
            Self::R => "📈",
            Self::Lua => "🌙",
            Self::Perl => "🐪",
            Self::Terraform | Self::Pulumi => "🏗️",
            Self::Docker => "🐳",
            Self::Custom(_) => "📁",
        }
    }
}

/// Marker files/directories that identify a project type
#[derive(Debug, Clone)]
pub struct ProjectMarker {
    /// What to look for
    pub indicator: MarkerKind,
    /// The project kind this marker identifies
    pub kind: ProjectKind,
    /// Priority when multiple markers match (higher = preferred)
    pub priority: u8,
}

/// Types of markers that can identify a project
#[derive(Debug, Clone)]
pub enum MarkerKind {
    /// Exact filename match (e.g., "package.json")
    File(&'static str),
    /// Exact directory name match (e.g., "node_modules")
    Directory(&'static str),
    /// File with specific extension
    Extension(&'static str),
    /// Multiple files (all must exist)
    AllOf(Vec<&'static str>),
    /// Multiple files (any must exist)
    AnyOf(Vec<&'static str>),
}

impl MarkerKind {
    /// Check if this marker matches at the given path
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::File(name) => path.join(name).is_file(),
            Self::Directory(name) => path.join(name).is_dir(),
            Self::Extension(ext) => {
                path.extension()
                    .map(|e| e.to_string_lossy().as_ref() == *ext)
                    .unwrap_or(false)
            }
            Self::AllOf(files) => files.iter().all(|f| path.join(f).exists()),
            Self::AnyOf(files) => files.iter().any(|f| path.join(f).exists()),
        }
    }
}

/// Git repository status for safety checks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitStatus {
    /// Whether this is a git repository
    pub is_repo: bool,
    /// Has uncommitted changes (modified/staged files)
    pub has_uncommitted: bool,
    /// Has untracked files
    pub has_untracked: bool,
    /// Has stashed changes
    pub has_stashed: bool,
    /// Current branch name
    pub branch: Option<String>,
    /// Remote URL (if any)
    pub remote: Option<String>,
    /// Last commit timestamp
    pub last_commit: Option<SystemTime>,
    /// Paths with uncommitted changes
    pub dirty_paths: Vec<PathBuf>,
}

impl GitStatus {
    /// Check if the repository is completely clean
    pub fn is_clean(&self) -> bool {
        self.is_repo && !self.has_uncommitted && !self.has_untracked
    }
}

/// Safety status for cleaning operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanSafety {
    /// Safe to clean without concerns
    Safe,
    /// Safe but with a warning
    Warning(CleanWarning),
    /// Blocked - should not clean without override
    Blocked(CleanBlock),
}

/// Warning types for cleaning operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanWarning {
    /// Project has uncommitted changes
    UncommittedChanges { paths: Vec<PathBuf> },
    /// Project has untracked files
    UntrackedFiles,
    /// Not a git repository (can't verify safety)
    NotGitRepo,
    /// Recently modified
    RecentlyModified { age_days: u32 },
    /// No lockfile found
    NoLockfile,
}

/// Blocking reasons for cleaning operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanBlock {
    /// Active lock file present (process using it)
    LockFilePresent(PathBuf),
    /// Process actively using the directory
    ProcessRunning { pid: u32, name: String },
    /// User explicitly protected this path
    UserProtected,
}

/// A detected development project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    pub id: ProjectId,
    /// Project ecosystem/type
    pub kind: ProjectKind,
    /// Root directory path
    pub root: PathBuf,
    /// Project name (usually directory name)
    pub name: String,
    /// Last modification time (skipped in serialization)
    #[serde(skip)]
    pub last_modified: Option<SystemTime>,
    /// Git status (if available, skipped in serialization)
    #[serde(skip)]
    pub git_status: Option<GitStatus>,
    /// Cleanable artifacts found
    pub artifacts: Vec<Artifact>,
    /// Total size of all artifacts
    pub total_size: u64,
    /// Size that can be cleaned
    pub cleanable_size: u64,
}

impl Project {
    /// Create a new project
    pub fn new(kind: ProjectKind, root: PathBuf) -> Self {
        let id = ProjectId::from_path(&root);
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());

        Self {
            id,
            kind,
            root,
            name,
            last_modified: None,
            git_status: None,
            artifacts: Vec::new(),
            total_size: 0,
            cleanable_size: 0,
        }
    }

    /// Check if it's safe to clean this project
    pub fn safety_check(&self) -> CleanSafety {
        // Check git status
        if let Some(status) = &self.git_status {
            if status.has_uncommitted {
                return CleanSafety::Warning(CleanWarning::UncommittedChanges {
                    paths: status.dirty_paths.clone(),
                });
            }
            if status.has_untracked {
                return CleanSafety::Warning(CleanWarning::UntrackedFiles);
            }
        } else {
            return CleanSafety::Warning(CleanWarning::NotGitRepo);
        }

        // Check age
        if let Some(modified) = self.last_modified {
            if let Ok(age) = modified.elapsed() {
                let days = age.as_secs() / 86400;
                if days < 7 {
                    return CleanSafety::Warning(CleanWarning::RecentlyModified {
                        age_days: days as u32,
                    });
                }
            }
        }

        CleanSafety::Safe
    }

    /// Get the number of artifacts
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }

    /// Calculate totals from artifacts
    pub fn calculate_totals(&mut self) {
        self.total_size = self.artifacts.iter().map(|a| a.size).sum();
        self.cleanable_size = self.total_size; // Can apply rules later
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}) - {}",
            self.kind.icon(),
            self.name,
            self.kind.display_name(),
            humansize::format_size(self.cleanable_size, humansize::BINARY)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_project_id_from_path() {
        let path1 = PathBuf::from("/home/user/project1");
        let path2 = PathBuf::from("/home/user/project2");

        let id1 = ProjectId::from_path(&path1);
        let id2 = ProjectId::from_path(&path2);
        let id1_again = ProjectId::from_path(&path1);

        assert_eq!(id1, id1_again);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_marker_kind_matches() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path();

        // Create a file
        std::fs::write(path.join("package.json"), "{}").unwrap();

        let marker = MarkerKind::File("package.json");
        assert!(marker.matches(path));

        let marker = MarkerKind::File("cargo.toml");
        assert!(!marker.matches(path));
    }

    #[test]
    fn test_project_safety_check() {
        let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));

        // No git status = warning
        assert!(matches!(
            project.safety_check(),
            CleanSafety::Warning(CleanWarning::NotGitRepo)
        ));

        // Clean git status = safe
        project.git_status = Some(GitStatus {
            is_repo: true,
            has_uncommitted: false,
            has_untracked: false,
            ..Default::default()
        });
        // Would be safe if not recently modified
    }
}
