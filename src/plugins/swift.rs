//! Swift/Xcode plugin

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::Path;

/// Plugin for Swift/Xcode projects
pub struct SwiftPlugin;

impl Plugin for SwiftPlugin {
    fn id(&self) -> &'static str {
        "swift"
    }

    fn name(&self) -> &'static str {
        "Swift/Xcode"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::SwiftSpm, ProjectKind::SwiftXcode]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("Package.swift"),
                kind: ProjectKind::SwiftSpm,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::Extension("xcodeproj"),
                kind: ProjectKind::SwiftXcode,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::Extension("xcworkspace"),
                kind: ProjectKind::SwiftXcode,
                priority: 55,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        // Check for SPM first (higher priority)
        if path.join("Package.swift").is_file() {
            return Some(ProjectKind::SwiftSpm);
        }

        // Check for Xcode project
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace") {
                        return Some(ProjectKind::SwiftXcode);
                    }
                }
            }
        }

        None
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // .build directory (Swift Package Manager)
        let build = project_root.join(".build");
        if build.exists() {
            artifacts.push(Artifact {
                path: build,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("swift build"),
            });
        }

        // .swiftpm directory
        let swiftpm = project_root.join(".swiftpm");
        if swiftpm.exists() {
            artifacts.push(Artifact {
                path: swiftpm,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // Pods directory (CocoaPods)
        let pods = project_root.join("Pods");
        if pods.exists() {
            artifacts.push(Artifact {
                path: pods,
                kind: ArtifactKind::Dependencies,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some("pod install".into()),
                    lockfile: Some(project_root.join("Podfile.lock")),
                    ..Default::default()
                },
            });
        }

        // DerivedData (if in project - usually in ~/Library)
        let derived_data = project_root.join("DerivedData");
        if derived_data.exists() {
            artifacts.push(Artifact {
                path: derived_data,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("xcodebuild"),
            });
        }

        // build directory
        let build_dir = project_root.join("build");
        if build_dir.exists() {
            artifacts.push(Artifact {
                path: build_dir,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("xcodebuild"),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &[".build", ".swiftpm", "Pods", "DerivedData", "build"]
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
    fn test_detect_spm() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Package.swift"),
            "// swift-tools-version:5.5\nimport PackageDescription\n",
        )
        .unwrap();

        let plugin = SwiftPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::SwiftSpm));
    }

    #[test]
    fn test_detect_xcode() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join("MyApp.xcodeproj")).unwrap();

        let plugin = SwiftPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::SwiftXcode));
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Package.swift"), "").unwrap();
        std::fs::create_dir(temp.path().join(".build")).unwrap();
        std::fs::create_dir(temp.path().join("Pods")).unwrap();

        let plugin = SwiftPlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 2);
    }
}
