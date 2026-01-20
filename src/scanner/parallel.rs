//! Parallel filesystem scanner implementation
//!
//! Uses jwalk for parallel directory traversal and rayon for parallel processing.

use crate::core::{
    ArtifactStats, Project, ProjectId, ScanConfig, ScanError, ScanProgress, ScanResult, Scanner,
};
use crate::error::{DevSweepError, Result};
use crate::plugins::PluginRegistry;
use dashmap::DashMap;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

/// High-performance parallel scanner
pub struct ParallelScanner {
    registry: Arc<PluginRegistry>,
    progress: Arc<ScanProgress>,
}

impl ParallelScanner {
    /// Create a new parallel scanner
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            progress: ScanProgress::new(),
        }
    }

    /// Scan a single root directory
    fn scan_root(&self, root: &Path, projects: &DashMap<ProjectId, Project>, config: &ScanConfig) -> Result<()> {
        let walker = WalkDir::new(root)
            .max_depth(config.max_depth.unwrap_or(usize::MAX))
            .follow_links(false);

        let skip_hidden = config.skip_hidden;

        // Directories to skip (artifact directories that contain nested packages)
        let skip_dirs: std::collections::HashSet<&str> = [
            "node_modules",
            "target",
            ".venv",
            "venv",
            "__pycache__",
            "vendor",
            "build",
            ".gradle",
            "bin",
            "obj",
            ".build",
            "Pods",
            "DerivedData",
            ".next",
            ".nuxt",
            "dist",
            ".cache",
            ".turbo",
            "coverage",
        ].into_iter().collect();

        let entries = walker.into_iter().filter_entry(move |e| {
            let name = e.file_name().to_str().unwrap_or("");

            // Skip artifact directories (they contain nested packages we don't want)
            if e.depth() > 0 && skip_dirs.contains(name) {
                return false;
            }

            // Skip hidden directories if configured
            if skip_hidden && e.depth() > 0 && name.starts_with('.') {
                // But allow some important hidden dirs
                let allowed_hidden = [".git", ".github", ".vscode", ".idea"];
                if !allowed_hidden.contains(&name) {
                    return false;
                }
            }

            true
        });

        for entry in entries {
            // Check for cancellation
            if self.progress.is_cancelled() {
                return Err(DevSweepError::ScanInterrupted);
            }

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    self.progress.add_error(ScanError::new(
                        PathBuf::new(),
                        format!("Walk error: {}", e),
                    ));
                    continue;
                }
            };

            // Only process directories
            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            self.progress.inc_directories();
            self.progress.set_current_path(path.to_path_buf());

            // Skip if already found as a project or inside a project
            let project_id = ProjectId::from_path(path);
            if projects.contains_key(&project_id) {
                continue;
            }

            // Try to detect project type
            if let Some((kind, plugin)) = self.registry.detect_project(path) {
                // Found a project!
                let mut project = Project::new(kind, path.to_path_buf());

                // Find artifacts
                match plugin.find_artifacts(path) {
                    Ok(mut artifacts) => {
                        // Calculate sizes in parallel
                        artifacts.par_iter_mut().for_each(|artifact| {
                            if let Ok(size) = plugin.calculate_size(artifact) {
                                artifact.size = size;
                            }
                            if let Ok(count) = crate::plugins::count_files(&artifact.path) {
                                artifact.file_count = count;
                            }
                        });

                        // Filter by minimum size if specified
                        if let Some(min_size) = config.min_size {
                            artifacts.retain(|a| a.size >= min_size);
                        }

                        // Skip if no meaningful artifacts
                        if artifacts.is_empty() {
                            continue;
                        }

                        project.artifacts = artifacts;
                        project.calculate_totals();

                        // Get last modified time
                        if let Ok(meta) = std::fs::metadata(path) {
                            project.last_modified = meta.modified().ok();
                        }

                        self.progress.inc_projects();
                        self.progress.add_size(project.cleanable_size);

                        projects.insert(project_id, project);
                    }
                    Err(e) => {
                        self.progress.add_error(ScanError::new(
                            path.to_path_buf(),
                            format!("Failed to find artifacts: {}", e),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Scanner for ParallelScanner {
    fn scan(&self, config: &ScanConfig) -> Result<ScanResult> {
        let start = Instant::now();

        // Validate roots
        if config.roots.is_empty() {
            return Err(DevSweepError::Config("No scan roots specified".into()));
        }

        for root in &config.roots {
            if !root.exists() {
                return Err(DevSweepError::PathNotFound(root.clone()));
            }
            if !root.is_dir() {
                return Err(DevSweepError::NotADirectory(root.clone()));
            }
        }

        // Configure thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.parallelism.unwrap_or(num_cpus::get()))
            .build()
            .map_err(|e| DevSweepError::Scanner(format!("Thread pool error: {}", e)))?;

        // Concurrent project map
        let projects: DashMap<ProjectId, Project> = DashMap::new();

        // Scan each root
        pool.install(|| {
            config.roots.par_iter().for_each(|root| {
                if let Err(e) = self.scan_root(root, &projects, config) {
                    if !matches!(e, DevSweepError::ScanInterrupted) {
                        self.progress.add_error(ScanError::new(
                            root.clone(),
                            e.to_string(),
                        ));
                    }
                }
            });
        });

        self.progress.mark_complete();

        // Check if scan was cancelled
        if self.progress.is_cancelled() {
            return Err(DevSweepError::ScanInterrupted);
        }

        // Collect and sort results
        let mut results: Vec<Project> = projects.into_iter().map(|(_, p)| p).collect();
        results.sort_by(|a, b| b.cleanable_size.cmp(&a.cleanable_size));

        // Apply limit if specified
        if let Some(limit) = config.limit {
            results.truncate(limit);
        }

        // Calculate statistics
        let mut stats = ArtifactStats::default();
        for project in &results {
            for artifact in &project.artifacts {
                stats.add(artifact);
            }
        }

        let total_size: u64 = results.iter().map(|p| p.total_size).sum();
        let total_cleanable: u64 = results.iter().map(|p| p.cleanable_size).sum();

        Ok(ScanResult {
            projects: results,
            total_size,
            total_cleanable,
            duration: start.elapsed(),
            directories_scanned: self.progress.directories_scanned.load(Ordering::Relaxed),
            errors: std::mem::take(&mut *self.progress.errors.lock()),
            stats,
        })
    }

    fn progress(&self) -> Arc<ScanProgress> {
        Arc::clone(&self.progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_node_project(path: &Path) {
        std::fs::write(path.join("package.json"), r#"{"name": "test"}"#).unwrap();
        std::fs::create_dir(path.join("node_modules")).unwrap();
        std::fs::write(path.join("node_modules/.package-lock.json"), "{}").unwrap();
    }

    fn setup_rust_project(path: &Path) {
        std::fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(path.join("target/debug")).unwrap();
        std::fs::write(path.join("target/debug/test"), "binary").unwrap();
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        let config = ScanConfig::new(temp.path());
        let result = scanner.scan(&config).unwrap();

        assert_eq!(result.projects.len(), 0);
        assert_eq!(result.total_cleanable, 0);
    }

    #[test]
    fn test_scan_node_project() {
        let temp = TempDir::new().unwrap();
        setup_node_project(temp.path());

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        let config = ScanConfig::new(temp.path());
        let result = scanner.scan(&config).unwrap();

        assert_eq!(result.projects.len(), 1);
        assert!(result.projects[0].artifacts.iter().any(|a| a.name() == "node_modules"));
    }

    #[test]
    fn test_scan_rust_project() {
        let temp = TempDir::new().unwrap();
        setup_rust_project(temp.path());

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        let config = ScanConfig::new(temp.path());
        let result = scanner.scan(&config).unwrap();

        assert_eq!(result.projects.len(), 1);
        assert!(result.projects[0].artifacts.iter().any(|a| a.name() == "target"));
    }

    #[test]
    fn test_scan_multiple_projects() {
        let temp = TempDir::new().unwrap();

        // Create Node project
        let node_proj = temp.path().join("node-app");
        std::fs::create_dir(&node_proj).unwrap();
        setup_node_project(&node_proj);

        // Create Rust project
        let rust_proj = temp.path().join("rust-app");
        std::fs::create_dir(&rust_proj).unwrap();
        setup_rust_project(&rust_proj);

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        let config = ScanConfig::new(temp.path());
        let result = scanner.scan(&config).unwrap();

        assert_eq!(result.projects.len(), 2);
    }

    #[test]
    fn test_scan_with_min_size_filter() {
        let temp = TempDir::new().unwrap();
        setup_node_project(temp.path());

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        // Set min size very high
        let config = ScanConfig::new(temp.path()).with_min_size(1_000_000_000);
        let result = scanner.scan(&config).unwrap();

        // Should find no projects since artifacts are small
        assert_eq!(result.projects.len(), 0);
    }

    #[test]
    fn test_scan_with_max_depth() {
        let temp = TempDir::new().unwrap();

        // Create nested project
        let deep = temp.path().join("a/b/c/d/e");
        std::fs::create_dir_all(&deep).unwrap();
        setup_node_project(&deep);

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        // Limit depth to 3
        let config = ScanConfig::new(temp.path()).with_max_depth(3);
        let result = scanner.scan(&config).unwrap();

        // Should not find the deeply nested project
        assert_eq!(result.projects.len(), 0);
    }

    #[test]
    fn test_scan_cancellation() {
        let temp = TempDir::new().unwrap();
        setup_node_project(temp.path());

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        // Cancel immediately
        scanner.cancel();

        let config = ScanConfig::new(temp.path());
        let result = scanner.scan(&config);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevSweepError::ScanInterrupted
        ));
    }

    #[test]
    fn test_progress_tracking() {
        let temp = TempDir::new().unwrap();
        setup_node_project(temp.path());

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);
        let progress = scanner.progress();

        let config = ScanConfig::new(temp.path());
        let _ = scanner.scan(&config);

        let snapshot = progress.snapshot();
        assert!(snapshot.is_complete);
        assert!(snapshot.directories_scanned > 0);
    }
}
