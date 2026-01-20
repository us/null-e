//! Git repository analysis and optimization
//!
//! Detects:
//! - Large .git directories that could benefit from `git gc`
//! - Loose objects that need packing
//! - Git LFS cache
//! - Large files in history

use super::{Recommendation, RecommendationKind, RiskLevel};
use crate::cleaners::calculate_dir_size;
use crate::error::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Git repository analyzer
pub struct GitAnalyzer {
    /// Minimum .git size to report (default 100MB)
    pub min_git_size: u64,
    /// Minimum loose objects to suggest gc
    pub min_loose_objects: usize,
}

impl Default for GitAnalyzer {
    fn default() -> Self {
        Self {
            min_git_size: 100_000_000, // 100MB
            min_loose_objects: 1000,
        }
    }
}

/// Information about a git repository
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    /// Path to the repository root
    pub path: PathBuf,
    /// Size of .git directory
    pub git_size: u64,
    /// Number of loose objects
    pub loose_objects: usize,
    /// Number of pack files
    pub pack_count: usize,
    /// Size of pack files
    pub pack_size: u64,
    /// Whether gc would help
    pub gc_recommended: bool,
    /// Estimated savings from gc
    pub estimated_savings: u64,
    /// Last commit date (if available)
    pub last_commit: Option<String>,
}

impl GitAnalyzer {
    /// Create a new git analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a directory for git repositories and analyze them
    pub fn scan(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let repos = self.find_git_repos(root, max_depth)?;

        let recommendations: Vec<Recommendation> = repos
            .par_iter()
            .filter_map(|repo_path| self.analyze_repo(repo_path).ok())
            .flatten()
            .collect();

        Ok(recommendations)
    }

    /// Find all .git directories under root
    fn find_git_repos(&self, root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
        let mut repos = Vec::new();

        for entry in WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip common non-project directories
            let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            if name == "node_modules" || name == ".cargo" || name == "target" || name == "venv" {
                continue;
            }

            if path.is_dir() && path.file_name().map(|n| n == ".git").unwrap_or(false) {
                if let Some(parent) = path.parent() {
                    repos.push(parent.to_path_buf());
                }
            }
        }

        Ok(repos)
    }

    /// Analyze a single git repository
    fn analyze_repo(&self, repo_path: &Path) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        let git_dir = repo_path.join(".git");

        if !git_dir.exists() {
            return Ok(recommendations);
        }

        // Calculate .git size
        let (git_size, _) = calculate_dir_size(&git_dir)?;

        if git_size < self.min_git_size {
            return Ok(recommendations);
        }

        // Count loose objects
        let objects_dir = git_dir.join("objects");
        let (loose_count, loose_size) = self.count_loose_objects(&objects_dir);

        // Count pack files
        let pack_dir = objects_dir.join("pack");
        let (pack_count, pack_size) = self.count_packs(&pack_dir);

        // Check if gc would help
        let gc_recommended = loose_count > self.min_loose_objects ||
                            (loose_size > 50_000_000 && loose_count > 500);

        // Estimate savings (loose objects can usually be compressed 50-80%)
        let estimated_savings = if gc_recommended {
            (loose_size as f64 * 0.6) as u64
        } else {
            0
        };

        // Get last commit info
        let last_commit = self.get_last_commit_date(repo_path);

        let info = GitRepoInfo {
            path: repo_path.to_path_buf(),
            git_size,
            loose_objects: loose_count,
            pack_count,
            pack_size,
            gc_recommended,
            estimated_savings,
            last_commit: last_commit.clone(),
        };

        // Create recommendation for large .git
        if git_size > self.min_git_size {
            let title = format!(
                "Large .git: {} ({})",
                repo_path.file_name().unwrap_or_default().to_string_lossy(),
                format_size(git_size)
            );

            let description = if gc_recommended {
                format!(
                    "{} loose objects ({} bytes). Running 'git gc' could save ~{}.",
                    loose_count,
                    format_size(loose_size),
                    format_size(estimated_savings)
                )
            } else {
                format!(
                    "Large repository with {} pack files. Already well-packed.",
                    pack_count
                )
            };

            let fix_command = if gc_recommended {
                Some(format!("cd {:?} && git gc --aggressive --prune=now", repo_path))
            } else {
                None
            };

            recommendations.push(Recommendation {
                kind: RecommendationKind::GitOptimization,
                title,
                description,
                path: repo_path.to_path_buf(),
                potential_savings: estimated_savings,
                fix_command,
                risk: RiskLevel::None,
            });
        }

        Ok(recommendations)
    }

    /// Count loose objects in objects directory
    fn count_loose_objects(&self, objects_dir: &Path) -> (usize, u64) {
        let mut count = 0;
        let mut size = 0u64;

        // Loose objects are in objects/XX/YYYYYYYY... subdirectories
        if let Ok(entries) = std::fs::read_dir(objects_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip pack and info directories
                if name_str == "pack" || name_str == "info" {
                    continue;
                }

                // Two-character hex directories contain loose objects
                if name_str.len() == 2 && name_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    if let Ok(subentries) = std::fs::read_dir(entry.path()) {
                        for subentry in subentries.filter_map(|e| e.ok()) {
                            count += 1;
                            if let Ok(meta) = subentry.metadata() {
                                size += meta.len();
                            }
                        }
                    }
                }
            }
        }

        (count, size)
    }

    /// Count pack files
    fn count_packs(&self, pack_dir: &Path) -> (usize, u64) {
        let mut count = 0;
        let mut size = 0u64;

        if let Ok(entries) = std::fs::read_dir(pack_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                if name.to_string_lossy().ends_with(".pack") {
                    count += 1;
                    if let Ok(meta) = entry.metadata() {
                        size += meta.len();
                    }
                }
            }
        }

        (count, size)
    }

    /// Get last commit date
    fn get_last_commit_date(&self, repo_path: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%ci"])
            .current_dir(repo_path)
            .output()
            .ok()?;

        if output.status.success() {
            let date = String::from_utf8_lossy(&output.stdout);
            Some(date.trim().to_string())
        } else {
            None
        }
    }

    /// Detect Git LFS cache
    pub fn detect_lfs_cache(&self) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        let home = dirs::home_dir().unwrap_or_default();

        // Git LFS cache locations
        let lfs_paths = [
            home.join(".cache/git-lfs"),
            home.join("Library/Caches/git-lfs"), // macOS
        ];

        for lfs_path in lfs_paths {
            if !lfs_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&lfs_path)?;
            if size < 100_000_000 {
                continue;
            }

            recommendations.push(Recommendation {
                kind: RecommendationKind::GitLfsCache,
                title: format!("Git LFS Cache ({})", format_size(size)),
                description: format!(
                    "Git LFS cached files ({} files). Can be pruned if not actively using LFS.",
                    file_count
                ),
                path: lfs_path,
                potential_savings: size,
                fix_command: Some("git lfs prune".to_string()),
                risk: RiskLevel::Low,
            });
        }

        Ok(recommendations)
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
    fn test_git_analyzer_creation() {
        let analyzer = GitAnalyzer::new();
        assert_eq!(analyzer.min_git_size, 100_000_000);
    }

    #[test]
    fn test_git_scan() {
        let analyzer = GitAnalyzer::new();
        // Scan current directory with low depth
        if let Ok(recommendations) = analyzer.scan(Path::new("."), 3) {
            println!("Found {} git recommendations", recommendations.len());
            for rec in &recommendations {
                println!("  {} - {}", rec.title, rec.description);
            }
        }
    }
}
