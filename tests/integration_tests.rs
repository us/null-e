//! Integration tests for null-e

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

// Import from the library
use null_e::prelude::*;
use null_e::plugins::*;

/// Helper to create a mock Node.js project
fn create_node_project(path: &std::path::Path) {
    std::fs::write(path.join("package.json"), r#"{"name": "test-project"}"#).unwrap();
    std::fs::create_dir(path.join("node_modules")).unwrap();
    std::fs::write(
        path.join("node_modules/.package-lock.json"),
        r#"{"lockfileVersion": 3}"#,
    )
    .unwrap();
    // Create some fake dependencies
    std::fs::create_dir_all(path.join("node_modules/lodash")).unwrap();
    std::fs::write(
        path.join("node_modules/lodash/index.js"),
        "module.exports = {};",
    )
    .unwrap();
}

/// Helper to create a mock Rust project
fn create_rust_project(path: &std::path::Path) {
    std::fs::write(
        path.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    std::fs::create_dir_all(path.join("target/debug")).unwrap();
    std::fs::write(path.join("target/debug/test-project"), "binary content").unwrap();
    std::fs::create_dir_all(path.join("target/release")).unwrap();
    std::fs::write(path.join("target/release/test-project"), "binary content").unwrap();
}

/// Helper to create a mock Python project
fn create_python_project(path: &std::path::Path) {
    std::fs::write(path.join("requirements.txt"), "flask==2.0.0\nrequests\n").unwrap();

    // Create a venv
    let venv = path.join(".venv");
    std::fs::create_dir(&venv).unwrap();
    std::fs::write(venv.join("pyvenv.cfg"), "home = /usr/bin\ninclude-system-site-packages = false\n").unwrap();

    // Create __pycache__
    std::fs::create_dir(path.join("__pycache__")).unwrap();
    std::fs::write(path.join("__pycache__/module.cpython-310.pyc"), "bytecode").unwrap();
}

// ============================================================================
// Plugin Registry Tests
// ============================================================================

#[test]
fn test_plugin_registry_has_builtins() {
    let registry = PluginRegistry::with_builtins();

    assert!(!registry.is_empty());
    assert!(registry.len() >= 6); // At least Node, Rust, Python, Go, Maven, Gradle

    // Check specific plugins exist
    assert!(registry.get_by_id("node").is_some());
    assert!(registry.get_by_id("rust").is_some());
    assert!(registry.get_by_id("python").is_some());
}

#[test]
fn test_plugin_registry_cleanable_dirs() {
    let registry = PluginRegistry::with_builtins();

    assert!(registry.is_cleanable_dir("node_modules"));
    assert!(registry.is_cleanable_dir("target"));
    assert!(registry.is_cleanable_dir("__pycache__"));
    assert!(registry.is_cleanable_dir(".venv"));

    // src is not cleanable
    assert!(!registry.is_cleanable_dir("src"));
    assert!(!registry.is_cleanable_dir("lib"));
}

// ============================================================================
// Scanner Tests
// ============================================================================

#[test]
fn test_scanner_empty_directory() {
    let temp = TempDir::new().unwrap();
    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config).unwrap();

    assert_eq!(result.projects.len(), 0);
    assert_eq!(result.total_cleanable, 0);
}

#[test]
fn test_scanner_finds_node_project() {
    let temp = TempDir::new().unwrap();
    create_node_project(temp.path());

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config).unwrap();

    assert_eq!(result.projects.len(), 1);
    assert_eq!(result.projects[0].kind, ProjectKind::NodeNpm);
    assert!(result.projects[0]
        .artifacts
        .iter()
        .any(|a| a.name() == "node_modules"));
}

#[test]
fn test_scanner_finds_rust_project() {
    let temp = TempDir::new().unwrap();
    create_rust_project(temp.path());

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config).unwrap();

    assert_eq!(result.projects.len(), 1);
    assert_eq!(result.projects[0].kind, ProjectKind::Rust);
    assert!(result.projects[0]
        .artifacts
        .iter()
        .any(|a| a.name() == "target"));
}

#[test]
fn test_scanner_finds_python_project() {
    let temp = TempDir::new().unwrap();
    create_python_project(temp.path());

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config).unwrap();

    assert_eq!(result.projects.len(), 1);
    assert_eq!(result.projects[0].kind, ProjectKind::PythonPip);

    let artifact_names: Vec<_> = result.projects[0]
        .artifacts
        .iter()
        .map(|a| a.name())
        .collect();

    assert!(artifact_names.contains(&".venv"));
    assert!(artifact_names.contains(&"__pycache__"));
}

