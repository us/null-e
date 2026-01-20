//! Java plugin (Maven and Gradle)

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::Path;

/// Plugin for Maven projects
pub struct MavenPlugin;

impl Plugin for MavenPlugin {
    fn id(&self) -> &'static str {
        "maven"
    }

    fn name(&self) -> &'static str {
        "Java (Maven)"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::JavaMaven]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![ProjectMarker {
            indicator: MarkerKind::File("pom.xml"),
            kind: ProjectKind::JavaMaven,
            priority: 60,
        }]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        if path.join("pom.xml").is_file() {
            Some(ProjectKind::JavaMaven)
        } else {
            None
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // target directory
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
                    restore_command: Some("mvn compile".into()),
                    lockfile: Some(project_root.join("pom.xml")),
                    restore_time_estimate: Some(60),
                    ..Default::default()
                },
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &["target"]
    }

    fn priority(&self) -> u8 {
        60
    }
}

/// Plugin for Gradle projects
pub struct GradlePlugin;

impl Plugin for GradlePlugin {
    fn id(&self) -> &'static str {
        "gradle"
    }

    fn name(&self) -> &'static str {
        "Java (Gradle)"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::JavaGradle]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("build.gradle"),
                kind: ProjectKind::JavaGradle,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::File("build.gradle.kts"),
                kind: ProjectKind::JavaGradle,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::File("settings.gradle"),
                kind: ProjectKind::JavaGradle,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::File("settings.gradle.kts"),
                kind: ProjectKind::JavaGradle,
                priority: 55,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        if path.join("build.gradle").is_file()
            || path.join("build.gradle.kts").is_file()
            || path.join("settings.gradle").is_file()
            || path.join("settings.gradle.kts").is_file()
        {
            Some(ProjectKind::JavaGradle)
        } else {
            None
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // build directory
        let build = project_root.join("build");
        if build.exists() {
            artifacts.push(Artifact {
                path: build,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some("./gradlew build".into()),
                    ..Default::default()
                },
            });
        }

        // .gradle directory (local cache)
        let gradle_cache = project_root.join(".gradle");
        if gradle_cache.exists() {
            artifacts.push(Artifact {
                path: gradle_cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // out directory (IntelliJ IDEA)
        let out = project_root.join("out");
        if out.exists() {
            artifacts.push(Artifact {
                path: out,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("./gradlew build"),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &["build", ".gradle", "out"]
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
    fn test_detect_maven() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("pom.xml"),
            r#"<project></project>"#,
        )
        .unwrap();

        let plugin = MavenPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::JavaMaven));
    }

    #[test]
    fn test_detect_gradle() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("build.gradle"), "").unwrap();

        let plugin = GradlePlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::JavaGradle));
    }

    #[test]
    fn test_detect_gradle_kts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("build.gradle.kts"), "").unwrap();

        let plugin = GradlePlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::JavaGradle));
    }

    #[test]
    fn test_find_gradle_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("build.gradle"), "").unwrap();
        std::fs::create_dir(temp.path().join("build")).unwrap();
        std::fs::create_dir(temp.path().join(".gradle")).unwrap();

        let plugin = GradlePlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 2);
    }
}
