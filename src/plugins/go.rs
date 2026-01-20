//! Go plugin

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::Path;

/// Plugin for Go projects
pub struct GoPlugin;

impl Plugin for GoPlugin {
    fn id(&self) -> &'static str {
        "go"
    }

    fn name(&self) -> &'static str {
        "Go"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::Go]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("go.mod"),
                kind: ProjectKind::Go,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::File("go.sum"),
                kind: ProjectKind::Go,
                priority: 55,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        if path.join("go.mod").is_file() {
            Some(ProjectKind::Go)
        } else {
            None
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // vendor directory (vendored dependencies)
        let vendor = project_root.join("vendor");
        if vendor.exists() {
            artifacts.push(Artifact {
                path: vendor,
                kind: ArtifactKind::Dependencies,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some("go mod vendor".into()),
                    lockfile: Some(project_root.join("go.sum")),
                    ..Default::default()
                },
            });
        }

        // bin directory (local binaries)
        let bin = project_root.join("bin");
        if bin.exists() {
            artifacts.push(Artifact {
                path: bin,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("go build"),
            });
        }

        // dist directory (release builds)
        let dist = project_root.join("dist");
        if dist.exists() {
            artifacts.push(Artifact {
                path: dist,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("go build"),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &["vendor"]
    }

    fn priority(&self) -> u8 {
        55
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_go() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("go.mod"),
            "module example.com/test\n\ngo 1.21\n",
        )
        .unwrap();

        let plugin = GoPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::Go));
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("go.mod"), "module test").unwrap();
        std::fs::create_dir(temp.path().join("vendor")).unwrap();

        let plugin = GoPlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].name(), "vendor");
    }
}