#[test]
fn test_scanner_finds_multiple_projects() {
    let temp = TempDir::new().unwrap();

    // Create subdirectories with different project types
    let node_dir = temp.path().join("my-node-app");
    std::fs::create_dir(&node_dir).unwrap();
    create_node_project(&node_dir);

    let rust_dir = temp.path().join("my-rust-app");
    std::fs::create_dir(&rust_dir).unwrap();
    create_rust_project(&rust_dir);

    let python_dir = temp.path().join("my-python-app");
    std::fs::create_dir(&python_dir).unwrap();
    create_python_project(&python_dir);

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config).unwrap();

    assert_eq!(result.projects.len(), 3);

    let kinds: Vec<_> = result.projects.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&ProjectKind::NodeNpm));
    assert!(kinds.contains(&ProjectKind::Rust));
    assert!(kinds.contains(&ProjectKind::PythonPip));
}

#[test]
fn test_scanner_respects_max_depth() {
    let temp = TempDir::new().unwrap();

    // Create deeply nested project
    let deep_path = temp.path().join("a/b/c/d/e/f");
    std::fs::create_dir_all(&deep_path).unwrap();
    create_node_project(&deep_path);

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    // Depth 3 should not find the project
    let config = ScanConfig::new(temp.path()).with_max_depth(3);
    let result = scanner.scan(&config).unwrap();
    assert_eq!(result.projects.len(), 0);

    // Depth 10 should find it
    let config = ScanConfig::new(temp.path()).with_max_depth(10);
    let result = scanner.scan(&config).unwrap();
    assert_eq!(result.projects.len(), 1);
}

#[test]
fn test_scanner_min_size_filter() {
    let temp = TempDir::new().unwrap();
    create_node_project(temp.path());

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    // Set min size very high - should filter out our small test project
    let config = ScanConfig::new(temp.path()).with_min_size(1_000_000_000); // 1 GB
    let result = scanner.scan(&config).unwrap();

    // Project found but artifacts filtered
    assert_eq!(result.projects.len(), 0);
}

#[test]
fn test_scanner_cancellation() {
    let temp = TempDir::new().unwrap();
    create_node_project(temp.path());

    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    // Cancel immediately
    scanner.cancel();

    let config = ScanConfig::new(temp.path());
    let result = scanner.scan(&config);

    assert!(result.is_err());
}

// ============================================================================
// Trash/Delete Tests
// ============================================================================

#[test]
fn test_dry_run_preserves_files() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    std::fs::write(&file, "test content").unwrap();

    let size = null_e::trash::delete_path(&file, DeleteMethod::DryRun).unwrap();

    assert!(size > 0);
    assert!(file.exists()); // File should still exist
}

#[test]
fn test_permanent_delete_removes_files() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    std::fs::write(&file, "test content").unwrap();

    let size = null_e::trash::delete_path(&file, DeleteMethod::Permanent).unwrap();

    assert!(size > 0);
    assert!(!file.exists()); // File should be gone
}

#[test]
fn test_permanent_delete_removes_directories() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().join("subdir");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("file1.txt"), "content1").unwrap();
    std::fs::write(dir.join("file2.txt"), "content2").unwrap();

    let size = null_e::trash::delete_path(&dir, DeleteMethod::Permanent).unwrap();

    assert!(size > 0);
    assert!(!dir.exists());
}

// ============================================================================
// Git Integration Tests
// ============================================================================

fn init_git_repo(path: &std::path::Path) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("git init failed");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .ok();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .ok();
}

#[test]
fn test_git_status_non_repo() {
    let temp = TempDir::new().unwrap();
    let status = null_e::git::get_git_status(temp.path()).unwrap();
    assert!(status.is_none());
}

#[test]
fn test_git_status_clean_repo() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    // Create and commit a file
    std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let status = null_e::git::get_git_status(temp.path()).unwrap().unwrap();
    assert!(status.is_repo);
    assert!(!status.has_uncommitted);
    assert!(!status.has_untracked);
}

#[test]
fn test_git_status_uncommitted_changes() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    // Create and commit a file
    std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Modify the file
    std::fs::write(temp.path().join("test.txt"), "modified").unwrap();

    let status = null_e::git::get_git_status(temp.path()).unwrap().unwrap();
    assert!(status.has_uncommitted);
}

// ============================================================================
// Protection Tests
// ============================================================================

#[test]
fn test_protection_blocks_uncommitted() {
    let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));
    project.git_status = Some(null_e::core::GitStatus {
        is_repo: true,
        has_uncommitted: true,
        dirty_paths: vec![PathBuf::from("src/index.js")],
        ..Default::default()
    });

    let result =
        null_e::git::check_project_protection(&project, ProtectionLevel::Block);

    assert!(!result.allowed);
    assert!(result.blocked_reason.is_some());
}

