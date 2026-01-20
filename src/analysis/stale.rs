//! Stale project detection
//!
//! Finds development projects that haven't been touched in a long time.
//! These are candidates for archiving or cleanup.

use super::{Recommendation, RecommendationKind, RiskLevel};
use crate::cleaners::calculate_dir_size;
use crate::error::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

/// Stale project detector
pub struct StaleProjectFinder {
    /// Minimum days since last activity to consider stale
    pub stale_threshold_days: u64,
    /// Minimum project size to report (bytes)
    pub min_project_size: u64,
}

impl Default for StaleProjectFinder {
    fn default() -> Self {
        Self {
            stale_threshold_days: 180, // 6 months
            min_project_size: 100_000_000, // 100MB
        }
    }
}

/// Project type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Node,
    Rust,
    Python,
    Go,
    Java,
    Swift,
    Ruby,
    Unknown,
}

impl ProjectType {
    /// Get icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Node => "📦",
            Self::Rust => "🦀",
            Self::Python => "🐍",
            Self::Go => "🐹",
            Self::Java => "☕",
            Self::Swift => "🍎",
            Self::Ruby => "💎",
            Self::Unknown => "📁",
        }
    }

    /// Get name for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Node => "Node.js",
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::Swift => "Swift",
            Self::Ruby => "Ruby",
            Self::Unknown => "Unknown",
        }
    }

    /// Get cleanable directories for this project type
    pub fn cleanable_dirs(&self) -> &[&str] {
        match self {
            Self::Node => &["node_modules", ".next", "dist", "build"],
            Self::Rust => &["target"],
            Self::Python => &["venv", ".venv", "__pycache__", ".pytest_cache"],
            Self::Go => &[],
            Self::Java => &["target", "build", ".gradle"],
            Self::Swift => &["DerivedData", ".build", "Pods"],
            Self::Ruby => &["vendor/bundle", ".bundle"],
            Self::Unknown => &[],
        }
    }
}

/// Information about a stale project
#[derive(Debug, Clone)]
pub struct StaleProject {
    /// Project root path
    pub path: PathBuf,
    /// Detected project type
    pub project_type: ProjectType,
    /// Days since last activity
    pub days_stale: u64,
    /// Total project size
    pub total_size: u64,
    /// Size of cleanable artifacts
    pub cleanable_size: u64,
    /// Last activity date
    pub last_activity: Option<String>,
}

impl StaleProjectFinder {
    /// Create a new stale project finder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom threshold
    pub fn with_threshold(days: u64) -> Self {
        Self {
            stale_threshold_days: days,
            ..Self::default()
        }
    }

    /// Scan for stale projects
    pub fn scan(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let projects = self.find_projects(root, max_depth)?;

        let recommendations: Vec<Recommendation> = projects
            .par_iter()
            .filter_map(|project_path| self.analyze_project(project_path).ok())
            .flatten()
            .collect();

        Ok(recommendations)
    }

