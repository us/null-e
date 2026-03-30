use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use null_e_core::caches::GlobalCache;
use null_e_core::cleaners::{CleanableItem, SafetyLevel};
use null_e_core::core::{Artifact, Project, ScanConfig, ScanResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfigDto {
    pub roots: Vec<String>,
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
}

impl From<ScanConfigDto> for ScanConfig {
    fn from(dto: ScanConfigDto) -> Self {
        ScanConfig {
            roots: dto.roots.into_iter().map(PathBuf::from).collect(),
            max_depth: dto.max_depth,
            min_size: dto.min_size,
            ..ScanConfig::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressDto {
    pub directories_scanned: usize,
    pub projects_found: usize,
    pub total_size_found: u64,
    pub current_path: String,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResultDto {
    pub projects: Vec<ProjectDto>,
    pub total_size: u64,
    pub total_cleanable: u64,
    pub duration_ms: u64,
    pub directories_scanned: usize,
}

impl From<ScanResult> for ScanResultDto {
    fn from(result: ScanResult) -> Self {
        Self {
            projects: result.projects.into_iter().map(ProjectDto::from).collect(),
            total_size: result.total_size,
            total_cleanable: result.total_cleanable,
            duration_ms: result.duration.as_millis() as u64,
            directories_scanned: result.directories_scanned,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: u64,
    pub kind: String,
    pub root: String,
    pub name: String,
    pub artifacts: Vec<ArtifactDto>,
    pub total_size: u64,
    pub cleanable_size: u64,
}

impl From<Project> for ProjectDto {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.0,
            kind: format!("{:?}", project.kind),
            root: project.root.to_string_lossy().to_string(),
            name: project.name,
            artifacts: project
                .artifacts
                .into_iter()
                .map(ArtifactDto::from)
                .collect(),
            total_size: project.total_size,
            cleanable_size: project.cleanable_size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDto {
    pub path: String,
    pub kind: String,
    pub size: u64,
    pub file_count: u64,
    pub name: String,
}

impl From<Artifact> for ArtifactDto {
    fn from(artifact: Artifact) -> Self {
        Self {
            name: artifact.name().to_string(),
            path: artifact.path.to_string_lossy().to_string(),
            kind: format!("{:?}", artifact.kind),
            size: artifact.size,
            file_count: artifact.file_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanConfigDto {
    pub use_trash: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanProgressDto {
    pub total_items: usize,
    pub completed_items: usize,
    pub bytes_cleaned: u64,
    pub current_item: String,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanSummaryDto {
    pub total_items: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub bytes_freed: u64,
    pub used_trash: bool,
    pub method_label: String,
    pub failures: Vec<CleanFailureDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanFailureDto {
    pub path: String,
    pub reason: String,
    pub is_tcc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCacheDto {
    pub name: String,
    pub id: String,
    pub icon: String,
    pub path: String,
    pub size: u64,
    pub file_count: u64,
    pub clean_command: Option<String>,
    pub description: String,
}

impl From<GlobalCache> for GlobalCacheDto {
    fn from(cache: GlobalCache) -> Self {
        Self {
            name: cache.name,
            id: cache.id.to_string(),
            icon: cache.icon.to_string(),
            path: cache.path.to_string_lossy().to_string(),
            size: cache.size,
            file_count: cache.file_count,
            clean_command: cache.clean_command.map(|s| s.to_string()),
            description: cache.description.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanableItemDto {
    pub name: String,
    pub category: String,
    pub subcategory: String,
    pub icon: String,
    pub path: String,
    pub size: u64,
    pub description: String,
    pub safety_level: String,
    pub clean_command: Option<String>,
}

impl From<CleanableItem> for CleanableItemDto {
    fn from(item: CleanableItem) -> Self {
        Self {
            name: item.name,
            category: item.category,
            subcategory: item.subcategory,
            icon: item.icon.to_string(),
            path: item.path.to_string_lossy().to_string(),
            size: item.size,
            description: item.description.to_string(),
            safety_level: match item.safe_to_delete {
                SafetyLevel::Safe => "safe".to_string(),
                SafetyLevel::SafeWithCost => "safe_with_cost".to_string(),
                SafetyLevel::Caution => "caution".to_string(),
                SafetyLevel::Dangerous => "dangerous".to_string(),
            },
            clean_command: item.clean_command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfoDto {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdaStatusDto {
    pub status: String,
    pub platform: String,
}
