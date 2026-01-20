//! Node.js/npm/yarn/pnpm/bun plugin

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::{Path, PathBuf};

/// Plugin for Node.js ecosystem (npm, yarn, pnpm, bun)
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn id(&self) -> &'static str {
        "node"
    }

    fn name(&self) -> &'static str {
        "Node.js (npm/yarn/pnpm/bun)"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[
            ProjectKind::NodeNpm,
            ProjectKind::NodeYarn,
            ProjectKind::NodePnpm,
            ProjectKind::NodeBun,
        ]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("package.json"),
                kind: ProjectKind::NodeNpm,
                priority: 50,
            },
            ProjectMarker {
                indicator: MarkerKind::File("yarn.lock"),
                kind: ProjectKind::NodeYarn,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::File("pnpm-lock.yaml"),
                kind: ProjectKind::NodePnpm,
                priority: 60,
            },
            ProjectMarker {
                indicator: MarkerKind::File("bun.lockb"),
                kind: ProjectKind::NodeBun,
                priority: 60,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        // Must have package.json
        if !path.join("package.json").is_file() {
            return None;
        }

        // Determine specific variant by lockfile
        if path.join("bun.lockb").exists() {
            Some(ProjectKind::NodeBun)
        } else if path.join("pnpm-lock.yaml").exists() {
            Some(ProjectKind::NodePnpm)
        } else if path.join("yarn.lock").exists() {
            Some(ProjectKind::NodeYarn)
        } else {
            Some(ProjectKind::NodeNpm)
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // node_modules - the big one!
        let node_modules = project_root.join("node_modules");
        if node_modules.exists() {
            artifacts.push(Artifact {
                path: node_modules,
                kind: ArtifactKind::Dependencies,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some(self.restore_command(project_root)),
                    lockfile: self.find_lockfile(project_root),
                    ..Default::default()
                },
            });
        }

        // .next (Next.js)
        let next_dir = project_root.join(".next");
        if next_dir.exists() {
            artifacts.push(Artifact {
                path: next_dir,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build"),
            });
        }

        // .nuxt (Nuxt.js)
        let nuxt_dir = project_root.join(".nuxt");
        if nuxt_dir.exists() {
            artifacts.push(Artifact {
                path: nuxt_dir,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build"),
            });
        }

        // dist folder
        let dist = project_root.join("dist");
        if dist.exists() && dist.is_dir() {
            artifacts.push(Artifact {
                path: dist,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build"),
            });
        }

        // build folder (Create React App, etc.)
        let build = project_root.join("build");
        if build.exists() && build.is_dir() {
            // Check if it's a build output, not source
            if !project_root.join("build/index.html").exists()
                || project_root.join("src").exists()
            {
                artifacts.push(Artifact {
                    path: build,
                    kind: ArtifactKind::BuildOutput,
                    size: 0,
                    file_count: 0,
                    age: None,
                    metadata: ArtifactMetadata::restorable("npm run build"),
                });
            }
        }

        // .cache (various tools)
        let cache = project_root.join(".cache");
        if cache.exists() {
            artifacts.push(Artifact {
                path: cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .parcel-cache
        let parcel_cache = project_root.join(".parcel-cache");
        if parcel_cache.exists() {
            artifacts.push(Artifact {
                path: parcel_cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .turbo (Turborepo)
        let turbo = project_root.join(".turbo");
        if turbo.exists() {
            artifacts.push(Artifact {
                path: turbo,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // coverage (test coverage)
        let coverage = project_root.join("coverage");
        if coverage.exists() {
            artifacts.push(Artifact {
                path: coverage,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm test -- --coverage"),
            });
        }

        // .nyc_output (Istanbul coverage)
        let nyc = project_root.join(".nyc_output");
        if nyc.exists() {
            artifacts.push(Artifact {
                path: nyc,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // storybook-static
        let storybook = project_root.join("storybook-static");
        if storybook.exists() {
            artifacts.push(Artifact {
                path: storybook,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build-storybook"),
            });
        }

        // .svelte-kit
        let svelte_kit = project_root.join(".svelte-kit");
        if svelte_kit.exists() {
            artifacts.push(Artifact {
                path: svelte_kit,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build"),
            });
        }

        // out (Next.js static export)
        let out = project_root.join("out");
        if out.exists() && project_root.join("next.config.js").exists() {
            artifacts.push(Artifact {
                path: out,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("npm run build"),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &[
            "node_modules",
            ".next",
            ".nuxt",
            ".cache",
            ".parcel-cache",
            ".turbo",
            "coverage",
            ".nyc_output",
            "storybook-static",
            ".svelte-kit",
        ]
    }

    fn priority(&self) -> u8 {
        50
    }
}

impl NodePlugin {
    fn restore_command(&self, path: &Path) -> String {
        if path.join("bun.lockb").exists() {
            "bun install".into()
        } else if path.join("pnpm-lock.yaml").exists() {
            "pnpm install".into()
        } else if path.join("yarn.lock").exists() {
            "yarn install".into()
        } else {
            "npm install".into()
        }
    }

    fn find_lockfile(&self, path: &Path) -> Option<PathBuf> {
        let candidates = [
            "bun.lockb",
            "pnpm-lock.yaml",
            "yarn.lock",
            "package-lock.json",
        ];

        candidates.iter().map(|f| path.join(f)).find(|p| p.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_node_project(temp: &TempDir) {
        std::fs::write(temp.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
    }

    #[test]
    fn test_detect_npm() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);

        let plugin = NodePlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodeNpm));
    }

    #[test]
    fn test_detect_yarn() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);
        std::fs::write(temp.path().join("yarn.lock"), "").unwrap();

        let plugin = NodePlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodeYarn));
    }

    #[test]
    fn test_detect_pnpm() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);
        std::fs::write(temp.path().join("pnpm-lock.yaml"), "").unwrap();

        let plugin = NodePlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::NodePnpm));
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);
        std::fs::create_dir(temp.path().join("node_modules")).unwrap();
        std::fs::create_dir(temp.path().join(".next")).unwrap();

        let plugin = NodePlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert_eq!(artifacts.len(), 2);
        assert!(artifacts.iter().any(|a| a.name() == "node_modules"));
        assert!(artifacts.iter().any(|a| a.name() == ".next"));
    }

    #[test]
    fn test_no_artifacts_without_dirs() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);

        let plugin = NodePlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert!(artifacts.is_empty());
    }
}