    /// Find all project directories
    fn find_projects(&self, root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
        let mut projects = Vec::new();

        // Project markers to look for
        let markers = [
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "setup.py",
            "requirements.txt",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "Package.swift",
            "Gemfile",
        ];

        for entry in WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip common non-project directories
                name != "node_modules" &&
                name != ".cargo" &&
                name != "target" &&
                name != "venv" &&
                name != ".venv" &&
                name != ".git" &&
                name != "vendor"
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if markers.iter().any(|m| *m == name) {
                    if let Some(parent) = entry.path().parent() {
                        if !projects.contains(&parent.to_path_buf()) {
                            projects.push(parent.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(projects)
    }

    /// Analyze a single project
    fn analyze_project(&self, project_path: &Path) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Detect project type
        let project_type = self.detect_project_type(project_path);

        // Get last activity (from git or file system)
        let (days_stale, last_activity) = self.get_last_activity(project_path);

        // Skip if not stale enough
        if days_stale < self.stale_threshold_days {
            return Ok(recommendations);
        }

        // Calculate project size
        let (total_size, _) = calculate_dir_size(project_path)?;

        if total_size < self.min_project_size {
            return Ok(recommendations);
        }

        // Calculate cleanable size (build artifacts)
        let cleanable_size = self.calculate_cleanable_size(project_path, project_type);

        let project_name = project_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let months = days_stale / 30;
        let time_desc = if months >= 12 {
            format!("{} years", months / 12)
        } else {
            format!("{} months", months)
        };

        recommendations.push(Recommendation {
            kind: RecommendationKind::StaleProject,
            title: format!(
                "{} {} ({}) - {} stale",
                project_type.icon(),
                project_name,
                format_size(total_size),
                time_desc
            ),
            description: if cleanable_size > 0 {
                format!(
                    "{} project not touched in {} days. {} in build artifacts can be cleaned.",
                    project_type.name(),
                    days_stale,
                    format_size(cleanable_size)
                )
            } else {
                format!(
                    "{} project not touched in {} days. Consider archiving or deleting.",
                    project_type.name(),
                    days_stale
                )
            },
            path: project_path.to_path_buf(),
            potential_savings: cleanable_size,
            fix_command: if cleanable_size > 0 {
                Some(self.get_clean_command(project_type, project_path))
            } else {
                None
            },
            risk: if cleanable_size > 0 {
                RiskLevel::Low
            } else {
                RiskLevel::High
            },
        });

        Ok(recommendations)
    }

    /// Detect project type from files
    fn detect_project_type(&self, path: &Path) -> ProjectType {
        if path.join("package.json").exists() {
            ProjectType::Node
        } else if path.join("Cargo.toml").exists() {
            ProjectType::Rust
        } else if path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("requirements.txt").exists()
        {
            ProjectType::Python
        } else if path.join("go.mod").exists() {
            ProjectType::Go
        } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            ProjectType::Java
        } else if path.join("Package.swift").exists() {
            ProjectType::Swift
        } else if path.join("Gemfile").exists() {
            ProjectType::Ruby
        } else {
            ProjectType::Unknown
        }
    }

    /// Get last activity date (prefer git, fall back to file mtime)
    fn get_last_activity(&self, path: &Path) -> (u64, Option<String>) {
        // Try git first
        if path.join(".git").exists() {
            if let Some((days, date)) = self.get_git_last_commit(path) {
                return (days, Some(date));
            }
        }

        // Fall back to file system mtime
        self.get_filesystem_mtime(path)
    }

    /// Get days since last git commit
    fn get_git_last_commit(&self, path: &Path) -> Option<(u64, String)> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%ct"])
            .current_dir(path)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout);
        let timestamp: i64 = timestamp_str.trim().parse().ok()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;

        let days = ((now - timestamp) / 86400) as u64;

        // Get formatted date
        let date_output = Command::new("git")
            .args(["log", "-1", "--format=%ci"])
            .current_dir(path)
            .output()
            .ok()?;

        let date = String::from_utf8_lossy(&date_output.stdout)
            .trim()
            .to_string();

        Some((days, date))
    }

    /// Get days since last file modification
    fn get_filesystem_mtime(&self, path: &Path) -> (u64, Option<String>) {
        let mut latest_mtime: Option<SystemTime> = None;

        // Check key files for mtime
        let key_files = ["package.json", "Cargo.toml", "pyproject.toml", "go.mod"];

        for file in key_files {
            let file_path = path.join(file);
            if let Ok(meta) = std::fs::metadata(&file_path) {
                if let Ok(mtime) = meta.modified() {
                    if latest_mtime.is_none() || mtime > latest_mtime.unwrap() {
                        latest_mtime = Some(mtime);
                    }
                }
            }
        }

        if let Some(mtime) = latest_mtime {
            if let Ok(duration) = mtime.elapsed() {
                let days = duration.as_secs() / 86400;
                return (days, None);
            }
        }

        (0, None)
    }

    /// Calculate size of cleanable artifacts
    fn calculate_cleanable_size(&self, path: &Path, project_type: ProjectType) -> u64 {
        let mut total = 0u64;

        for dir_name in project_type.cleanable_dirs() {
            let dir_path = path.join(dir_name);
            if dir_path.exists() {
                if let Ok((size, _)) = calculate_dir_size(&dir_path) {
                    total += size;
                }
            }
        }

        total
    }

    /// Get command to clean project artifacts
    fn get_clean_command(&self, project_type: ProjectType, path: &Path) -> String {
        let path_str = path.to_string_lossy();
        match project_type {
            ProjectType::Node => format!("rm -rf {}/node_modules", path_str),
            ProjectType::Rust => format!("cargo clean --manifest-path {}/Cargo.toml", path_str),
            ProjectType::Python => format!("rm -rf {}/.venv {}/venv", path_str, path_str),
            ProjectType::Java => format!("rm -rf {}/target {}/build", path_str, path_str),
            ProjectType::Swift => format!("rm -rf {}/.build {}/DerivedData", path_str, path_str),
            ProjectType::Ruby => format!("rm -rf {}/vendor/bundle", path_str),
            _ => format!("# No automatic cleanup for {}", path_str),
        }
    }
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    super::format_size(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stale_finder_creation() {
        let finder = StaleProjectFinder::new();
        assert_eq!(finder.stale_threshold_days, 180);
    }

    #[test]
    fn test_project_type_detection() {
        let finder = StaleProjectFinder::new();
        // Test on current directory if it's a project
        let project_type = finder.detect_project_type(Path::new("."));
        println!("Detected project type: {:?}", project_type);
    }

    #[test]
    fn test_stale_scan() {
        let finder = StaleProjectFinder::with_threshold(30); // 30 days for testing
        if let Ok(recommendations) = finder.scan(Path::new("."), 2) {
            println!("Found {} stale project recommendations", recommendations.len());
            for rec in &recommendations {
                println!("  {} - {}", rec.title, rec.description);
            }
        }
    }
}
