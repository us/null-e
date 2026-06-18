//! Node.js/npm/yarn/pnpm/bun plugin

use crate::core::{
    Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker,
};
use crate::error::Result;
use crate::plugins::Plugin;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
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

    fn detect_with_listing(&self, path: &Path, listing: &HashSet<OsString>) -> Option<ProjectKind> {
        // If the listing is empty we may have failed to read the directory (or it
        // is genuinely empty). Either way, delegate to `detect` so the original
        // syscall-based behaviour is preserved exactly (an empty dir has no
        // package.json, so `detect` returns `None` regardless).
        if listing.is_empty() {
            return self.detect(path);
        }

        // Must have package.json. Membership in the listing is necessary but not
        // sufficient — the original requires it to be a *file*, so confirm with a
        // single `is_file()` only when the name is actually present.
        if !listing.contains(OsStr::new("package.json")) || !path.join("package.json").is_file() {
            return None;
        }

        // Determine specific variant by lockfile. Membership presence is
        // equivalent to the original `.exists()` checks, with zero extra syscalls.
        if listing.contains(OsStr::new("bun.lockb")) {
            Some(ProjectKind::NodeBun)
        } else if listing.contains(OsStr::new("pnpm-lock.yaml")) {
            Some(ProjectKind::NodePnpm)
        } else if listing.contains(OsStr::new("yarn.lock")) {
            Some(ProjectKind::NodeYarn)
        } else {
            Some(ProjectKind::NodeNpm)
        }
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        // Read the project root ONCE into a set of immediate child names so each
        // candidate artifact is a cheap membership check instead of a `path.exists()`
        // syscall. If the directory can't be read, fall back to the original
        // per-path implementation so nothing breaks.
        let listing: HashSet<OsString> = match std::fs::read_dir(project_root) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name())
                .collect(),
            Err(_) => return self.find_artifacts_slow(project_root),
        };

        // `present(name)` mirrors the original `project_root.join(name).exists()`:
        // a directory entry of that name exists. Secondary type/sibling checks
        // (`.is_dir()`, nested files) are preserved verbatim below.
        let present = |name: &str| listing.contains(OsStr::new(name));

        let mut artifacts = Vec::new();

        // node_modules - the big one!
        if present("node_modules") {
            let node_modules = project_root.join("node_modules");
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
        if present(".next") {
            let next_dir = project_root.join(".next");
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
        if present(".nuxt") {
            let nuxt_dir = project_root.join(".nuxt");
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
        if present("dist") && dist.is_dir() {
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
        if present("build") && build.is_dir() {
            // Check if it's a build output, not source
            if !project_root.join("build/index.html").exists() || project_root.join("src").exists()
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
        if present(".cache") {
            let cache = project_root.join(".cache");
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
        if present(".parcel-cache") {
            let parcel_cache = project_root.join(".parcel-cache");
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
        if present(".turbo") {
            let turbo = project_root.join(".turbo");
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
        if present("coverage") {
            let coverage = project_root.join("coverage");
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
        if present(".nyc_output") {
            let nyc = project_root.join(".nyc_output");
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
        if present("storybook-static") {
            let storybook = project_root.join("storybook-static");
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
        if present(".svelte-kit") {
            let svelte_kit = project_root.join(".svelte-kit");
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
        if present("out") && project_root.join("next.config.js").exists() {
            let out = project_root.join("out");
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
    /// Original per-path implementation of [`Plugin::find_artifacts`], used as a
    /// fallback when the project root cannot be listed via `read_dir`. Keeps the
    /// EXACT artifact-detection semantics so the fast path and this path agree.
    fn find_artifacts_slow(&self, project_root: &Path) -> Result<Vec<Artifact>> {
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
            if !project_root.join("build/index.html").exists() || project_root.join("src").exists()
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

    /// The single-readdir fast path must agree with the per-path fallback on the
    /// exact set of artifacts (names), so behaviour is preserved.
    #[test]
    fn test_fast_and_slow_find_artifacts_agree() {
        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);
        // A mix of dir-only, dir+sibling, and dir+nested-file gated artifacts.
        std::fs::create_dir(temp.path().join("node_modules")).unwrap();
        std::fs::create_dir(temp.path().join(".next")).unwrap();
        std::fs::create_dir(temp.path().join("dist")).unwrap();
        std::fs::create_dir(temp.path().join("coverage")).unwrap();
        std::fs::create_dir(temp.path().join("out")).unwrap();
        std::fs::write(temp.path().join("next.config.js"), "").unwrap();
        // `build` with index.html but no src/ must be excluded by both paths.
        std::fs::create_dir(temp.path().join("build")).unwrap();
        std::fs::write(temp.path().join("build/index.html"), "").unwrap();

        let plugin = NodePlugin;
        let mut fast: Vec<String> = plugin
            .find_artifacts(temp.path())
            .unwrap()
            .iter()
            .map(|a| a.name().to_string())
            .collect();
        let mut slow: Vec<String> = plugin
            .find_artifacts_slow(temp.path())
            .unwrap()
            .iter()
            .map(|a| a.name().to_string())
            .collect();
        fast.sort();
        slow.sort();
        assert_eq!(fast, slow);
        // Sanity: the gated `build` artifact must be absent in both.
        assert!(!fast.contains(&"build".to_string()));
        assert!(fast.contains(&"out".to_string()));
    }

    /// `detect_with_listing` must return the same result as `detect`.
    #[test]
    fn test_detect_with_listing_matches_detect() {
        use std::collections::HashSet;
        use std::ffi::OsString;

        let temp = TempDir::new().unwrap();
        setup_node_project(&temp);
        std::fs::write(temp.path().join("pnpm-lock.yaml"), "").unwrap();

        let listing: HashSet<OsString> = std::fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect();

        let plugin = NodePlugin;
        assert_eq!(
            plugin.detect_with_listing(temp.path(), &listing),
            plugin.detect(temp.path())
        );
        assert_eq!(
            plugin.detect_with_listing(temp.path(), &listing),
            Some(ProjectKind::NodePnpm)
        );

        // Empty listing delegates to `detect` (read_dir-failure fallback path).
        let empty: HashSet<OsString> = HashSet::new();
        assert_eq!(
            plugin.detect_with_listing(temp.path(), &empty),
            plugin.detect(temp.path())
        );
    }
}
