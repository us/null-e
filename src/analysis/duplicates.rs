//! Duplicate dependency detection
//!
//! Finds duplicate dependencies across projects:
//! - Same npm package in multiple node_modules
//! - Multiple Python venvs with similar packages
//! - Duplicate cargo dependencies in different targets

use super::{Recommendation, RecommendationKind, RiskLevel};
use crate::cleaners::calculate_dir_size;
use crate::error::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Duplicate dependency finder
pub struct DuplicateFinder {
    /// Minimum size to report duplicates (bytes)
    pub min_duplicate_size: u64,
    /// Minimum number of duplicates to report
    pub min_duplicate_count: usize,
}

impl Default for DuplicateFinder {
    fn default() -> Self {
        Self {
            min_duplicate_size: 10_000_000, // 10MB total across duplicates
            min_duplicate_count: 2,
        }
    }
}

/// A detected duplicate
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// Package name
    pub name: String,
    /// Versions found
    pub versions: Vec<String>,
    /// Locations where this package is duplicated
    pub locations: Vec<PathBuf>,
    /// Total size across all duplicates
    pub total_size: u64,
    /// Potential savings if consolidated
    pub potential_savings: u64,
}

/// Package info extracted from package.json
#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
}

impl DuplicateFinder {
    /// Create a new duplicate finder
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan for duplicate dependencies
    pub fn scan(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Find node_modules duplicates
        recommendations.extend(self.find_node_duplicates(root, max_depth)?);

        // Find Python venv duplicates
        recommendations.extend(self.find_python_duplicates(root, max_depth)?);

        // Find Rust target duplicates
        recommendations.extend(self.find_rust_duplicates(root, max_depth)?);

        Ok(recommendations)
    }

