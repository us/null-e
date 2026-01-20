//! Rust/Cargo plugin

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::Path;

/// Plugin for Rust/Cargo projects
pub struct RustPlugin;

impl Plugin for RustPlugin {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn name(&self) -> &'static str {
        "Rust (Cargo)"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::Rust]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("Cargo.toml"),
                kind: ProjectKind::Rust,
                priority: 60,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        if path.join("Cargo.toml").is_file() {
            Some(ProjectKind::Rust)
        } else {
            None
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // target directory - the BIG one for Rust
        let target = project_root.join("target");
        if target.exists() {
            artifacts.push(Artifact {
                path: target,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some("cargo build".into()),
                    lockfile: Some(project_root.join("Cargo.lock")),
                    restore_time_estimate: Some(60), // Rust builds can be slow
                    ..Default::default()
                },
            });
        }

        // debug artifacts in target/debug (if we want to be more granular)
        // For now, we just clean the whole target directory

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &["target"]
    }

    fn priority(&self) -> u8 {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let plugin = RustPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::Rust));
    }

    #[test]
    fn test_no_detect_without_cargo() {
        let temp = TempDir::new().unwrap();

        let plugin = RustPlugin;
        assert_eq!(plugin.detect(temp.path()), None);
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        std::fs::create_dir(temp.path().join("target")).unwrap();

        let plugin = RustPlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].name(), "target");
        assert_eq!(artifacts[0].kind, ArtifactKind::BuildOutput);
    }
}