#[test]
fn test_protection_warns_uncommitted() {
    let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));
    project.git_status = Some(null_e::core::GitStatus {
        is_repo: true,
        has_uncommitted: true,
        dirty_paths: vec![PathBuf::from("src/index.js")],
        ..Default::default()
    });

    let result =
        null_e::git::check_project_protection(&project, ProtectionLevel::Warn);

    assert!(result.allowed);
    assert!(result.has_warnings());
}

#[test]
fn test_protection_none_allows_everything() {
    let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));
    project.git_status = Some(null_e::core::GitStatus {
        is_repo: true,
        has_uncommitted: true,
        dirty_paths: vec![PathBuf::from("src/index.js")],
        ..Default::default()
    });

    let result =
        null_e::git::check_project_protection(&project, ProtectionLevel::None);

    assert!(result.allowed);
    assert!(!result.has_warnings());
}

// ============================================================================
// Config Tests
// ============================================================================

#[test]
fn test_config_default() {
    let config = null_e::config::Config::default();

    assert_eq!(config.clean.delete_method, DeleteMethod::Trash);
    assert_eq!(config.clean.protection_level, ProtectionLevel::Warn);
    assert!(config.scan.skip_hidden);
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = null_e::config::Config::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let loaded: null_e::config::Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.clean.delete_method, loaded.clean.delete_method);
    assert_eq!(
        config.clean.protection_level,
        loaded.clean.protection_level
    );
}

// ============================================================================
// Individual Plugin Tests
// ============================================================================

mod plugin_tests {
    use super::*;

    #[test]
    fn test_node_plugin_detection() {
        let temp = TempDir::new().unwrap();

        // No detection without package.json
        let plugin = NodePlugin;
        assert!(plugin.detect(temp.path()).is_none());

        // Detect npm by default
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodeNpm));

        // Detect yarn
        std::fs::write(temp.path().join("yarn.lock"), "").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodeYarn));

        // Detect pnpm (higher priority than yarn)
        std::fs::write(temp.path().join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodePnpm));
    }

    #[test]
    fn test_rust_plugin_detection() {
        let temp = TempDir::new().unwrap();

        let plugin = RustPlugin;
        assert!(plugin.detect(temp.path()).is_none());

        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::Rust));
    }

    #[test]
    fn test_python_plugin_detection() {
        let temp = TempDir::new().unwrap();

        let plugin = PythonPlugin;
        assert!(plugin.detect(temp.path()).is_none());

        std::fs::write(temp.path().join("requirements.txt"), "flask").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::PythonPip));

        // Poetry takes precedence
        std::fs::write(temp.path().join("poetry.lock"), "").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::PythonPoetry));
    }

    #[test]
    fn test_go_plugin_detection() {
        let temp = TempDir::new().unwrap();

        let plugin = GoPlugin;
        assert!(plugin.detect(temp.path()).is_none());

        std::fs::write(temp.path().join("go.mod"), "module test").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::Go));
    }

    #[test]
    fn test_maven_plugin_detection() {
        let temp = TempDir::new().unwrap();

        let plugin = MavenPlugin;
        assert!(plugin.detect(temp.path()).is_none());

        std::fs::write(temp.path().join("pom.xml"), "<project></project>").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::JavaMaven));
    }

    #[test]
    fn test_gradle_plugin_detection() {
        let temp = TempDir::new().unwrap();

        let plugin = GradlePlugin;
        assert!(plugin.detect(temp.path()).is_none());

        std::fs::write(temp.path().join("build.gradle"), "").unwrap();
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::JavaGradle));

        // Also detect Kotlin DSL
        let temp2 = TempDir::new().unwrap();
        std::fs::write(temp2.path().join("build.gradle.kts"), "").unwrap();
        assert_eq!(plugin.detect(temp2.path()), Some(ProjectKind::JavaGradle));
    }
}

// ============================================================================
// Statistics Tests
// ============================================================================

#[test]
fn test_artifact_stats() {
    use null_e::core::ArtifactStats;

    let mut stats = ArtifactStats::default();

    let artifact1 = Artifact::new(
        PathBuf::from("/test/node_modules"),
        ArtifactKind::Dependencies,
    );
    let mut artifact1 = artifact1;
    artifact1.size = 1000;
    artifact1.file_count = 100;

    let artifact2 = Artifact::new(PathBuf::from("/test/.cache"), ArtifactKind::Cache);
    let mut artifact2 = artifact2;
    artifact2.size = 500;
    artifact2.file_count = 50;

    stats.add(&artifact1);
    stats.add(&artifact2);

    assert_eq!(stats.total_size, 1500);
    assert_eq!(stats.total_files, 150);
    assert_eq!(stats.total_artifacts, 2);

    let largest = stats.largest_kind();
    assert!(largest.is_some());
    assert_eq!(largest.unwrap().0, ArtifactKind::Dependencies);
}
