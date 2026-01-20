//! Cloud CLI cleanup module
//!
//! Handles cleanup of cloud provider CLI caches:
//! - AWS CLI
//! - Google Cloud SDK (gcloud)
//! - Azure CLI
//! - Kubernetes (kubectl)
//! - Terraform
//! - Pulumi

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;

/// Cloud CLI cleaner
pub struct CloudCliCleaner {
    home: PathBuf,
}

impl CloudCliCleaner {
    /// Create a new cloud CLI cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all cloud CLI cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // AWS
        items.extend(self.detect_aws()?);

        // Google Cloud
        items.extend(self.detect_gcloud()?);

        // Azure
        items.extend(self.detect_azure()?);

        // Kubernetes
        items.extend(self.detect_kubernetes()?);

        // Terraform
        items.extend(self.detect_terraform()?);

        // Pulumi
        items.extend(self.detect_pulumi()?);

        // Helm
        items.extend(self.detect_helm()?);

        Ok(items)
    }

    /// Detect AWS CLI caches
    fn detect_aws(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let aws_paths = [
            (".aws/cli/cache", "AWS CLI Cache", SafetyLevel::Safe),
            (".aws/sso/cache", "AWS SSO Cache", SafetyLevel::Safe),
            (".aws/boto/cache", "AWS Boto Cache", SafetyLevel::Safe),
        ];

        for (rel_path, name, safety) in aws_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 5_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Cloud CLI".to_string(),
                subcategory: "AWS".to_string(),
                icon: "☁️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "AWS CLI credential and API response cache. Safe to delete.",
                safe_to_delete: safety,
                clean_command: None,
            });
        }

        // SAM CLI cache
        let sam_cache = self.home.join(".aws-sam/cache");
        if sam_cache.exists() {
            let (size, file_count) = calculate_dir_size(&sam_cache)?;
            if size > 50_000_000 {
                items.push(CleanableItem {
                    name: "AWS SAM Cache".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "AWS".to_string(),
                    icon: "☁️",
                    path: sam_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "AWS SAM build cache. Will be rebuilt on next sam build.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect Google Cloud SDK caches
    fn detect_gcloud(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let gcloud_paths = [
            (".config/gcloud/logs", "gcloud Logs", SafetyLevel::Safe),
            (".config/gcloud/cache", "gcloud Cache", SafetyLevel::Safe),
            (".config/gcloud/application_default_credentials_cache", "gcloud ADC Cache", SafetyLevel::Safe),
        ];

        for (rel_path, name, safety) in gcloud_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 10_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Cloud CLI".to_string(),
                subcategory: "Google Cloud".to_string(),
                icon: "🌩️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Google Cloud SDK cache and logs. Safe to delete.",
                safe_to_delete: safety,
                clean_command: None,
            });
        }

        // Main gcloud directory (just report size, don't suggest deleting config)
        let gcloud_dir = self.home.join(".config/gcloud");
        if gcloud_dir.exists() {
            let (size, _) = calculate_dir_size(&gcloud_dir)?;
            if size > 500_000_000 {
                // Only if very large, suggest looking at it
                items.push(CleanableItem {
                    name: "gcloud Directory".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Google Cloud".to_string(),
                    icon: "🌩️",
                    path: gcloud_dir,
                    size,
                    file_count: None,
                    last_modified: None,
                    description: "Google Cloud SDK directory. Contains config - clean subdirs only.",
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: Some("gcloud components cleanup --unused".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect Azure CLI caches
    fn detect_azure(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let azure_paths = [
            (".azure/logs", "Azure CLI Logs", SafetyLevel::Safe),
            (".azure/cliextensions", "Azure CLI Extensions Cache", SafetyLevel::SafeWithCost),
            (".azure/commands", "Azure CLI Commands Cache", SafetyLevel::Safe),
        ];

        for (rel_path, name, safety) in azure_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 10_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Cloud CLI".to_string(),
                subcategory: "Azure".to_string(),
                icon: "🔷",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Azure CLI cache and logs.",
                safe_to_delete: safety,
                clean_command: Some("az cache purge".to_string()),
            });
        }

        Ok(items)
    }

    /// Detect Kubernetes caches
    fn detect_kubernetes(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let kube_paths = [
            (".kube/cache", "kubectl Cache", SafetyLevel::Safe),
            (".kube/http-cache", "kubectl HTTP Cache", SafetyLevel::Safe),
        ];

        for (rel_path, name, safety) in kube_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 10_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Cloud CLI".to_string(),
                subcategory: "Kubernetes".to_string(),
                icon: "☸️",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Kubernetes API discovery cache. Safe to delete.",
                safe_to_delete: safety,
                clean_command: None,
            });
        }

        // Minikube
        let minikube_cache = self.home.join(".minikube/cache");
        if minikube_cache.exists() {
            let (size, file_count) = calculate_dir_size(&minikube_cache)?;
            if size > 500_000_000 {
                items.push(CleanableItem {
                    name: "Minikube Cache".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Kubernetes".to_string(),
                    icon: "☸️",
                    path: minikube_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Minikube ISO and preload images. Can be re-downloaded.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("minikube delete --purge".to_string()),
                });
            }
        }

        // Kind
        let kind_cache = self.home.join(".kind");
        if kind_cache.exists() {
            let (size, file_count) = calculate_dir_size(&kind_cache)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "Kind Cache".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Kubernetes".to_string(),
                    icon: "☸️",
                    path: kind_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Kind (Kubernetes in Docker) cache.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect Terraform caches
    fn detect_terraform(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Terraform plugin cache
        let tf_plugin_cache = self.home.join(".terraform.d/plugin-cache");
        if tf_plugin_cache.exists() {
            let (size, file_count) = calculate_dir_size(&tf_plugin_cache)?;
            if size > 100_000_000 {
                items.push(CleanableItem {
                    name: "Terraform Plugin Cache".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Terraform".to_string(),
                    icon: "🏗️",
                    path: tf_plugin_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Terraform provider plugins cache. Will be re-downloaded.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                });
            }
        }

        // OpenTofu
        let tofu_cache = self.home.join(".terraform.d");
        if tofu_cache.exists() {
            let (size, file_count) = calculate_dir_size(&tofu_cache)?;
            if size > 200_000_000 {
                items.push(CleanableItem {
                    name: "Terraform/OpenTofu Data".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Terraform".to_string(),
                    icon: "🏗️",
                    path: tofu_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Terraform/OpenTofu plugins and credentials cache.",
                    safe_to_delete: SafetyLevel::Caution,
                    clean_command: None,
                });
            }
        }

        Ok(items)
    }

    /// Detect Pulumi caches
    fn detect_pulumi(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let pulumi_dir = self.home.join(".pulumi");
        if !pulumi_dir.exists() {
            return Ok(items);
        }

        // Pulumi plugins
        let plugins = pulumi_dir.join("plugins");
        if plugins.exists() {
            let (size, file_count) = calculate_dir_size(&plugins)?;
            if size > 500_000_000 {
                items.push(CleanableItem {
                    name: "Pulumi Plugins".to_string(),
                    category: "Cloud CLI".to_string(),
                    subcategory: "Pulumi".to_string(),
                    icon: "🫁",
                    path: plugins,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "Pulumi provider plugins. Can be re-downloaded.",
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("pulumi plugin rm --all".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Detect Helm caches
    fn detect_helm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let helm_paths = [
            (".cache/helm", "Helm Cache"),
            ("Library/Caches/helm", "Helm Cache (macOS)"),
        ];

        for (rel_path, name) in helm_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size < 50_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: name.to_string(),
                category: "Cloud CLI".to_string(),
                subcategory: "Helm".to_string(),
                icon: "⛵",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Helm chart cache. Will be re-downloaded.",
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }
}

impl Default for CloudCliCleaner {
    fn default() -> Self {
        Self::new().expect("CloudCliCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_cli_cleaner_creation() {
        let cleaner = CloudCliCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_cloud_cli_detection() {
        if let Some(cleaner) = CloudCliCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} cloud CLI items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
