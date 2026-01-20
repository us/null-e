//! Docker integration for cleaning containers, images, and volumes
//!
//! This module provides functionality to clean up Docker artifacts:
//! - Dangling images
//! - Unused volumes
//! - Build cache
//! - Stopped containers

use crate::core::{Artifact, ArtifactKind, ArtifactMetadata};
use crate::error::{DevSweepError, Result};
use std::path::PathBuf;
use std::process::Command;

/// Docker artifact types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerArtifactType {
    /// Dangling images (untagged)
    DanglingImages,
    /// All unused images
    UnusedImages,
    /// Unused volumes
    UnusedVolumes,
    /// Build cache
    BuildCache,
    /// Stopped containers
    StoppedContainers,
}

impl DockerArtifactType {
    /// Get the docker command to list this artifact type
    pub fn list_command(&self) -> Vec<&'static str> {
        match self {
            Self::DanglingImages => vec!["images", "-f", "dangling=true", "-q"],
            Self::UnusedImages => vec!["images", "-q"],
            Self::UnusedVolumes => vec!["volume", "ls", "-f", "dangling=true", "-q"],
            Self::BuildCache => vec!["builder", "du"],
            Self::StoppedContainers => vec!["ps", "-a", "-f", "status=exited", "-q"],
        }
    }

    /// Get the docker command to clean this artifact type
    pub fn clean_command(&self) -> Vec<&'static str> {
        match self {
            Self::DanglingImages => vec!["image", "prune", "-f"],
            Self::UnusedImages => vec!["image", "prune", "-a", "-f"],
            Self::UnusedVolumes => vec!["volume", "prune", "-f"],
            Self::BuildCache => vec!["builder", "prune", "-f"],
            Self::StoppedContainers => vec!["container", "prune", "-f"],
        }
    }
}

/// Check if Docker is available
pub fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get Docker disk usage summary
pub fn get_docker_disk_usage() -> Result<DockerDiskUsage> {
    if !is_docker_available() {
        return Err(DevSweepError::DockerNotAvailable);
    }

    let output = Command::new("docker")
        .args(["system", "df", "--format", "{{json .}}"])
        .output()
        .map_err(|e| DevSweepError::Docker(e.to_string()))?;

    if !output.status.success() {
        return Err(DevSweepError::Docker(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut usage = DockerDiskUsage::default();

    for line in stdout.lines() {
        if let Ok(entry) = serde_json::from_str::<DockerDfEntry>(line) {
            match entry.r#type.as_str() {
                "Images" => {
                    usage.images_size = parse_docker_size(&entry.size);
                    usage.images_reclaimable = parse_docker_size(&entry.reclaimable);
                }
                "Containers" => {
                    usage.containers_size = parse_docker_size(&entry.size);
                    usage.containers_reclaimable = parse_docker_size(&entry.reclaimable);
                }
                "Local Volumes" => {
                    usage.volumes_size = parse_docker_size(&entry.size);
                    usage.volumes_reclaimable = parse_docker_size(&entry.reclaimable);
                }
                "Build Cache" => {
                    usage.build_cache_size = parse_docker_size(&entry.size);
                    usage.build_cache_reclaimable = parse_docker_size(&entry.reclaimable);
                }
                _ => {}
            }
        }
    }

    Ok(usage)
}

/// Docker disk usage summary
#[derive(Debug, Clone, Default)]
pub struct DockerDiskUsage {
    pub images_size: u64,
    pub images_reclaimable: u64,
    pub containers_size: u64,
    pub containers_reclaimable: u64,
    pub volumes_size: u64,
    pub volumes_reclaimable: u64,
    pub build_cache_size: u64,
    pub build_cache_reclaimable: u64,
}

impl DockerDiskUsage {
    /// Get total size
    pub fn total_size(&self) -> u64 {
        self.images_size
            + self.containers_size
            + self.volumes_size
            + self.build_cache_size
    }

    /// Get total reclaimable
    pub fn total_reclaimable(&self) -> u64 {
        self.images_reclaimable
            + self.containers_reclaimable
            + self.volumes_reclaimable
            + self.build_cache_reclaimable
    }

    /// Convert to artifacts
    pub fn to_artifacts(&self) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        if self.images_reclaimable > 0 {
            artifacts.push(Artifact {
                path: PathBuf::from("docker://images"),
                kind: ArtifactKind::Docker,
                size: self.images_reclaimable,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: false,
                    restore_command: Some("docker pull".into()),
                    ..Default::default()
                },
            });
        }

        if self.volumes_reclaimable > 0 {
            artifacts.push(Artifact {
                path: PathBuf::from("docker://volumes"),
                kind: ArtifactKind::Docker,
                size: self.volumes_reclaimable,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: false,
                    ..Default::default()
                },
            });
        }

        if self.build_cache_reclaimable > 0 {
            artifacts.push(Artifact {
                path: PathBuf::from("docker://build-cache"),
                kind: ArtifactKind::Docker,
                size: self.build_cache_reclaimable,
                file_count: 0,
                age: None,
                metadata: ArtifactMetadata {
                    restorable: true,
                    restore_command: Some("docker build".into()),
                    ..Default::default()
                },
            });
        }

        artifacts
    }
}

#[derive(serde::Deserialize)]
struct DockerDfEntry {
    #[serde(rename = "Type")]
    r#type: String,
    #[serde(rename = "Size")]
    size: String,
    #[serde(rename = "Reclaimable")]
    reclaimable: String,
}

/// Parse Docker size string (e.g., "1.5GB", "100MB")
fn parse_docker_size(s: &str) -> u64 {
    let s = s.trim();

    // Remove percentage in parentheses
    let s = s.split('(').next().unwrap_or(s).trim();

    let (num_part, unit) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1_000_000_000u64)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1_000_000u64)
    } else if s.ends_with("KB") || s.ends_with("kB") {
        (&s[..s.len() - 2], 1_000u64)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1u64)
    } else {
        (s, 1u64)
    };

    num_part
        .trim()
        .parse::<f64>()
        .map(|n| (n * unit as f64) as u64)
        .unwrap_or(0)
}

/// Clean Docker artifacts
pub fn clean_docker(artifact_type: DockerArtifactType) -> Result<u64> {
    if !is_docker_available() {
        return Err(DevSweepError::DockerNotAvailable);
    }

    let args = artifact_type.clean_command();
    let output = Command::new("docker")
        .args(&args)
        .output()
        .map_err(|e| DevSweepError::Docker(e.to_string()))?;

    if !output.status.success() {
        return Err(DevSweepError::Docker(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Try to parse reclaimed space from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reclaimed = parse_reclaimed_space(&stdout);

    Ok(reclaimed)
}

/// Parse reclaimed space from docker prune output
fn parse_reclaimed_space(output: &str) -> u64 {
    // Docker outputs like "Total reclaimed space: 1.5GB"
    for line in output.lines() {
        if line.contains("reclaimed space:") || line.contains("Reclaimed space:") {
            if let Some(size_part) = line.split(':').nth(1) {
                return parse_docker_size(size_part);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_size() {
        assert_eq!(parse_docker_size("1.5GB"), 1_500_000_000);
        assert_eq!(parse_docker_size("100MB"), 100_000_000);
        assert_eq!(parse_docker_size("500KB"), 500_000);
        assert_eq!(parse_docker_size("1.5GB (50%)"), 1_500_000_000);
    }

    #[test]
    fn test_docker_artifact_commands() {
        let dangling = DockerArtifactType::DanglingImages;
        assert!(dangling.clean_command().contains(&"prune"));
    }
}
