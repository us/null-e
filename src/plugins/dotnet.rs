//! .NET plugin

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::Path;

/// Plugin for .NET projects
pub struct DotNetPlugin;

impl Plugin for DotNetPlugin {
    fn id(&self) -> &'static str {
        "dotnet"
    }

    fn name(&self) -> &'static str {
        ".NET"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[ProjectKind::DotNet, ProjectKind::FSharp]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::AnyOf(vec!["*.csproj", "*.fsproj", "*.vbproj"]),
                kind: ProjectKind::DotNet,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::File("*.sln"),
                kind: ProjectKind::DotNet,
                priority: 50,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        // Check for project files
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".csproj")
                        || name.ends_with(".vbproj")
                        || name.ends_with(".sln")
                    {
                        return Some(ProjectKind::DotNet);
                    }
                    if name.ends_with(".fsproj") {
                        return Some(ProjectKind::FSharp);
                    }
                }
            }
        }
        None
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // bin directory
        let bin = project_root.join("bin");
        if bin.exists() {
            artifacts.push(Artifact {
                path: bin,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("dotnet build"),
            });
        }

        // obj directory
        let obj = project_root.join("obj");
        if obj.exists() {
            artifacts.push(Artifact {
                path: obj,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("dotnet build"),
            });
        }

        // packages directory (older NuGet style)
        let packages = project_root.join("packages");
        if packages.exists() {
            artifacts.push(Artifact {
                path: packages,
                kind: ArtifactKind::Dependencies,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("dotnet restore"),
            });
        }

        // TestResults
        let test_results = project_root.join("TestResults");
        if test_results.exists() {
            artifacts.push(Artifact {
                path: test_results,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("dotnet test"),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &["bin", "obj", "packages", "TestResults"]
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
    fn test_detect_dotnet() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("MyApp.csproj"), "<Project></Project>").unwrap();

        let plugin = DotNetPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::DotNet));
    }

    #[test]
    fn test_detect_fsharp() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("MyApp.fsproj"), "<Project></Project>").unwrap();

        let plugin = DotNetPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::FSharp));
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("App.csproj"), "").unwrap();
        std::fs::create_dir(temp.path().join("bin")).unwrap();
        std::fs::create_dir(temp.path().join("obj")).unwrap();

        let plugin = DotNetPlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 2);
    }
}
