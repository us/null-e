//! Git status detection
//!
//! Check for uncommitted changes, untracked files, etc.

use crate::core::GitStatus;
use crate::error::{DevSweepError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check git status for a project directory
///
/// This uses the git command-line tool for reliability and compatibility.
pub fn get_git_status(project_root: &Path) -> Result<Option<GitStatus>> {
    // Check if this is a git repository
    let git_dir = project_root.join(".git");
    if !git_dir.exists() {
        // Try to find parent git repo
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(project_root)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                // Found a parent repo, continue
            }
            _ => return Ok(None), // Not a git repo
        }
    }

    let mut status = GitStatus {
        is_repo: true,
        ..Default::default()
    };

    // Get current branch
    if let Ok(output) = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()
    {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                status.branch = Some(branch);
            }
        }
    }

    // Get remote URL
    if let Ok(output) = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_root)
        .output()
    {
        if output.status.success() {
            let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !remote.is_empty() {
                status.remote = Some(remote);
            }
        }
    }

    // Check for uncommitted changes (modified/staged files)
    if let Ok(output) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()
    {
        if output.status.success() {
            let status_output = String::from_utf8_lossy(&output.stdout);

            for line in status_output.lines() {
                if line.len() < 3 {
                    continue;
                }

                let status_code = &line[..2];
                let file_path = &line[3..];

                // First character: staged changes, Second: unstaged changes
                let first = status_code.chars().next().unwrap_or(' ');
                let second = status_code.chars().nth(1).unwrap_or(' ');

                // Check for untracked
                if first == '?' && second == '?' {
                    status.has_untracked = true;
                } else {
                    // Any other status means uncommitted changes
                    if first != ' ' || second != ' ' {
                        status.has_uncommitted = true;
                        status.dirty_paths.push(PathBuf::from(file_path));
                    }
                }
            }
        }
    }

    // Check for stashed changes
    if let Ok(output) = Command::new("git")
        .args(["stash", "list"])
        .current_dir(project_root)
        .output()
    {
        if output.status.success() {
            let stash_output = String::from_utf8_lossy(&output.stdout);
            status.has_stashed = !stash_output.trim().is_empty();
        }
    }

    Ok(Some(status))
}

/// Quick check if a path has uncommitted changes
pub fn has_uncommitted_changes(path: &Path) -> Result<bool> {
    match get_git_status(path)? {
        Some(status) => Ok(status.has_uncommitted),
        None => Ok(false),
    }
}

/// Check if a specific file/directory is tracked by git
pub fn is_git_tracked(repo_root: &Path, path: &Path) -> Result<bool> {
    let relative = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy();

    let output = Command::new("git")
        .args(["ls-files", &relative])
        .current_dir(repo_root)
        .output()
        .map_err(|e| DevSweepError::Git(e.to_string()))?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Find the git repository root for a path
pub fn find_repo_root(path: &Path) -> Result<Option<PathBuf>> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let root = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Ok(Some(PathBuf::from(root)))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_git_repo(path: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .expect("git init failed");

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .ok();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(path)
            .output()
            .ok();
    }

    #[test]
    fn test_non_git_repo() {
        let temp = TempDir::new().unwrap();
        let status = get_git_status(temp.path()).unwrap();
        assert!(status.is_none());
    }

    #[test]
    fn test_clean_repo() {
        let temp = TempDir::new().unwrap();
        init_git_repo(temp.path());

        // Create and commit a file
        std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        let status = get_git_status(temp.path()).unwrap().unwrap();
        assert!(status.is_repo);
        assert!(!status.has_uncommitted);
        assert!(!status.has_untracked);
    }

    #[test]
    fn test_uncommitted_changes() {
        let temp = TempDir::new().unwrap();
        init_git_repo(temp.path());

        // Create and commit a file
        std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Modify the file
        std::fs::write(temp.path().join("test.txt"), "modified").unwrap();

        let status = get_git_status(temp.path()).unwrap().unwrap();
        assert!(status.has_uncommitted);
    }

    #[test]
    fn test_untracked_files() {
        let temp = TempDir::new().unwrap();
        init_git_repo(temp.path());

        // Create untracked file
        std::fs::write(temp.path().join("new.txt"), "new file").unwrap();

        let status = get_git_status(temp.path()).unwrap().unwrap();
        assert!(status.has_untracked);
    }

    #[test]
    fn test_has_uncommitted_changes_helper() {
        let temp = TempDir::new().unwrap();

        // Non-git repo should return false
        assert!(!has_uncommitted_changes(temp.path()).unwrap());

        // Init and create uncommitted changes
        init_git_repo(temp.path());
        std::fs::write(temp.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();

        assert!(has_uncommitted_changes(temp.path()).unwrap());
    }
}
