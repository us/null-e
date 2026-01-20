//! Python plugin (pip, poetry, pipenv, conda, uv)

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata, MarkerKind, ProjectKind, ProjectMarker};
use crate::error::Result;
use crate::plugins::Plugin;
use std::path::{Path, PathBuf};

/// Plugin for Python ecosystem
pub struct PythonPlugin;

impl Plugin for PythonPlugin {
    fn id(&self) -> &'static str {
        "python"
    }

    fn name(&self) -> &'static str {
        "Python (pip/poetry/pipenv/uv)"
    }

    fn supported_kinds(&self) -> &[ProjectKind] {
        &[
            ProjectKind::PythonPip,
            ProjectKind::PythonPoetry,
            ProjectKind::PythonPipenv,
            ProjectKind::PythonConda,
            ProjectKind::PythonUv,
        ]
    }

    fn markers(&self) -> Vec<ProjectMarker> {
        vec![
            ProjectMarker {
                indicator: MarkerKind::File("pyproject.toml"),
                kind: ProjectKind::PythonPoetry,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::File("Pipfile"),
                kind: ProjectKind::PythonPipenv,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::File("requirements.txt"),
                kind: ProjectKind::PythonPip,
                priority: 50,
            },
            ProjectMarker {
                indicator: MarkerKind::File("setup.py"),
                kind: ProjectKind::PythonPip,
                priority: 45,
            },
            ProjectMarker {
                indicator: MarkerKind::File("environment.yml"),
                kind: ProjectKind::PythonConda,
                priority: 55,
            },
            ProjectMarker {
                indicator: MarkerKind::File("uv.lock"),
                kind: ProjectKind::PythonUv,
                priority: 60,
            },
        ]
    }

    fn detect(&self, path: &Path) -> Option<ProjectKind> {
        // Check for uv (newest)
        if path.join("uv.lock").exists() {
            return Some(ProjectKind::PythonUv);
        }

        // Check for Poetry
        if path.join("poetry.lock").exists() {
            return Some(ProjectKind::PythonPoetry);
        }

        // Check for pyproject.toml (could be poetry or modern pip)
        if path.join("pyproject.toml").exists() {
            // Try to determine if it's poetry
            if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
                if content.contains("[tool.poetry]") {
                    return Some(ProjectKind::PythonPoetry);
                }
            }
            return Some(ProjectKind::PythonPip);
        }

        // Check for Pipenv
        if path.join("Pipfile").exists() {
            return Some(ProjectKind::PythonPipenv);
        }

        // Check for Conda
        if path.join("environment.yml").exists() || path.join("environment.yaml").exists() {
            return Some(ProjectKind::PythonConda);
        }

        // Check for basic requirements.txt or setup.py
        if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
            return Some(ProjectKind::PythonPip);
        }

        None
    }

    fn find_artifacts(&self, project_root: &Path) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        // Virtual environments
        for venv_name in &[".venv", "venv", "env", ".env"] {
            let venv_path = project_root.join(venv_name);
            if venv_path.exists() && is_venv(&venv_path) {
                artifacts.push(Artifact {
                    path: venv_path,
                    kind: ArtifactKind::VirtualEnv,
                    size: 0,
                    file_count: 0,
                    age: None,
                    metadata: ArtifactMetadata {
                        restorable: true,
                        restore_command: Some(self.restore_command(project_root)),
                        lockfile: self.find_lockfile(project_root),
                        restore_time_estimate: Some(30),
                        ..Default::default()
                    },
                });
                break; // Usually only one venv
            }
        }

        // __pycache__ directories (can be multiple)
        self.find_pycache_dirs(project_root, &mut artifacts)?;

        // .pytest_cache
        let pytest_cache = project_root.join(".pytest_cache");
        if pytest_cache.exists() {
            artifacts.push(Artifact {
                path: pytest_cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .mypy_cache
        let mypy_cache = project_root.join(".mypy_cache");
        if mypy_cache.exists() {
            artifacts.push(Artifact {
                path: mypy_cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .ruff_cache
        let ruff_cache = project_root.join(".ruff_cache");
        if ruff_cache.exists() {
            artifacts.push(Artifact {
                path: ruff_cache,
                kind: ArtifactKind::Cache,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .tox
        let tox = project_root.join(".tox");
        if tox.exists() {
            artifacts.push(Artifact {
                path: tox,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("tox"),
            });
        }

        // .nox
        let nox = project_root.join(".nox");
        if nox.exists() {
            artifacts.push(Artifact {
                path: nox,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("nox"),
            });
        }

        // *.egg-info
        self.find_egg_info(project_root, &mut artifacts)?;

        // dist and build directories
        let dist = project_root.join("dist");
        if dist.exists() {
            artifacts.push(Artifact {
                path: dist,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("python -m build"),
            });
        }

        let build = project_root.join("build");
        if build.exists() {
            artifacts.push(Artifact {
                path: build,
                kind: ArtifactKind::BuildOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::restorable("python setup.py build"),
            });
        }

        // htmlcov (coverage reports)
        let htmlcov = project_root.join("htmlcov");
        if htmlcov.exists() {
            artifacts.push(Artifact {
                path: htmlcov,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        // .coverage file
        let coverage = project_root.join(".coverage");
        if coverage.exists() {
            artifacts.push(Artifact {
                path: coverage,
                kind: ArtifactKind::TestOutput,
                size: 0,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata::default(),
            });
        }

        Ok(artifacts)
    }

    fn cleanable_dirs(&self) -> &[&'static str] {
        &[
            ".venv",
            "venv",
            "__pycache__",
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            ".nox",
            "htmlcov",
        ]
    }

    fn priority(&self) -> u8 {
        50
    }
}

impl PythonPlugin {
    fn restore_command(&self, path: &Path) -> String {
        if path.join("uv.lock").exists() {
            "uv sync".into()
        } else if path.join("poetry.lock").exists() {
            "poetry install".into()
        } else if path.join("Pipfile.lock").exists() {
            "pipenv install".into()
        } else if path.join("requirements.txt").exists() {
            "pip install -r requirements.txt".into()
        } else {
            "pip install -e .".into()
        }
    }

    fn find_lockfile(&self, path: &Path) -> Option<PathBuf> {
        let candidates = [
            "uv.lock",
            "poetry.lock",
            "Pipfile.lock",
            "requirements.txt",
        ];

        candidates.iter().map(|f| path.join(f)).find(|p| p.exists())
    }

    fn find_pycache_dirs(&self, root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
        // Only search one level deep for __pycache__ in the project root
        // to avoid scanning into venv
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str());
                    if name == Some("__pycache__") {
                        artifacts.push(Artifact {
                            path,
                            kind: ArtifactKind::Bytecode,
                            size: 0,
                            file_count: 0,
                            age: None,
                            metadata: ArtifactMetadata::default(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn find_egg_info(&self, root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".egg-info") {
                            artifacts.push(Artifact {
                                path,
                                kind: ArtifactKind::BuildOutput,
                                size: 0,
                                file_count: 0,
                                age: None,
                                metadata: ArtifactMetadata::default(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Check if a directory is a Python virtual environment
fn is_venv(path: &Path) -> bool {
    // Check for pyvenv.cfg (standard venv marker)
    if path.join("pyvenv.cfg").exists() {
        return true;
    }

    // Check for bin/python or Scripts/python.exe
    let has_python = path.join("bin/python").exists()
        || path.join("bin/python3").exists()
        || path.join("Scripts/python.exe").exists();

    has_python
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_poetry() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("poetry.lock"), "").unwrap();

        let plugin = PythonPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::PythonPoetry));
    }

    #[test]
    fn test_detect_pip() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("requirements.txt"), "flask\n").unwrap();

        let plugin = PythonPlugin;
        assert_eq!(plugin.detect(temp.path()), Some(ProjectKind::PythonPip));
    }

    #[test]
    fn test_is_venv() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("pyvenv.cfg"), "").unwrap();

        assert!(is_venv(temp.path()));
    }

    #[test]
    fn test_find_artifacts() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("requirements.txt"), "").unwrap();

        // Create a fake venv
        let venv = temp.path().join(".venv");
        std::fs::create_dir(&venv).unwrap();
        std::fs::write(venv.join("pyvenv.cfg"), "").unwrap();

        // Create __pycache__
        std::fs::create_dir(temp.path().join("__pycache__")).unwrap();

        // Create .pytest_cache
        std::fs::create_dir(temp.path().join(".pytest_cache")).unwrap();

        let plugin = PythonPlugin;
        let artifacts = plugin.find_artifacts(temp.path()).unwrap();

        assert!(artifacts.len() >= 3);
        assert!(artifacts.iter().any(|a| a.name() == ".venv"));
        assert!(artifacts.iter().any(|a| a.name() == "__pycache__"));
        assert!(artifacts.iter().any(|a| a.name() == ".pytest_cache"));
    }
}
