//! Docker cleanup module
//!
//! Handles cleanup of Docker resources:
//! - Dangling images
//! - Stopped containers
//! - Unused volumes
//! - Build cache

use super::{CleanableItem, SafetyLevel};
use crate::docker::parse_docker_size;
use crate::error::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Command;

/// Docker cleaner
pub struct DockerCleaner;

impl DockerCleaner {
    /// Create a new Docker cleaner
    pub fn new() -> Self {
        Self
    }

    /// Check if Docker is available
    pub fn is_available(&self) -> bool {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Detect all Docker cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        if !self.is_available() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Get disk usage summary
        if let Ok(df) = self.get_disk_usage() {
            items.extend(df);
        }

        // Dangling images
        items.extend(self.detect_dangling_images()?);

        // Stopped containers
        items.extend(self.detect_stopped_containers()?);

        // Unused volumes
        items.extend(self.detect_unused_volumes()?);

        // Build cache
        items.extend(self.detect_build_cache()?);

        Ok(items)
    }

    /// Get Docker disk usage summary
    fn get_disk_usage(&self) -> Result<Vec<CleanableItem>> {
        let output = Command::new("docker")
            .args([
                "system",
                "df",
                "--format",
                "{{.Type}}\t{{.Size}}\t{{.Reclaimable}}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let type_name = parts[0];
                let _total_size = parse_docker_size(parts[1]);
                let reclaimable = parse_docker_size(
                    parts[2]
                        .trim_end_matches([')', '%', '('])
                        .split('(')
                        .next()
                        .unwrap_or("0"),
                );

                if reclaimable > 0 {
                    let (icon, desc, safety) = match type_name {
                        "Images" => (
                            "🐳",
                            "Docker images not used by any container",
                            SafetyLevel::SafeWithCost,
                        ),
                        "Containers" => ("📦", "Stopped Docker containers", SafetyLevel::Safe),
                        "Local Volumes" => (
                            "💾",
                            "Docker volumes not used by any container",
                            SafetyLevel::Caution,
                        ),
                        "Build Cache" => ("🔨", "Docker build cache layers", SafetyLevel::Safe),
                        _ => ("🐳", "Docker resources", SafetyLevel::SafeWithCost),
                    };

                    items.push(CleanableItem {
                        name: format!("Docker {}", type_name),
                        category: "Docker".to_string(),
                        subcategory: type_name.to_string(),
                        icon,
                        path: PathBuf::from("/var/lib/docker"), // Placeholder
                        size: reclaimable,
                        file_count: None,
                        last_modified: None,
                        description: Cow::Borrowed(desc),
                        safe_to_delete: safety,
                        clean_command: Some(match type_name {
                            "Images" => "docker image prune -af".to_string(),
                            "Containers" => "docker container prune -f".to_string(),
                            "Local Volumes" => "docker volume prune -f".to_string(),
                            "Build Cache" => "docker builder prune -f".to_string(),
                            _ => "docker system prune -f".to_string(),
                        }),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect dangling images
    fn detect_dangling_images(&self) -> Result<Vec<CleanableItem>> {
        let output = Command::new("docker")
            .args([
                "images",
                "-f",
                "dangling=true",
                "--format",
                "{{.ID}}\t{{.Size}}\t{{.CreatedAt}}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let id = parts[0];
                let size = parse_docker_size(parts[1]);

                if size > 0 {
                    items.push(CleanableItem {
                        name: format!("Dangling Image: {}", &id[..12.min(id.len())]),
                        category: "Docker".to_string(),
                        subcategory: "Dangling Images".to_string(),
                        icon: "👻",
                        path: PathBuf::from(format!("/var/lib/docker/image/{}", id)),
                        size,
                        file_count: None,
                        last_modified: None,
                        description: Cow::Borrowed("Untagged image not used by any container."),
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: Some(format!("docker rmi -f {}", id)),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect stopped containers
    fn detect_stopped_containers(&self) -> Result<Vec<CleanableItem>> {
        let output = Command::new("docker")
            .args([
                "ps",
                "-a",
                "-f",
                "status=exited",
                "--format",
                "{{.ID}}\t{{.Names}}\t{{.Size}}\t{{.CreatedAt}}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let id = parts[0];
                let name = parts[1];
                let size_str = parts[2];

                // Parse container size (format: "0B (virtual 123MB)")
                let size = if let Some(virtual_start) = size_str.find("virtual ") {
                    let virtual_size = &size_str[virtual_start + 8..];
                    let end = virtual_size.find(')').unwrap_or(virtual_size.len());
                    parse_docker_size(&virtual_size[..end])
                } else {
                    parse_docker_size(size_str)
                };

                items.push(CleanableItem {
                    name: format!("Container: {}", name),
                    category: "Docker".to_string(),
                    subcategory: "Stopped Containers".to_string(),
                    icon: "📦",
                    path: PathBuf::from(format!("/var/lib/docker/containers/{}", id)),
                    size,
                    file_count: None,
                    last_modified: None,
                    description: Cow::Borrowed("Stopped container that can be removed."),
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some(format!("docker rm -f {}", id)),
                });
            }
        }

        Ok(items)
    }

    /// Detect unused volumes
    fn detect_unused_volumes(&self) -> Result<Vec<CleanableItem>> {
        // Get dangling volumes
        let output = Command::new("docker")
            .args([
                "volume",
                "ls",
                "-f",
                "dangling=true",
                "--format",
                "{{.Name}}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }

            // Get volume size
            let inspect = Command::new("docker")
                .args(["system", "df", "-v", "--format", "{{.Name}}\t{{.Size}}"])
                .output()
                .ok();

            let size = inspect
                .and_then(|o| {
                    let out = String::from_utf8_lossy(&o.stdout);
                    out.lines()
                        .find(|l| l.starts_with(name))
                        .and_then(|l| l.split('\t').nth(1))
                        .map(parse_docker_size)
                })
                .unwrap_or(0);

            items.push(CleanableItem {
                name: format!(
                    "Volume: {}",
                    if name.len() > 20 { &name[..20] } else { name }
                ),
                category: "Docker".to_string(),
                subcategory: "Volumes".to_string(),
                icon: "💾",
                path: PathBuf::from(format!("/var/lib/docker/volumes/{}", name)),
                size,
                file_count: None,
                last_modified: None,
                description: Cow::Borrowed("Docker volume not used by any container."),
                safe_to_delete: SafetyLevel::Caution,
                clean_command: Some(format!("docker volume rm {}", name)),
            });
        }

        Ok(items)
    }

    /// Detect build cache
    fn detect_build_cache(&self) -> Result<Vec<CleanableItem>> {
        let output = Command::new("docker")
            .args([
                "builder",
                "du",
                "--format",
                "{{.ID}}\t{{.Size}}\t{{.LastUsedAt}}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut total_size = 0u64;
        let mut count = 0usize;

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                total_size += parse_docker_size(parts[1]);
                count += 1;
            }
        }

        if total_size > 0 {
            Ok(vec![CleanableItem {
                name: format!("Build Cache ({} layers)", count),
                category: "Docker".to_string(),
                subcategory: "Build Cache".to_string(),
                icon: "🔨",
                path: PathBuf::from("/var/lib/docker/buildkit"),
                size: total_size,
                file_count: Some(count as u64),
                last_modified: None,
                description: Cow::Borrowed("Docker build cache layers. Speeds up rebuilds."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: Some("docker builder prune -a".to_string()),
            }])
        } else {
            Ok(vec![])
        }
    }

    /// Clean all Docker resources
    pub fn clean_all(&self, include_volumes: bool) -> Result<u64> {
        let args = if include_volumes {
            vec!["system", "prune", "-a", "--volumes", "-f"]
        } else {
            vec!["system", "prune", "-a", "-f"]
        };

        let output = Command::new("docker").args(&args).output()?;

        if !output.status.success() {
            return Ok(0);
        }

        // Parse "Total reclaimed space: X.XXGB" from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("reclaimed space") {
                if let Some(size_str) = line.split(':').nth(1) {
                    return Ok(parse_docker_size(size_str.trim()));
                }
            }
        }

        Ok(0)
    }
}

impl Default for DockerCleaner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_size() {
        // Docker uses SI units (not IEC)
        assert_eq!(parse_docker_size("1.5GB"), 1_500_000_000);
        assert_eq!(parse_docker_size("234MB"), 234_000_000);
        assert_eq!(parse_docker_size("567kB"), 567_000);
        assert_eq!(parse_docker_size("100B"), 100);
    }

    #[test]
    fn test_docker_cleaner() {
        let cleaner = DockerCleaner::new();
        if cleaner.is_available() {
            let items = cleaner.detect().unwrap();
            println!("Found {} Docker items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        } else {
            println!("Docker not available");
        }
    }
}
