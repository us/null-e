//! ML/AI cleanup module
//!
//! Handles cleanup of machine learning development files:
//! - Hugging Face models and datasets
//! - Ollama models
//! - PyTorch model cache
//! - Keras models
//! - TensorFlow cache
//! - Jupyter checkpoints

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// ML/AI cleaner
pub struct MlCleaner {
    home: PathBuf,
}

impl MlCleaner {
    /// Create a new ML cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all ML cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Hugging Face
        items.extend(self.detect_huggingface()?);

        // Ollama
        items.extend(self.detect_ollama()?);

        // PyTorch
        items.extend(self.detect_pytorch()?);

        // Keras
        items.extend(self.detect_keras()?);

        // TensorFlow
        items.extend(self.detect_tensorflow()?);

        // Jupyter
        items.extend(self.detect_jupyter()?);

        // LM Studio
        items.extend(self.detect_lmstudio()?);

        // GPT4All
        items.extend(self.detect_gpt4all()?);

        Ok(items)
    }

    /// Detect Hugging Face cache
    fn detect_huggingface(&self) -> Result<Vec<CleanableItem>> {
        let hf_paths = [
            (".cache/huggingface/hub", "HF Models"),
            (".cache/huggingface/datasets", "HF Datasets"),
            (".cache/huggingface/transformers", "HF Transformers"),
        ];

        let mut items = Vec::new();

        for (rel_path, name) in hf_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            // List individual models/datasets
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if !entry_path.is_dir() {
                        continue;
                    }

                    let entry_name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip hidden files and special directories
                    if entry_name.starts_with('.') || entry_name == "version.txt" {
                        continue;
                    }

                    let (size, file_count) = calculate_dir_size(&entry_path)?;
                    if size < 10_000_000 {
                        // Skip if less than 10MB
                        continue;
                    }

                    // Parse model name from directory structure
                    let display_name = entry_name
                        .replace("models--", "")
                        .replace("datasets--", "")
                        .replace("--", "/");

                    items.push(CleanableItem {
                        name: format!("{}: {}", name, display_name),
                        category: "ML/AI".to_string(),
                        subcategory: "Hugging Face".to_string(),
                        icon: "🤗",
                        path: entry_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: Cow::Borrowed(
                            "Downloaded ML model or dataset. Can be re-downloaded.",
                        ),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: None,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Ollama models
    fn detect_ollama(&self) -> Result<Vec<CleanableItem>> {
        let ollama_path = self.home.join(".ollama/models");

        if !ollama_path.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Ollama stores models in blobs and manifests
        let blobs_path = ollama_path.join("blobs");
        let manifests_path = ollama_path.join("manifests");

        // Get model names from manifests
        if manifests_path.exists() {
            self.scan_ollama_manifests(&manifests_path, &blobs_path, &mut items)?;
        }

        // If no manifests, just report total blobs size
        if items.is_empty() && blobs_path.exists() {
            let (size, file_count) = calculate_dir_size(&blobs_path)?;
            if size > 0 {
                items.push(CleanableItem {
                    name: "Ollama Models (all)".to_string(),
                    category: "ML/AI".to_string(),
                    subcategory: "Ollama".to_string(),
                    icon: "🦙",
                    path: ollama_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed(
                        "Local LLM models. Can be re-downloaded with 'ollama pull'.",
                    ),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: Some("ollama rm <model>".to_string()),
                });
            }
        }

        Ok(items)
    }

    /// Scan Ollama manifests for model info
    fn scan_ollama_manifests(
        &self,
        manifests_path: &Path,
        blobs_path: &Path,
        items: &mut Vec<CleanableItem>,
    ) -> Result<()> {
        // manifests/registry.ollama.ai/library/<model>/<tag>
        let registry_path = manifests_path.join("registry.ollama.ai/library");
        if !registry_path.exists() {
            return Ok(());
        }

        // Calculate actual blobs directory size instead of using an arbitrary multiplier
        let (blobs_size, blobs_file_count) = if blobs_path.exists() {
            calculate_dir_size(blobs_path)?
        } else {
            (0, 0)
        };

        if let Ok(models) = std::fs::read_dir(&registry_path) {
            let model_entries: Vec<_> = models.filter_map(|e| e.ok()).collect();
            let model_count = model_entries.len().max(1) as u64;

            for model in model_entries {
                let model_path = model.path();
                if !model_path.is_dir() {
                    continue;
                }

                let model_name = model_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Approximate per-model size by dividing total blobs evenly across models
                // (blobs are shared, so this is a rough but reasonable estimate)
                let estimated_size = blobs_size / model_count;
                let estimated_files = blobs_file_count / model_count;

                if estimated_size > 100_000_000 {
                    items.push(CleanableItem {
                        name: format!("Ollama: {}", model_name),
                        category: "ML/AI".to_string(),
                        subcategory: "Ollama".to_string(),
                        icon: "🦙",
                        path: model_path,
                        size: estimated_size,
                        file_count: Some(estimated_files),
                        last_modified: get_mtime(&model.path()),
                        description: Cow::Borrowed("Local LLM model. Use 'ollama rm' to remove."),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: Some(format!("ollama rm {}", model_name)),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect PyTorch cache
    fn detect_pytorch(&self) -> Result<Vec<CleanableItem>> {
        let torch_paths = [
            (".cache/torch", "PyTorch Cache"),
            (".cache/torch/hub", "PyTorch Hub Models"),
        ];

        let mut items = Vec::new();

        for (rel_path, name) in torch_paths {
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
                category: "ML/AI".to_string(),
                subcategory: "PyTorch".to_string(),
                icon: "🔥",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("PyTorch model cache. Can be re-downloaded."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Keras cache
    fn detect_keras(&self) -> Result<Vec<CleanableItem>> {
        let keras_path = self.home.join(".keras");

        if !keras_path.exists() {
            return Ok(vec![]);
        }

        let models_path = keras_path.join("models");
        if models_path.exists() {
            let (size, file_count) = calculate_dir_size(&models_path)?;
            if size > 10_000_000 {
                return Ok(vec![CleanableItem {
                    name: "Keras Models".to_string(),
                    category: "ML/AI".to_string(),
                    subcategory: "Keras".to_string(),
                    icon: "🧠",
                    path: models_path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed("Keras pre-trained models. Can be re-downloaded."),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                }]);
            }
        }

        Ok(vec![])
    }

    /// Detect TensorFlow cache
    fn detect_tensorflow(&self) -> Result<Vec<CleanableItem>> {
        let tf_paths = [
            (".tensorflow", "TensorFlow Cache"),
            (".cache/tensorflow", "TensorFlow Hub Cache"),
        ];

        let mut items = Vec::new();

        for (rel_path, name) in tf_paths {
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
                category: "ML/AI".to_string(),
                subcategory: "TensorFlow".to_string(),
                icon: "📊",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("TensorFlow model cache. Can be re-downloaded."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Jupyter cache and checkpoints
    fn detect_jupyter(&self) -> Result<Vec<CleanableItem>> {
        let jupyter_paths = [
            (".cache/jupyter", "Jupyter Cache"),
            (".jupyter", "Jupyter Config & Data"),
            (".local/share/jupyter", "Jupyter Data"),
        ];

        let mut items = Vec::new();

        for (rel_path, name) in jupyter_paths {
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
                category: "ML/AI".to_string(),
                subcategory: "Jupyter".to_string(),
                icon: "📓",
                path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Jupyter notebook cache and runtime data."),
                safe_to_delete: SafetyLevel::Safe,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect LM Studio models
    fn detect_lmstudio(&self) -> Result<Vec<CleanableItem>> {
        let lmstudio_path = self.home.join(".lmstudio/models");
        let alt_path = self.home.join(".cache/lm-studio");

        // Use whichever path actually exists
        let active_path = if lmstudio_path.exists() {
            lmstudio_path
        } else if alt_path.exists() {
            alt_path
        } else {
            return Ok(vec![]);
        };

        let (size, file_count) = calculate_dir_size(&active_path)?;
        if size == 0 {
            return Ok(vec![]);
        }

        Ok(vec![CleanableItem {
            name: "LM Studio Models".to_string(),
            category: "ML/AI".to_string(),
            subcategory: "LM Studio".to_string(),
            icon: "🎯",
            path: active_path,
            size,
            file_count: Some(file_count),
            last_modified: None,
            description: Cow::Borrowed("LM Studio downloaded models. Can be re-downloaded."),
            safe_to_delete: SafetyLevel::SafeWithCost,
            clean_command: None,
        }])
    }

    /// Detect GPT4All models
    fn detect_gpt4all(&self) -> Result<Vec<CleanableItem>> {
        let gpt4all_paths = [
            ".cache/gpt4all",
            "Library/Application Support/nomic.ai/GPT4All",
        ];

        for rel_path in gpt4all_paths {
            let path = self.home.join(rel_path);
            if !path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&path)?;
            if size > 100_000_000 {
                return Ok(vec![CleanableItem {
                    name: "GPT4All Models".to_string(),
                    category: "ML/AI".to_string(),
                    subcategory: "GPT4All".to_string(),
                    icon: "🤖",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: Cow::Borrowed("GPT4All downloaded models. Can be re-downloaded."),
                    safe_to_delete: SafetyLevel::SafeWithCost,
                    clean_command: None,
                }]);
            }
        }

        Ok(vec![])
    }
}

impl Default for MlCleaner {
    fn default() -> Self {
        Self::new().expect("MlCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_cleaner_creation() {
        let cleaner = MlCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_ml_detection() {
        if let Some(cleaner) = MlCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} ML items", items.len());
            for item in &items {
                println!("  {} {} ({} bytes)", item.icon, item.name, item.size);
            }
        }
    }
}