    /// Find duplicate packages in node_modules
    fn find_node_duplicates(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let mut package_locations: HashMap<String, Vec<(PathBuf, String, u64)>> = HashMap::new();

        // Find all node_modules directories
        for entry in WalkDir::new(root)
            .max_depth(max_depth + 5) // Go deeper to find nested node_modules
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != ".git" && name != "target" && name != "venv"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Look for package.json files in node_modules
            if path.is_file() && path.file_name().map(|n| n == "package.json").unwrap_or(false) {
                // Check if inside node_modules
                let path_str = path.to_string_lossy();
                if !path_str.contains("node_modules") {
                    continue;
                }

                // Read package.json
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                        if let (Some(name), Some(version)) = (pkg.name, pkg.version) {
                            // Skip scoped packages that are likely unique
                            if name.starts_with('@') && name.contains('/') {
                                continue;
                            }

                            // Get package directory
                            if let Some(pkg_dir) = path.parent() {
                                let size = calculate_dir_size(pkg_dir)
                                    .map(|(s, _)| s)
                                    .unwrap_or(0);

                                package_locations
                                    .entry(name)
                                    .or_default()
                                    .push((pkg_dir.to_path_buf(), version, size));
                            }
                        }
                    }
                }
            }
        }

        // Find packages that appear multiple times
        let mut recommendations = Vec::new();

        for (name, locations) in package_locations {
            if locations.len() < self.min_duplicate_count {
                continue;
            }

            let total_size: u64 = locations.iter().map(|(_, _, s)| s).sum();

            if total_size < self.min_duplicate_size {
                continue;
            }

            // Group by version
            let mut version_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for (path, version, _) in &locations {
                version_map
                    .entry(version.clone())
                    .or_default()
                    .push(path.clone());
            }

            let versions: Vec<String> = version_map.keys().cloned().collect();
            let unique_versions = versions.len();

            // Potential savings: keep only one copy of each version
            let potential_savings = if unique_versions == 1 {
                // Same version everywhere - could theoretically use symlinks
                total_size - locations.first().map(|(_, _, s)| *s).unwrap_or(0)
            } else {
                // Different versions - harder to dedupe
                0
            };

            if potential_savings < 1_000_000 {
                continue;
            }

            let locations_paths: Vec<PathBuf> = locations.iter().map(|(p, _, _)| p.clone()).collect();

            recommendations.push(Recommendation {
                kind: RecommendationKind::DuplicateDependency,
                title: format!(
                    "📦 {} ({} copies, {})",
                    name,
                    locations.len(),
                    format_size(total_size)
                ),
                description: if unique_versions == 1 {
                    format!(
                        "Same version ({}) installed {} times. Could save {} with deduplication.",
                        versions.first().unwrap_or(&"?".to_string()),
                        locations.len(),
                        format_size(potential_savings)
                    )
                } else {
                    format!(
                        "{} different versions across {} installations. Consider using pnpm or yarn workspaces.",
                        unique_versions,
                        locations.len()
                    )
                },
                path: locations_paths.first().cloned().unwrap_or_default(),
                potential_savings,
                fix_command: Some("Consider using pnpm or yarn workspaces for deduplication".to_string()),
                risk: RiskLevel::Low,
            });
        }

        // Sort by potential savings
        recommendations.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));

        // Limit to top 20
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// Find duplicate Python venvs
    fn find_python_duplicates(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let mut venvs: Vec<(PathBuf, u64)> = Vec::new();

        // Find all venv/virtualenv directories
        for entry in WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != ".git" && name != "node_modules" && name != "target"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

            // Check for venv markers
            if path.is_dir() && (name == "venv" || name == ".venv" || name == "env") {
                let pyvenv_cfg = path.join("pyvenv.cfg");
                if pyvenv_cfg.exists() {
                    let size = calculate_dir_size(path).map(|(s, _)| s).unwrap_or(0);
                    venvs.push((path.to_path_buf(), size));
                }
            }
        }

        let mut recommendations = Vec::new();

        if venvs.len() >= 3 {
            let total_size: u64 = venvs.iter().map(|(_, s)| s).sum();
            let avg_size = total_size / venvs.len() as u64;

            // Many venvs often have duplicate packages
            // Estimate ~40% could be shared
            let potential_savings = (total_size as f64 * 0.4) as u64;

            if total_size > 500_000_000 {
                recommendations.push(Recommendation {
                    kind: RecommendationKind::DuplicateDependency,
                    title: format!(
                        "🐍 {} Python venvs ({})",
                        venvs.len(),
                        format_size(total_size)
                    ),
                    description: format!(
                        "Found {} virtual environments averaging {}. Consider using uv, poetry, or conda for better dependency management.",
                        venvs.len(),
                        format_size(avg_size)
                    ),
                    path: venvs.first().map(|(p, _)| p.clone()).unwrap_or_default(),
                    potential_savings,
                    fix_command: Some("Consider using uv or poetry with centralized cache".to_string()),
                    risk: RiskLevel::Low,
                });
            }
        }

        Ok(recommendations)
    }

    /// Find duplicate Rust target directories
    fn find_rust_duplicates(&self, root: &Path, max_depth: usize) -> Result<Vec<Recommendation>> {
        let mut targets: Vec<(PathBuf, u64)> = Vec::new();

        // Find all Rust target directories
        for entry in WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != ".git" && name != "node_modules" && name != "venv"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check for target with Cargo.toml sibling
            if path.is_dir() && path.file_name().map(|n| n == "target").unwrap_or(false) {
                if let Some(parent) = path.parent() {
                    if parent.join("Cargo.toml").exists() {
                        let size = calculate_dir_size(path).map(|(s, _)| s).unwrap_or(0);
                        if size > 50_000_000 {
                            targets.push((path.to_path_buf(), size));
                        }
                    }
                }
            }
        }

        let mut recommendations = Vec::new();

        if targets.len() >= 2 {
            let total_size: u64 = targets.iter().map(|(_, s)| s).sum();

            // Rust target directories often share compiled dependencies
            // With a shared target directory, could save ~30-50%
            let potential_savings = (total_size as f64 * 0.35) as u64;

            if total_size > 1_000_000_000 {
                recommendations.push(Recommendation {
                    kind: RecommendationKind::DuplicateDependency,
                    title: format!(
                        "🦀 {} Rust targets ({})",
                        targets.len(),
                        format_size(total_size)
                    ),
                    description: format!(
                        "Found {} Rust projects with separate target directories. Consider using CARGO_TARGET_DIR for shared compilation cache.",
                        targets.len()
                    ),
                    path: targets.first().map(|(p, _)| p.clone()).unwrap_or_default(),
                    potential_savings,
                    fix_command: Some("export CARGO_TARGET_DIR=~/.cargo/target".to_string()),
                    risk: RiskLevel::None,
                });
            }
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
    fn test_duplicate_finder_creation() {
        let finder = DuplicateFinder::new();
        assert_eq!(finder.min_duplicate_count, 2);
    }

    #[test]
    fn test_duplicate_scan() {
        let finder = DuplicateFinder::new();
        if let Ok(recommendations) = finder.scan(Path::new("."), 3) {
            println!("Found {} duplicate recommendations", recommendations.len());
            for rec in &recommendations {
                println!("  {} - {}", rec.title, rec.description);
            }
        }
    }
}
