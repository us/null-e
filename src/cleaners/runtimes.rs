//! Language Runtimes / Version Managers cleanup module
//!
//! Handles cleanup of installed language versions from version managers:
//! - Node.js: nvm, fnm, volta, n
//! - Python: pyenv, conda environments, conda package cache
//! - Ruby: rbenv, rvm
//! - Java: sdkman
//! - Rust: rustup toolchains
//! - Go: gvm, sdk

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

/// Language Runtimes cleaner
pub struct RuntimesCleaner {
    home: PathBuf,
}

impl RuntimesCleaner {
    /// Create a new runtimes cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all cleanable runtime versions
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Node.js version managers
        items.extend(self.detect_node_versions()?);

        // Python version managers
        items.extend(self.detect_python_versions()?);

        // Ruby version managers
        items.extend(self.detect_ruby_versions()?);

        // Java version managers
        items.extend(self.detect_java_versions()?);

        // Rust toolchains
        items.extend(self.detect_rust_toolchains()?);

        // Go version managers
        items.extend(self.detect_go_versions()?);

        Ok(items)
    }

    /// Determine safety level based on active status and age
    fn determine_safety(&self, is_active: bool, last_modified: Option<SystemTime>) -> SafetyLevel {
        if is_active {
            SafetyLevel::Dangerous
        } else {
            let age_days = last_modified
                .and_then(|t| t.elapsed().ok())
                .map(|d| d.as_secs() / 86400);

            match age_days {
                Some(d) if d < 30 => SafetyLevel::Caution,
                Some(_) => SafetyLevel::Safe,
                None => SafetyLevel::SafeWithCost,
            }
        }
    }

    /// Get description based on safety level
    fn get_description(&self, is_active: bool, last_modified: Option<SystemTime>) -> &'static str {
        if is_active {
            "ACTIVE VERSION - Currently in use"
        } else {
            let age_days = last_modified
                .and_then(|t| t.elapsed().ok())
                .map(|d| d.as_secs() / 86400);

            match age_days {
                Some(d) if d < 30 => "Recently used. Delete with caution",
                Some(_) => "Old version. Safe to delete",
                None => "Unknown age. Can be reinstalled if needed",
            }
        }
    }

    // ==================== Node.js ====================

    /// Detect Node.js versions from all version managers
    fn detect_node_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // nvm
        items.extend(self.detect_nvm()?);

        // fnm
        items.extend(self.detect_fnm()?);

        // volta
        items.extend(self.detect_volta()?);

        // n
        items.extend(self.detect_n()?);

        Ok(items)
    }

    /// Detect nvm installed Node versions
    fn detect_nvm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let nvm_dir = std::env::var("NVM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".nvm"));

        let versions_path = nvm_dir.join("versions/node");
        if !versions_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version from alias/default symlink
        let active_version = self.detect_nvm_active(&nvm_dir);

        if let Ok(entries) = std::fs::read_dir(&versions_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let version = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 10_000_000 {
                    // Skip tiny installs
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| version.contains(v) || v.contains(&version))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Node.js {} (nvm)", version),
                    category: "Node.js".to_string(),
                    subcategory: "nvm".to_string(),
                    icon: "📦",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("nvm uninstall {}", version)),
                });
            }
        }

        Ok(items)
    }

    /// Detect nvm active version
    fn detect_nvm_active(&self, nvm_dir: &PathBuf) -> Option<String> {
        let default_alias = nvm_dir.join("alias/default");
        if default_alias.exists() {
            if let Ok(content) = std::fs::read_to_string(&default_alias) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    /// Detect fnm installed Node versions
    fn detect_fnm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // fnm paths vary by OS
        #[cfg(target_os = "macos")]
        let fnm_path = self.home.join("Library/Application Support/fnm/node-versions");
        #[cfg(target_os = "linux")]
        let fnm_path = self.home.join(".local/share/fnm/node-versions");
        #[cfg(target_os = "windows")]
        let fnm_path = self.home.join("AppData/Local/fnm/node-versions");
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let fnm_path = self.home.join(".local/share/fnm/node-versions");

        if !fnm_path.exists() {
            return Ok(vec![]);
        }

        // Try to get active version via fnm current
        let active_version = self.detect_fnm_active();

        if let Ok(entries) = std::fs::read_dir(&fnm_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let version = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 10_000_000 {
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| version.contains(v) || v.contains(&version))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Node.js {} (fnm)", version),
                    category: "Node.js".to_string(),
                    subcategory: "fnm".to_string(),
                    icon: "📦",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("fnm uninstall {}", version)),
                });
            }
        }

        Ok(items)
    }

    /// Detect fnm active version
    fn detect_fnm_active(&self) -> Option<String> {
        Command::new("fnm")
            .args(["current"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Detect volta installed Node versions
    fn detect_volta(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let volta_path = self.home.join(".volta/tools/image/node");
        if !volta_path.exists() {
            return Ok(vec![]);
        }

        // Try to read platform.json for active version
        let active_version = self.detect_volta_active();

        if let Ok(entries) = std::fs::read_dir(&volta_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let version = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 10_000_000 {
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| version == *v)
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Node.js {} (volta)", version),
                    category: "Node.js".to_string(),
                    subcategory: "volta".to_string(),
                    icon: "📦",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: None, // volta doesn't have uninstall command
                });
            }
        }

        Ok(items)
    }

    /// Detect volta active version from platform.json
    fn detect_volta_active(&self) -> Option<String> {
        let platform_json = self.home.join(".volta/tools/user/platform.json");
        if platform_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&platform_json) {
                // Simple JSON parsing for "node": { "runtime": "version" }
                if let Some(idx) = content.find("\"runtime\"") {
                    let after = &content[idx..];
                    if let Some(colon) = after.find(':') {
                        let value_part = &after[colon + 1..];
                        if let Some(quote_start) = value_part.find('"') {
                            let version_start = &value_part[quote_start + 1..];
                            if let Some(quote_end) = version_start.find('"') {
                                return Some(version_start[..quote_end].to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect n installed Node versions
    fn detect_n(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // n installs to /usr/local/n/versions/node by default
        let n_paths = [
            PathBuf::from("/usr/local/n/versions/node"),
            std::env::var("N_PREFIX")
                .map(|p| PathBuf::from(p).join("n/versions/node"))
                .unwrap_or_else(|_| self.home.join("n/versions/node")),
        ];

        // Detect active version via symlink
        let active_version = self.detect_n_active();

        for n_path in n_paths {
            if !n_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&n_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let version = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 10_000_000 {
                        continue;
                    }

                    let last_modified = get_mtime(&path);
                    let is_active = active_version
                        .as_ref()
                        .map(|v| version == *v)
                        .unwrap_or(false);

                    items.push(CleanableItem {
                        name: format!("Node.js {} (n)", version),
                        category: "Node.js".to_string(),
                        subcategory: "n".to_string(),
                        icon: "📦",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified,
                        description: self.get_description(is_active, last_modified),
                        safe_to_delete: self.determine_safety(is_active, last_modified),
                        clean_command: Some(format!("n rm {}", version)),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect n active version via node --version
    fn detect_n_active(&self) -> Option<String> {
        Command::new("node")
            .args(["--version"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().trim_start_matches('v').to_string())
                } else {
                    None
                }
            })
    }

    // ==================== Python ====================

    /// Detect Python versions from all version managers
    fn detect_python_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // pyenv
        items.extend(self.detect_pyenv()?);

        // conda environments
        items.extend(self.detect_conda_envs()?);

        // conda package cache
        items.extend(self.detect_conda_pkgs()?);

        Ok(items)
    }

    /// Detect pyenv installed Python versions
    fn detect_pyenv(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let pyenv_root = std::env::var("PYENV_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".pyenv"));

        let versions_path = pyenv_root.join("versions");
        if !versions_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version from .pyenv/version file
        let active_version = self.detect_pyenv_active(&pyenv_root);

        if let Ok(entries) = std::fs::read_dir(&versions_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let version = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 50_000_000 {
                    // Python installs are usually ~200MB+
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| version == *v)
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Python {} (pyenv)", version),
                    category: "Python".to_string(),
                    subcategory: "pyenv".to_string(),
                    icon: "🐍",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("pyenv uninstall -f {}", version)),
                });
            }
        }

        Ok(items)
    }

    /// Detect pyenv active version from version file
    fn detect_pyenv_active(&self, pyenv_root: &PathBuf) -> Option<String> {
        let version_file = pyenv_root.join("version");
        if version_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&version_file) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    /// Detect conda environments
    fn detect_conda_envs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Conda paths vary by platform and installation
        #[cfg(target_os = "windows")]
        let conda_bases: Vec<PathBuf> = vec![
            self.home.join("Anaconda3"),
            self.home.join("miniconda3"),
            self.home.join("Miniforge3"),
        ];
        #[cfg(not(target_os = "windows"))]
        let conda_bases: Vec<PathBuf> = vec![
            self.home.join("anaconda3"),
            self.home.join("miniconda3"),
            self.home.join("miniforge3"),
            self.home.join("mambaforge"),
            PathBuf::from("/opt/anaconda3"),
            PathBuf::from("/opt/miniconda3"),
        ];

        // Try to get active environment
        let active_env = self.detect_conda_active();

        for conda_base in conda_bases {
            let envs_path = conda_base.join("envs");
            if !envs_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&envs_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let env_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 50_000_000 {
                        continue;
                    }

                    let last_modified = get_mtime(&path);
                    let is_active = active_env
                        .as_ref()
                        .map(|v| env_name == *v)
                        .unwrap_or(false);

                    items.push(CleanableItem {
                        name: format!("Conda env: {}", env_name),
                        category: "Python".to_string(),
                        subcategory: "conda".to_string(),
                        icon: "🐍",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified,
                        description: self.get_description(is_active, last_modified),
                        safe_to_delete: self.determine_safety(is_active, last_modified),
                        clean_command: Some(format!("conda remove -n {} --all -y", env_name)),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect conda active environment
    fn detect_conda_active(&self) -> Option<String> {
        std::env::var("CONDA_DEFAULT_ENV").ok()
    }

    /// Detect conda package cache
    fn detect_conda_pkgs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        #[cfg(target_os = "windows")]
        let conda_bases: Vec<PathBuf> = vec![
            self.home.join("Anaconda3"),
            self.home.join("miniconda3"),
            self.home.join("Miniforge3"),
        ];
        #[cfg(not(target_os = "windows"))]
        let conda_bases: Vec<PathBuf> = vec![
            self.home.join("anaconda3"),
            self.home.join("miniconda3"),
            self.home.join("miniforge3"),
            self.home.join("mambaforge"),
        ];

        for conda_base in conda_bases {
            let pkgs_path = conda_base.join("pkgs");
            if !pkgs_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&pkgs_path)?;
            if size < 500_000_000 {
                // 500MB minimum
                continue;
            }

            items.push(CleanableItem {
                name: "Conda Package Cache".to_string(),
                category: "Python".to_string(),
                subcategory: "conda".to_string(),
                icon: "🐍",
                path: pkgs_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: "Conda package cache. Safe to clean, packages will be re-downloaded.",
                safe_to_delete: SafetyLevel::Safe,
                clean_command: Some("conda clean --all -y".to_string()),
            });
        }

        Ok(items)
    }

    // ==================== Ruby ====================

    /// Detect Ruby versions from version managers
    fn detect_ruby_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // rbenv
        items.extend(self.detect_rbenv()?);

        // rvm
        items.extend(self.detect_rvm()?);

        Ok(items)
    }

    /// Detect rbenv installed Ruby versions
    fn detect_rbenv(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let rbenv_root = std::env::var("RBENV_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".rbenv"));

        let versions_path = rbenv_root.join("versions");
        if !versions_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version
        let active_version = self.detect_rbenv_active(&rbenv_root);

        if let Ok(entries) = std::fs::read_dir(&versions_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let version = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 50_000_000 {
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| version == *v)
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Ruby {} (rbenv)", version),
                    category: "Ruby".to_string(),
                    subcategory: "rbenv".to_string(),
                    icon: "💎",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("rbenv uninstall -f {}", version)),
                });
            }
        }

        Ok(items)
    }

    /// Detect rbenv active version
    fn detect_rbenv_active(&self, rbenv_root: &PathBuf) -> Option<String> {
        let version_file = rbenv_root.join("version");
        if version_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&version_file) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    /// Detect rvm installed Ruby versions
    fn detect_rvm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let rvm_path = self.home.join(".rvm/rubies");
        if !rvm_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version from config/default
        let active_version = self.detect_rvm_active();

        if let Ok(entries) = std::fs::read_dir(&rvm_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Skip non-ruby directories
                if !name.starts_with("ruby-") && !name.starts_with("jruby-") {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 50_000_000 {
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| name.contains(v))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("{} (rvm)", name),
                    category: "Ruby".to_string(),
                    subcategory: "rvm".to_string(),
                    icon: "💎",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("rvm remove {}", name)),
                });
            }
        }

        Ok(items)
    }

    /// Detect rvm active version
    fn detect_rvm_active(&self) -> Option<String> {
        let default_file = self.home.join(".rvm/config/default");
        if default_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&default_file) {
                // File format: ruby-version@gemset or just ruby-version
                let version = content
                    .trim()
                    .split('@')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    // ==================== Java ====================

    /// Detect Java versions from sdkman
    fn detect_java_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let sdkman_dir = std::env::var("SDKMAN_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".sdkman"));

        let java_path = sdkman_dir.join("candidates/java");
        if !java_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version via current symlink
        let active_version = self.detect_sdkman_active(&java_path);

        if let Ok(entries) = std::fs::read_dir(&java_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Skip the current symlink
                if name == "current" {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 100_000_000 {
                    // JDKs are usually 200MB+
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| name == *v)
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Java {} (sdkman)", name),
                    category: "Java".to_string(),
                    subcategory: "sdkman".to_string(),
                    icon: "☕",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("sdk uninstall java {}", name)),
                });
            }
        }

        Ok(items)
    }

    /// Detect sdkman active version via current symlink
    fn detect_sdkman_active(&self, java_path: &PathBuf) -> Option<String> {
        let current_link = java_path.join("current");
        if current_link.exists() {
            if let Ok(target) = std::fs::read_link(&current_link) {
                return target
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string());
            }
        }
        None
    }

    // ==================== Rust ====================

    /// Detect Rust toolchains from rustup
    fn detect_rust_toolchains(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let rustup_home = std::env::var("RUSTUP_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home.join(".rustup"));

        let toolchains_path = rustup_home.join("toolchains");
        if !toolchains_path.exists() {
            return Ok(vec![]);
        }

        // Detect active toolchain
        let active_toolchain = self.detect_rustup_active();

        if let Ok(entries) = std::fs::read_dir(&toolchains_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 100_000_000 {
                    // Rust toolchains are usually 400MB+
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_toolchain
                    .as_ref()
                    .map(|v| name.starts_with(v) || v.starts_with(&name))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Rust {}", name),
                    category: "Rust".to_string(),
                    subcategory: "rustup".to_string(),
                    icon: "🦀",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("rustup toolchain remove {}", name)),
                });
            }
        }

        Ok(items)
    }

    /// Detect rustup active toolchain
    fn detect_rustup_active(&self) -> Option<String> {
        Command::new("rustup")
            .args(["default"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok().and_then(|s| {
                        // Output is like "stable-x86_64-apple-darwin (default)"
                        s.split_whitespace().next().map(|v| v.to_string())
                    })
                } else {
                    None
                }
            })
    }

    // ==================== Go ====================

    /// Detect Go versions from version managers
    fn detect_go_versions(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // gvm
        items.extend(self.detect_gvm()?);

        // go SDK downloads
        items.extend(self.detect_go_sdk()?);

        Ok(items)
    }

    /// Detect gvm installed Go versions
    fn detect_gvm(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let gvm_path = self.home.join(".gvm/gos");
        if !gvm_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version
        let active_version = self.detect_gvm_active();

        if let Ok(entries) = std::fs::read_dir(&gvm_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 100_000_000 {
                    // Go installs are usually 400MB+
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| name.contains(v))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Go {} (gvm)", name),
                    category: "Go".to_string(),
                    subcategory: "gvm".to_string(),
                    icon: "🐹",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: Some(format!("gvm uninstall {}", name)),
                });
            }
        }

        Ok(items)
    }

    /// Detect gvm active version
    fn detect_gvm_active(&self) -> Option<String> {
        let default_file = self.home.join(".gvm/environments/default");
        if default_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&default_file) {
                // Parse environment file for gvm_go_name
                for line in content.lines() {
                    if line.starts_with("export gvm_go_name=") || line.starts_with("gvm_go_name=") {
                        let value = line.split('=').nth(1).unwrap_or("");
                        let clean = value.trim_matches('"').trim_matches('\'');
                        if !clean.is_empty() {
                            return Some(clean.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect Go SDK downloads (golang.org/dl)
    fn detect_go_sdk(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let sdk_path = self.home.join("sdk");
        if !sdk_path.exists() {
            return Ok(vec![]);
        }

        // Detect active version via go version
        let active_version = self.detect_go_active();

        if let Ok(entries) = std::fs::read_dir(&sdk_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Only match go* directories
                if !name.starts_with("go") {
                    continue;
                }

                let (size, file_count) = calculate_dir_size(&path)?;
                if size < 100_000_000 {
                    continue;
                }

                let last_modified = get_mtime(&path);
                let is_active = active_version
                    .as_ref()
                    .map(|v| name.contains(v) || v.contains(&name.trim_start_matches("go")))
                    .unwrap_or(false);

                items.push(CleanableItem {
                    name: format!("Go SDK: {}", name),
                    category: "Go".to_string(),
                    subcategory: "sdk".to_string(),
                    icon: "🐹",
                    path,
                    size,
                    file_count: Some(file_count),
                    last_modified,
                    description: self.get_description(is_active, last_modified),
                    safe_to_delete: self.determine_safety(is_active, last_modified),
                    clean_command: None, // Direct path deletion
                });
            }
        }

        Ok(items)
    }

    /// Detect active Go version
    fn detect_go_active(&self) -> Option<String> {
        Command::new("go")
            .args(["version"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok().and_then(|s| {
                        // Output: "go version go1.21.0 darwin/arm64"
                        s.split_whitespace()
                            .nth(2)
                            .map(|v| v.trim_start_matches("go").to_string())
                    })
                } else {
                    None
                }
            })
    }
}

impl Default for RuntimesCleaner {
    fn default() -> Self {
        Self::new().expect("RuntimesCleaner requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtimes_cleaner_creation() {
        let cleaner = RuntimesCleaner::new();
        assert!(cleaner.is_some());
    }

    #[test]
    fn test_runtimes_detection() {
        if let Some(cleaner) = RuntimesCleaner::new() {
            let items = cleaner.detect().unwrap();
            println!("Found {} runtime items", items.len());
            for item in &items {
                println!(
                    "  {} {} ({} bytes) - {:?}",
                    item.icon, item.name, item.size, item.safe_to_delete
                );
            }
        }
    }

    #[test]
    fn test_safety_level_determination() {
        let cleaner = RuntimesCleaner::new().unwrap();

        // Active version should be dangerous
        assert_eq!(
            cleaner.determine_safety(true, None),
            SafetyLevel::Dangerous
        );

        // Old version should be safe
        let old_time = SystemTime::now() - std::time::Duration::from_secs(60 * 86400); // 60 days
        assert_eq!(
            cleaner.determine_safety(false, Some(old_time)),
            SafetyLevel::Safe
        );

        // Recent version should be caution
        let recent_time = SystemTime::now() - std::time::Duration::from_secs(10 * 86400); // 10 days
        assert_eq!(
            cleaner.determine_safety(false, Some(recent_time)),
            SafetyLevel::Caution
        );
    }
}
