//! Plugin registry - central management for all plugins

use super::Plugin;
use crate::core::{ProjectKind, ProjectMarker};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Central registry for all plugins
pub struct PluginRegistry {
    /// All registered plugins
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
    /// Markers indexed for fast lookup
    markers: RwLock<Vec<(ProjectMarker, Arc<dyn Plugin>)>>,
    /// Plugins indexed by project kind
    by_kind: RwLock<HashMap<ProjectKind, Vec<Arc<dyn Plugin>>>>,
    /// Quick lookup for cleanable directory names
    cleanable_dirs: RwLock<HashMap<&'static str, Vec<Arc<dyn Plugin>>>>,
}

impl PluginRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
            markers: RwLock::new(Vec::new()),
            by_kind: RwLock::new(HashMap::new()),
            cleanable_dirs: RwLock::new(HashMap::new()),
        }
    }

    /// Create registry with all built-in plugins
    pub fn with_builtins() -> Self {
        let registry = Self::new();

        for plugin in super::builtin_plugins() {
            registry.register(Arc::from(plugin));
        }

        registry
    }

    /// Register a new plugin
    pub fn register(&self, plugin: Arc<dyn Plugin>) {
        // Add markers for fast lookup
        for marker in plugin.markers() {
            self.markers.write().push((marker, Arc::clone(&plugin)));
        }

        // Index by project kind
        for kind in plugin.supported_kinds() {
            self.by_kind
                .write()
                .entry(*kind)
                .or_default()
                .push(Arc::clone(&plugin));
        }

        // Index cleanable directories
        for dir in plugin.cleanable_dirs() {
            self.cleanable_dirs
                .write()
                .entry(dir)
                .or_default()
                .push(Arc::clone(&plugin));
        }

        self.plugins.write().push(plugin);
    }

    /// Get all registered plugins
    pub fn all(&self) -> Vec<Arc<dyn Plugin>> {
        self.plugins.read().clone()
    }

    /// Get all registered markers for scanning
    pub fn all_markers(&self) -> Vec<ProjectMarker> {
        self.markers.read().iter().map(|(m, _)| m.clone()).collect()
    }

    /// Find plugins that handle a project kind
    pub fn plugins_for_kind(&self, kind: ProjectKind) -> Vec<Arc<dyn Plugin>> {
        self.by_kind.read().get(&kind).cloned().unwrap_or_default()
    }

    /// Check if a directory name is a known cleanable artifact
    pub fn is_cleanable_dir(&self, name: &str) -> bool {
        self.cleanable_dirs.read().contains_key(name)
    }

    /// Get plugins that can handle a cleanable directory
    pub fn plugins_for_cleanable_dir(&self, name: &str) -> Vec<Arc<dyn Plugin>> {
        self.cleanable_dirs
            .read()
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    /// Detect project type at path
    pub fn detect_project(&self, path: &Path) -> Option<(ProjectKind, Arc<dyn Plugin>)> {
        let plugins = self.plugins.read();

        // Collect all matches with their priorities
        let mut candidates: Vec<_> = plugins
            .iter()
            .filter_map(|p| p.detect(path).map(|k| (k, Arc::clone(p), p.priority())))
            .collect();

        // Sort by priority (descending)
        candidates.sort_by(|a, b| b.2.cmp(&a.2));

        candidates.into_iter().next().map(|(k, p, _)| (k, p))
    }

    /// Get all unique cleanable directory names
    pub fn all_cleanable_dir_names(&self) -> Vec<&'static str> {
        self.cleanable_dirs.read().keys().copied().collect()
    }

    /// Get plugin by ID
    pub fn get_by_id(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.read().iter().find(|p| p.id() == id).cloned()
    }

    /// Get number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.read().len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.plugins.read().is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_with_builtins() {
        let registry = PluginRegistry::with_builtins();
        assert!(!registry.is_empty());
        assert!(registry.len() >= 5); // At least our main plugins
    }

    #[test]
    fn test_cleanable_dirs() {
        let registry = PluginRegistry::with_builtins();
        assert!(registry.is_cleanable_dir("node_modules"));
        assert!(registry.is_cleanable_dir("target"));
        assert!(registry.is_cleanable_dir("__pycache__"));
        assert!(!registry.is_cleanable_dir("src"));
    }

    #[test]
    fn test_get_by_id() {
        let registry = PluginRegistry::with_builtins();
        let node = registry.get_by_id("node");
        assert!(node.is_some());
        assert_eq!(node.unwrap().id(), "node");
    }
}
