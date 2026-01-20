//! Git protection - safety guards for cleaning operations

use crate::core::{Artifact, Project};
use crate::error::Result;

/// Protection level for cleaning operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProtectionLevel {
    /// No protection - delete anything (dangerous!)
    None,
    /// Warn but allow cleaning projects with uncommitted changes
    #[default]
    Warn,
    /// Block cleaning projects with uncommitted changes
    Block,
    /// Paranoid: require explicit confirmation for everything
    Paranoid,
}

impl ProtectionLevel {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "warn" => Some(Self::Warn),
            "block" => Some(Self::Block),
            "paranoid" => Some(Self::Paranoid),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Warn => "warn",
            Self::Block => "block",
            Self::Paranoid => "paranoid",
        }
    }
}

/// Result of a protection check
#[derive(Debug, Clone)]
pub struct ProtectionResult {
    /// Whether cleaning is allowed
    pub allowed: bool,
    /// Warnings to show the user
    pub warnings: Vec<String>,
    /// Reason if blocked
    pub blocked_reason: Option<String>,
    /// Suggested action
    pub suggestion: Option<String>,
}

impl ProtectionResult {
    /// Create an allowed result
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            warnings: Vec::new(),
            blocked_reason: None,
            suggestion: None,
        }
    }

    /// Create a blocked result
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            warnings: Vec::new(),
            blocked_reason: Some(reason.into()),
            suggestion: None,
        }
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Check if it's safe to clean a project
pub fn check_project_protection(
    project: &Project,
    level: ProtectionLevel,
) -> ProtectionResult {
    if level == ProtectionLevel::None {
        return ProtectionResult::allowed();
    }

    let mut result = ProtectionResult::allowed();

    // Check git status
    match &project.git_status {
        Some(status) if status.has_uncommitted => {
            let msg = format!(
                "Project '{}' has uncommitted changes ({} files)",
                project.name,
                status.dirty_paths.len()
            );

            match level {
                ProtectionLevel::Warn => {
                    result = result.with_warning(msg);
                    result = result.with_suggestion("Commit or stash changes first");
                }
                ProtectionLevel::Block | ProtectionLevel::Paranoid => {
                    return ProtectionResult::blocked(msg)
                        .with_suggestion("Use --force to override or commit changes first");
                }
                _ => {}
            }
        }
        Some(status) if status.has_untracked => {
            let msg = format!("Project '{}' has untracked files", project.name);
            result = result.with_warning(msg);
        }
        None => {
            let msg = format!(
                "Project '{}' is not a git repository - cannot verify safety",
                project.name
            );

            match level {
                ProtectionLevel::Paranoid => {
                    return ProtectionResult::blocked(msg)
                        .with_suggestion("Initialize a git repo or use --force");
                }
                _ => {
                    result = result.with_warning(msg);
                }
            }
        }
        _ => {}
    }

    // Check if recently modified
    if let Some(modified) = project.last_modified {
        if let Ok(age) = modified.elapsed() {
            let days = age.as_secs() / 86400;
            if days < 7 {
                let msg = format!(
                    "Project '{}' was modified recently ({} days ago)",
                    project.name, days
                );

                match level {
                    ProtectionLevel::Paranoid => {
                        return ProtectionResult::blocked(msg);
                    }
                    _ => {
                        result = result.with_warning(msg);
                    }
                }
            }
        }
    }

    result
}

/// Check if a specific artifact is safe to clean
pub fn check_artifact_protection(
    artifact: &Artifact,
    project: &Project,
    level: ProtectionLevel,
) -> ProtectionResult {
    if level == ProtectionLevel::None {
        return ProtectionResult::allowed();
    }

    let mut result = ProtectionResult::allowed();

    // Check if artifact path contains uncommitted changes
    if let Some(status) = &project.git_status {
        for dirty_path in &status.dirty_paths {
            // Check if dirty path is inside artifact path
            if dirty_path.starts_with(&artifact.path) || artifact.path.starts_with(dirty_path) {
                let msg = format!(
                    "Artifact '{}' contains uncommitted changes",
                    artifact.path.display()
                );

                match level {
                    ProtectionLevel::Warn => {
                        result = result.with_warning(msg);
                    }
                    ProtectionLevel::Block | ProtectionLevel::Paranoid => {
                        return ProtectionResult::blocked(msg);
                    }
                    _ => {}
                }
            }
        }
    }

    // Check artifact safety level
    match artifact.kind.default_safety() {
        crate::core::ArtifactSafety::NeverAuto => {
            return ProtectionResult::blocked(format!(
                "Artifact '{}' should never be auto-deleted",
                artifact.name()
            ));
        }
        crate::core::ArtifactSafety::RequiresConfirmation
            if level == ProtectionLevel::Paranoid =>
        {
            return ProtectionResult::blocked(format!(
                "Artifact '{}' requires explicit confirmation",
                artifact.name()
            ));
        }
        crate::core::ArtifactSafety::SafeWithLockfile if artifact.metadata.lockfile.is_none() => {
            result = result.with_warning(format!(
                "No lockfile found for '{}' - reinstallation may change versions",
                artifact.name()
            ));
        }
        _ => {}
    }

    result
}

/// Add git status to all projects in a list
pub fn enrich_with_git_status(projects: &mut [Project]) -> Result<()> {
    use rayon::prelude::*;

    projects.par_iter_mut().for_each(|project| {
        if let Ok(status) = super::get_git_status(&project.root) {
            project.git_status = status;
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GitStatus, ProjectKind};
    use std::path::PathBuf;

    fn create_test_project(has_uncommitted: bool) -> Project {
        let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));

        if has_uncommitted {
            project.git_status = Some(GitStatus {
                is_repo: true,
                has_uncommitted: true,
                dirty_paths: vec![PathBuf::from("src/index.js")],
                ..Default::default()
            });
        } else {
            project.git_status = Some(GitStatus {
                is_repo: true,
                has_uncommitted: false,
                ..Default::default()
            });
        }

        project
    }

    #[test]
    fn test_protection_none_allows_everything() {
        let project = create_test_project(true);
        let result = check_project_protection(&project, ProtectionLevel::None);
        assert!(result.allowed);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_protection_warn_uncommitted() {
        let project = create_test_project(true);
        let result = check_project_protection(&project, ProtectionLevel::Warn);
        assert!(result.allowed);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_protection_block_uncommitted() {
        let project = create_test_project(true);
        let result = check_project_protection(&project, ProtectionLevel::Block);
        assert!(!result.allowed);
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn test_protection_clean_repo_allowed() {
        let project = create_test_project(false);
        let result = check_project_protection(&project, ProtectionLevel::Block);
        assert!(result.allowed);
    }

    #[test]
    fn test_protection_no_git_repo() {
        let mut project = Project::new(ProjectKind::NodeNpm, PathBuf::from("/test"));
        project.git_status = None;

        let result = check_project_protection(&project, ProtectionLevel::Warn);
        assert!(result.allowed);
        assert!(result.has_warnings());

        let result = check_project_protection(&project, ProtectionLevel::Paranoid);
        assert!(!result.allowed);
    }

    #[test]
    fn test_protection_level_from_str() {
        assert_eq!(ProtectionLevel::from_str("none"), Some(ProtectionLevel::None));
        assert_eq!(ProtectionLevel::from_str("WARN"), Some(ProtectionLevel::Warn));
        assert_eq!(ProtectionLevel::from_str("Block"), Some(ProtectionLevel::Block));
        assert_eq!(ProtectionLevel::from_str("invalid"), None);
    }
}
