//! System-Wide Binary & Runtime Analyzer
//!
//! Detects all installed binaries from various sources:
//! - System paths (/usr/bin/, /bin/)
//! - Homebrew (/opt/homebrew/, /usr/local/Cellar/)
//! - Version managers (nvm, pyenv, rbenv, fnm, volta, mise, asdf)
//! - Package managers (pip, npm, cargo, pipx, uv)
//!
//! Identifies duplicates, conflicting installations, and unused version managers.

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Binary source - where a binary was installed from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinarySource {
    /// System binary (/usr/bin/, /bin/)
    System,
    /// Homebrew (Apple Silicon: /opt/homebrew/, Intel: /usr/local/Cellar/)
    Homebrew,
    /// Homebrew cask
    HomebrewCask,
    /// Cargo/Rust (~/.cargo/bin/)
    Cargo,
    /// pip install --user
    Pip,
    /// pipx (~/.local/pipx/)
    Pipx,
    /// uv installed tools (~/.local/bin/ from uv)
    Uv,
    /// npm global packages
    Npm,
    /// pyenv (~/.pyenv/)
    Pyenv,
    /// rbenv (~/.rbenv/)
    Rbenv,
    /// rvm (~/.rvm/)
    Rvm,
    /// nvm (~/.nvm/)
    Nvm,
    /// fnm (Fast Node Manager)
    Fnm,
    /// volta (~/.volta/)
    Volta,
    /// rustup (~/.rustup/)
    Rustup,
    /// sdkman (~/.sdkman/)
    Sdkman,
    /// gvm (~/.gvm/)
    Gvm,
    /// mise (~/.local/share/mise/)
    Mise,
    /// asdf (~/.asdf/)
    Asdf,
    /// Manual installation
    Manual,
    /// Unknown source
    Unknown,
}

impl BinarySource {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Homebrew => "Homebrew",
            Self::HomebrewCask => "Homebrew Cask",
            Self::Cargo => "Cargo",
            Self::Pip => "pip",
            Self::Pipx => "pipx",
            Self::Uv => "uv",
            Self::Npm => "npm",
            Self::Pyenv => "pyenv",
            Self::Rbenv => "rbenv",
            Self::Rvm => "rvm",
            Self::Nvm => "nvm",
            Self::Fnm => "fnm",
            Self::Volta => "Volta",
            Self::Rustup => "rustup",
            Self::Sdkman => "SDKMAN",
            Self::Gvm => "gvm",
            Self::Mise => "mise",
            Self::Asdf => "asdf",
            Self::Manual => "Manual",
            Self::Unknown => "Unknown",
        }
    }

    /// Is this a version manager?
    pub fn is_version_manager(&self) -> bool {
        matches!(
            self,
            Self::Pyenv
                | Self::Rbenv
                | Self::Rvm
                | Self::Nvm
                | Self::Fnm
                | Self::Volta
                | Self::Rustup
                | Self::Sdkman
                | Self::Gvm
                | Self::Mise
                | Self::Asdf
        )
    }
}

/// Binary type - what kind of binary is this?
#[derive(Debug, Clone)]
pub enum BinaryType {
    /// Actual compiled binary
    Binary,
    /// Symlink to another file
    Symlink { target: PathBuf },
    /// Shell script wrapper (shim)
    Wrapper { target: PathBuf },
    /// Hard link
    HardLink,
}

/// A single binary instance found on the system
#[derive(Debug, Clone)]
pub struct BinaryInstance {
    /// Command name (e.g., "python3")
    pub command: String,
    /// Full path to binary
    pub path: PathBuf,
    /// Resolved path after following symlinks
    pub resolved_path: PathBuf,
    /// Source of this binary
    pub source: BinarySource,
    /// Version string (if detected)
    pub version: Option<String>,
    /// Binary size in bytes
    pub binary_size: u64,
    /// Type of binary
    pub binary_type: BinaryType,
    /// Is this binary in the current PATH?
    pub in_path: bool,
    /// Is this the active/default version?
    pub is_active: bool,
}

/// Recommendation for handling duplicate binaries
#[derive(Debug, Clone)]
pub enum DuplicateRecommendation {
    /// Old versions that can be removed
    RemoveOldVersions { versions: Vec<String> },
    /// Duplicate from another source
    RemoveDuplicateSource { source: BinarySource },
    /// Conflicting version managers
    ConflictingManagers { managers: Vec<BinarySource> },
    /// Unused version manager
    UnusedVersionManager { name: String, size: u64 },
    /// Stale config in shell rc file
    StaleConfig { file: PathBuf, manager: String },
    /// Keep all instances
    KeepAll { reason: String },
}

/// A group of duplicate binaries
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// Command name
    pub command: String,
    /// All instances of this command
    pub instances: Vec<BinaryInstance>,
    /// Total size of all instances
    pub total_size: u64,
    /// Recommendation for handling
    pub recommendation: DuplicateRecommendation,
    /// Safety level for cleanup
    pub safety: SafetyLevel,
}

/// Result of binary analysis
#[derive(Debug, Default)]
pub struct BinaryAnalysisResult {
    /// All discovered binaries
    pub binaries: Vec<BinaryInstance>,
    /// Duplicate groups
    pub duplicates: Vec<DuplicateGroup>,
    /// Unused version managers
    pub unused_managers: Vec<CleanableItem>,
    /// Stale configurations
    pub stale_configs: Vec<CleanableItem>,
    /// Total potential savings
    pub potential_savings: u64,
}

/// Binary Analyzer - discovers and analyzes system binaries
pub struct BinaryAnalyzer {
    home: PathBuf,
    current_path: Vec<PathBuf>,
}

impl BinaryAnalyzer {
    /// Create a new binary analyzer
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        let path_str = std::env::var("PATH").unwrap_or_default();
        let current_path: Vec<PathBuf> = std::env::split_paths(&path_str).collect();

        Some(Self { home, current_path })
    }

    /// Perform full binary analysis
    pub fn analyze(&self) -> Result<BinaryAnalysisResult> {
        let mut result = BinaryAnalysisResult::default();

        // Discover binaries for key commands
        // Build dynamic list including all Python versions
        let mut commands: Vec<&str> = vec![
            // Python (base commands)
            "python",
            "python3",
            "python2",
            "pip",
            "pip3",
            "pipx",
            "uv",
            "ruff",
            "mypy",
            "black",
            "poetry",
            "pdm",
            // Node.js / JavaScript
            "node",
            "npm",
            "npx",
            "corepack",
            "yarn",
            "pnpm",
            "bun",
            "bunx",
            "deno",
            "tsx",
            "ts-node",
            // Ruby
            "ruby",
            "gem",
            "bundle",
            "bundler",
            "rake",
            "rails",
            // Go
            "go",
            "gofmt",
            "gopls",
            // Rust
            "rustc",
            "cargo",
            "rustup",
            "rustfmt",
            "clippy-driver",
            // Java / JVM
            "java",
            "javac",
            "kotlin",
            "kotlinc",
            "scala",
            "scalac",
            "sbt",
            "gradle",
            "mvn",
            "groovy",
            "clojure",
            "clj",
            // .NET / C#
            "dotnet",
            "csc",
            "fsc",
            "nuget",
            // PHP
            "php",
            "composer",
            "pecl",
            "phpunit",
            "laravel",
            // Perl
            "perl",
            "cpan",
            "cpanm",
            // Elixir / Erlang
            "elixir",
            "erl",
            "erlc",
            "mix",
            "iex",
            "rebar3",
            // Swift
            "swift",
            "swiftc",
            // Haskell
            "ghc",
            "ghci",
            "cabal",
            "stack",
            // Lua
            "lua",
            "luajit",
            "luarocks",
            // R
            "R",
            "Rscript",
            // Julia
            "julia",
            // Zig
            "zig",
            // Nim
            "nim",
            "nimble",
            // Crystal
            "crystal",
            "shards",
            // OCaml
            "ocaml",
            "opam",
            "dune",
            // Common dev tools
            "git",
            "vim",
            "nvim",
            "emacs",
            "code",
            "cursor",
            // Build tools
            "make",
            "cmake",
            "ninja",
            "meson",
            // Container tools
            "docker",
            "podman",
            "kubectl",
            "helm",
        ];

        // Add Python version-specific commands (3.7 through 3.14)
        let python_versions: Vec<String> = (7..=14)
            .map(|v| format!("python3.{}", v))
            .collect();
        for v in &python_versions {
            commands.push(v.as_str());
        }

        // Convert to slice for discover_binaries
        let commands: Vec<&str> = commands;

        result.binaries = self.discover_binaries(&commands);

        // Find duplicates
        result.duplicates = self.find_duplicates(&result.binaries);

        // Detect unused version managers
        result.unused_managers = self.detect_unused_version_managers()?;

        // Detect stale configs
        result.stale_configs = self.detect_stale_configs()?;

        // Calculate potential savings
        result.potential_savings = result
            .duplicates
            .iter()
            .filter(|d| d.safety != SafetyLevel::Dangerous)
            .map(|d| {
                // Don't count active/system binaries
                d.instances
                    .iter()
                    .filter(|i| !i.is_active && i.source != BinarySource::System)
                    .map(|i| i.binary_size)
                    .sum::<u64>()
            })
            .sum::<u64>()
            + result.unused_managers.iter().map(|m| m.size).sum::<u64>();

        Ok(result)
    }

    /// Discover binaries for given commands using `which -a`
    fn discover_binaries(&self, commands: &[&str]) -> Vec<BinaryInstance> {
        let mut binaries = Vec::new();

        for command in commands {
            if let Ok(output) = Command::new("which").arg("-a").arg(command).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let path = PathBuf::from(line.trim());
                        if path.exists() {
                            if let Some(instance) = self.analyze_binary(command, &path) {
                                binaries.push(instance);
                            }
                        }
                    }
                }
            }
        }

        binaries
    }

    /// Analyze a single binary
    fn analyze_binary(&self, command: &str, path: &Path) -> Option<BinaryInstance> {
        let resolved_path = self.resolve_symlink_chain(path);
        let source = self.determine_source(&resolved_path);
        let version = self.get_version(command, path);
        let binary_size = std::fs::metadata(&resolved_path).ok()?.len();
        let binary_type = self.determine_binary_type(path);
        let in_path = self.is_in_current_path(path);
        let is_active = self.is_active_version(command, path);

        Some(BinaryInstance {
            command: command.to_string(),
            path: path.to_path_buf(),
            resolved_path,
            source,
            version,
            binary_size,
            binary_type,
            in_path,
            is_active,
        })
    }

    /// Resolve symlink chain to find actual binary
    fn resolve_symlink_chain(&self, path: &Path) -> PathBuf {
        let mut current = path.to_path_buf();
        let mut visited = HashSet::new();

        while let Ok(target) = std::fs::read_link(&current) {
            if visited.contains(&current) {
                // Circular symlink
                break;
            }
            visited.insert(current.clone());

            // Handle relative symlinks
            if target.is_relative() {
                if let Some(parent) = current.parent() {
                    current = parent.join(&target);
                } else {
                    current = target;
                }
            } else {
                current = target;
            }

            // Canonicalize to resolve ..
            if let Ok(canonical) = current.canonicalize() {
                current = canonical;
            }
        }

        current
    }

    /// Determine the source of a binary based on its path
    fn determine_source(&self, path: &Path) -> BinarySource {
        let path_str = path.to_string_lossy();

        // Homebrew (Apple Silicon)
        if path_str.starts_with("/opt/homebrew/") {
            return if path_str.contains("/Caskroom/") {
                BinarySource::HomebrewCask
            } else {
                BinarySource::Homebrew
            };
        }

        // Homebrew (Intel Mac)
        if path_str.starts_with("/usr/local/Cellar/")
            || path_str.starts_with("/usr/local/opt/")
            || (path_str.starts_with("/usr/local/bin/")
                && self.is_homebrew_managed(&PathBuf::from(path_str.to_string())))
        {
            return BinarySource::Homebrew;
        }

        // System paths
        if path_str.starts_with("/usr/bin/")
            || path_str.starts_with("/bin/")
            || path_str.starts_with("/usr/sbin/")
            || path_str.starts_with("/sbin/")
        {
            return BinarySource::System;
        }

        // Cargo/Rust
        if path_str.contains(".cargo/bin") {
            return BinarySource::Cargo;
        }

        // Rustup
        if path_str.contains(".rustup/") {
            return BinarySource::Rustup;
        }

        // Python version managers
        if path_str.contains(".pyenv/") {
            return BinarySource::Pyenv;
        }

        // Ruby version managers
        if path_str.contains(".rbenv/") {
            return BinarySource::Rbenv;
        }
        if path_str.contains(".rvm/") {
            return BinarySource::Rvm;
        }

        // Node version managers
        if path_str.contains(".nvm/") {
            return BinarySource::Nvm;
        }
        if path_str.contains(".fnm/") || path_str.contains("fnm/node-versions") {
            return BinarySource::Fnm;
        }
        if path_str.contains(".volta/") {
            return BinarySource::Volta;
        }

        // Modern version managers
        if path_str.contains(".local/share/mise/") || path_str.contains("mise/installs") {
            return BinarySource::Mise;
        }
        if path_str.contains(".asdf/") {
            return BinarySource::Asdf;
        }

        // Java version managers
        if path_str.contains(".sdkman/") {
            return BinarySource::Sdkman;
        }

        // Go version managers
        if path_str.contains(".gvm/") {
            return BinarySource::Gvm;
        }

        // uv managed Python
        if path_str.contains(".local/share/uv/") {
            return BinarySource::Uv;
        }

        // Bun
        if path_str.contains(".bun/") {
            return BinarySource::Manual; // Bun-installed binaries
        }

        // Deno
        if path_str.contains(".deno/") {
            return BinarySource::Manual; // Deno-installed
        }

        // .NET
        if path_str.contains(".dotnet/") {
            return BinarySource::Manual; // .NET SDK
        }

        // Haskell
        if path_str.contains(".ghcup/") || path_str.contains(".cabal/") || path_str.contains(".stack/") {
            return BinarySource::Manual; // Haskell toolchain
        }

        // pip/pipx/uv installed in ~/.local/bin
        if path_str.contains(".local/bin") {
            // Check if from pipx
            let pipx_venvs = self.home.join(".local/pipx/venvs");
            if pipx_venvs.exists() {
                // Check if this binary is from a pipx venv
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if pipx_venvs.join(&*name_str).exists() {
                        return BinarySource::Pipx;
                    }
                }
            }

            // Check if uv tool
            let uv_tools = self.home.join(".local/share/uv/tools");
            if uv_tools.exists() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Check in uv tools directory
                    if std::fs::read_dir(&uv_tools)
                        .map(|entries| entries.filter_map(|e| e.ok()).any(|e| {
                            e.file_name().to_string_lossy() == *name_str
                        }))
                        .unwrap_or(false)
                    {
                        return BinarySource::Uv;
                    }
                }
            }

            // Could be uv or pip
            return BinarySource::Pip;
        }

        // npm global
        if path_str.contains("node_modules") || path_str.contains(".npm") {
            return BinarySource::Npm;
        }

        // Go binaries in GOPATH/bin
        if path_str.contains("/go/bin/") {
            return BinarySource::Manual; // go install
        }

        BinarySource::Unknown
    }

    /// Check if a /usr/local/bin path is managed by Homebrew
    fn is_homebrew_managed(&self, path: &Path) -> bool {
        if let Ok(target) = std::fs::read_link(path) {
            let target_str = target.to_string_lossy();
            target_str.contains("Cellar") || target_str.contains("/opt/homebrew/")
        } else {
            false
        }
    }

    /// Determine the type of binary
    fn determine_binary_type(&self, path: &Path) -> BinaryType {
        // Check if symlink
        if let Ok(target) = std::fs::read_link(path) {
            return BinaryType::Symlink { target };
        }

        // Check if it's a shim/wrapper script
        if self.is_wrapper_script(path) {
            let resolved = self.resolve_symlink_chain(path);
            return BinaryType::Wrapper { target: resolved };
        }

        BinaryType::Binary
    }

    /// Check if a file is a wrapper/shim script
    fn is_wrapper_script(&self, path: &Path) -> bool {
        // Read first few bytes to check for shebang
        if let Ok(content) = std::fs::read(path) {
            if content.len() >= 2 && &content[0..2] == b"#!" {
                // It's a script - check for common shim patterns
                let text = String::from_utf8_lossy(&content);
                return text.contains("shim")
                    || text.contains("exec")
                    || text.contains("pyenv")
                    || text.contains("rbenv")
                    || text.contains("asdf");
            }
        }
        false
    }

    /// Get version of a binary
    fn get_version(&self, command: &str, path: &Path) -> Option<String> {
        // Try common version flags
        let version_flags = ["--version", "-version", "-v", "version"];

        for flag in version_flags {
            if let Ok(output) = Command::new(path).arg(flag).output() {
                if output.status.success() || !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(version) = self.extract_version(&stdout, command) {
                        return Some(version);
                    }
                }
                // Try stderr too
                let stderr = String::from_utf8_lossy(&output.stderr);
                if let Some(version) = self.extract_version(&stderr, command) {
                    return Some(version);
                }
            }
        }

        None
    }

    /// Extract version number from version output
    fn extract_version(&self, text: &str, _command: &str) -> Option<String> {
        // Common patterns:
        // "Python 3.13.5"
        // "node v22.22.0"
        // "ruby 3.4.7"
        // "go version go1.21.0"

        let text = text.trim();
        let first_line = text.lines().next()?;

        // Try to find version number pattern
        let version_re =
            regex::Regex::new(r"[vV]?(\d+\.\d+(?:\.\d+)?(?:-[a-zA-Z0-9.]+)?)").ok()?;

        if let Some(caps) = version_re.captures(first_line) {
            return Some(caps.get(1)?.as_str().to_string());
        }

        None
    }

    /// Check if path is in current PATH
    fn is_in_current_path(&self, path: &Path) -> bool {
        if let Some(parent) = path.parent() {
            self.current_path.iter().any(|p| p == parent)
        } else {
            false
        }
    }

    /// Check if this is the active version (first in PATH)
    fn is_active_version(&self, command: &str, path: &Path) -> bool {
        // Run `which` to get the active version
        if let Ok(output) = Command::new("which").arg(command).output() {
            if output.status.success() {
                let active_path = String::from_utf8_lossy(&output.stdout);
                let active_path = PathBuf::from(active_path.trim());
                return active_path == path;
            }
        }
        false
    }

    /// Find duplicate binaries
    fn find_duplicates(&self, binaries: &[BinaryInstance]) -> Vec<DuplicateGroup> {
        // Group by command name
        let mut groups: HashMap<String, Vec<&BinaryInstance>> = HashMap::new();

        for binary in binaries {
            groups
                .entry(binary.command.clone())
                .or_default()
                .push(binary);
        }

        let mut duplicates = Vec::new();

        for (command, instances) in groups {
            if instances.len() <= 1 {
                continue; // Not a duplicate
            }

            // Analyze duplicate type
            let sources: HashSet<BinarySource> = instances.iter().map(|i| i.source).collect();

            let versions: HashSet<&str> = instances
                .iter()
                .filter_map(|i| i.version.as_deref())
                .collect();

            let has_system = instances
                .iter()
                .any(|i| i.source == BinarySource::System);
            let has_active = instances.iter().any(|i| i.is_active);

            // Determine recommendation
            let recommendation = if sources.len() > 1 {
                // Same tool from multiple sources
                let managers: Vec<BinarySource> = sources
                    .iter()
                    .filter(|s| s.is_version_manager() || **s == BinarySource::Homebrew)
                    .copied()
                    .collect();

                if managers.len() > 1 {
                    DuplicateRecommendation::ConflictingManagers { managers }
                } else if let Some(non_active_source) = sources
                    .iter()
                    .find(|s| **s != BinarySource::System && !instances.iter().any(|i| i.is_active && i.source == **s))
                {
                    DuplicateRecommendation::RemoveDuplicateSource {
                        source: *non_active_source,
                    }
                } else {
                    DuplicateRecommendation::KeepAll {
                        reason: "Multiple sources with different purposes".to_string(),
                    }
                }
            } else if versions.len() > 1 {
                // Multiple versions from same source
                let old_versions: Vec<String> = instances
                    .iter()
                    .filter(|i| !i.is_active)
                    .filter_map(|i| i.version.clone())
                    .collect();

                DuplicateRecommendation::RemoveOldVersions {
                    versions: old_versions,
                }
            } else {
                DuplicateRecommendation::KeepAll {
                    reason: "Same version, same source".to_string(),
                }
            };

            // Determine safety level
            let safety = if has_system && instances.len() == 1 {
                SafetyLevel::Dangerous
            } else if has_active {
                // Can remove non-active duplicates
                let non_active_count = instances.iter().filter(|i| !i.is_active).count();
                if non_active_count > 0 {
                    SafetyLevel::SafeWithCost
                } else {
                    SafetyLevel::Dangerous
                }
            } else if sources.iter().any(|s| s.is_version_manager()) {
                SafetyLevel::Caution
            } else {
                SafetyLevel::Safe
            };

            let total_size = instances.iter().map(|i| i.binary_size).sum();

            duplicates.push(DuplicateGroup {
                command,
                instances: instances.into_iter().cloned().collect(),
                total_size,
                recommendation,
                safety,
            });
        }

        // Sort by size
        duplicates.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        duplicates
    }

    /// Check if any common project directories contain .tool-versions files
    /// This helps avoid false positives when detecting "unused" version managers
    fn has_tool_versions_in_projects(&self) -> bool {
        // Common project directory names
        let project_dirs = [
            "coding", "projects", "dev", "work", "code", "src",
            "Documents", "Developer", "workspace", "repos", "git",
        ];

        for dir_name in project_dirs {
            let dir = self.home.join(dir_name);
            if dir.exists() && dir.is_dir() {
                // Quick shallow check (max 2 levels deep to avoid long scans)
                if self.find_tool_versions_shallow(&dir, 2) {
                    return true;
                }
            }
        }

        false
    }

    /// Shallow search for .tool-versions (max depth levels)
    fn find_tool_versions_shallow(&self, dir: &Path, max_depth: usize) -> bool {
        if max_depth == 0 {
            return false;
        }

        // Check current directory
        if dir.join(".tool-versions").exists() || dir.join(".mise.toml").exists() {
            return true;
        }

        // Check subdirectories
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden dirs and common non-project dirs
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "venv" {
                            continue;
                        }
                    }
                    if self.find_tool_versions_shallow(&path, max_depth - 1) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Detect unused version managers
    fn detect_unused_version_managers(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Check mise
        let mise_dir = self.home.join(".local/share/mise");
        if mise_dir.exists() {
            let tool_versions = self.home.join(".tool-versions");
            let mise_toml = self.home.join(".mise.toml");
            let mise_config = self.home.join(".config/mise/config.toml");

            // Also check common project directories for .tool-versions
            let has_project_config = self.has_tool_versions_in_projects();

            if !tool_versions.exists() && !mise_toml.exists() && !mise_config.exists() && !has_project_config {
                // mise installed but not configured anywhere
                let (size, file_count) = calculate_dir_size(&mise_dir)?;
                if size > 10_000_000 {
                    // 10MB minimum
                    items.push(CleanableItem {
                        name: "mise (possibly unused)".to_string(),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Unused Manager".to_string(),
                        icon: "🔧",
                        path: mise_dir,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "mise installed but no config found in ~ or common project dirs",
                        // Changed to Caution - user should verify no projects use it
                        safe_to_delete: SafetyLevel::Caution,
                        clean_command: Some("rm -rf ~/.local/share/mise ~/.local/bin/mise".to_string()),
                    });
                }
            }
        }

        // Check asdf
        let asdf_dir = self.home.join(".asdf");
        if asdf_dir.exists() {
            let tool_versions = self.home.join(".tool-versions");
            let asdf_installs = asdf_dir.join("installs");

            // Check if any tools are actually installed
            let has_installs = asdf_installs.exists()
                && std::fs::read_dir(&asdf_installs)
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false);

            // Also check common project directories
            let has_project_config = self.has_tool_versions_in_projects();

            if !tool_versions.exists() && !has_installs && !has_project_config {
                let (size, file_count) = calculate_dir_size(&asdf_dir)?;
                if size > 10_000_000 {
                    items.push(CleanableItem {
                        name: "asdf (possibly unused)".to_string(),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Unused Manager".to_string(),
                        icon: "🔧",
                        path: asdf_dir,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "asdf installed but no tools or config found",
                        // Changed to Caution - user should verify
                        safe_to_delete: SafetyLevel::Caution,
                        clean_command: Some("rm -rf ~/.asdf".to_string()),
                    });
                }
            }
        }

        // Check volta
        let volta_dir = self.home.join(".volta");
        if volta_dir.exists() {
            let tools_dir = volta_dir.join("tools/image");
            let has_tools = tools_dir.exists()
                && std::fs::read_dir(&tools_dir)
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false);

            if !has_tools {
                let (size, file_count) = calculate_dir_size(&volta_dir)?;
                if size > 10_000_000 {
                    items.push(CleanableItem {
                        name: "Volta (unused)".to_string(),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Unused Manager".to_string(),
                        icon: "⚡",
                        path: volta_dir,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Volta installed but no Node.js versions configured",
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: Some("rm -rf ~/.volta".to_string()),
                    });
                }
            }
        }

        // Check for Homebrew Node alongside version manager Node
        self.check_homebrew_node_duplicate(&mut items)?;

        // Check for Homebrew Python alongside pyenv
        self.check_homebrew_python_duplicate(&mut items)?;

        // Detect old Homebrew Cellar versions
        self.detect_homebrew_old_versions(&mut items)?;

        // Detect uv cache and tools
        self.detect_uv_cache(&mut items)?;

        Ok(items)
    }

    /// Check if Homebrew Python is installed alongside pyenv
    fn check_homebrew_python_duplicate(&self, items: &mut Vec<CleanableItem>) -> Result<()> {
        let has_pyenv = self.home.join(".pyenv/versions").exists()
            && std::fs::read_dir(self.home.join(".pyenv/versions"))
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);

        if !has_pyenv {
            return Ok(());
        }

        // Check for Homebrew Python versions
        let cellar_base = if PathBuf::from("/opt/homebrew/Cellar").exists() {
            PathBuf::from("/opt/homebrew/Cellar")
        } else {
            PathBuf::from("/usr/local/Cellar")
        };

        // Look for python@3.x packages
        if let Ok(entries) = std::fs::read_dir(&cellar_base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("python@") || name == "python" {
                    let path = entry.path();
                    let (size, file_count) = calculate_dir_size(&path)?;

                    if size > 10_000_000 {
                        items.push(CleanableItem {
                            name: format!("Homebrew {} (duplicate with pyenv)", name),
                            category: "Binary Analysis".to_string(),
                            subcategory: "Duplicate Source".to_string(),
                            icon: "🐍",
                            path: path.clone(),
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&path),
                            description: "Homebrew Python installed alongside pyenv. Consider using only one.",
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: Some(format!("brew uninstall {}", name)),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect old Homebrew Cellar versions (keeps only latest, flags older ones)
    fn detect_homebrew_old_versions(&self, items: &mut Vec<CleanableItem>) -> Result<()> {
        let cellar_base = if PathBuf::from("/opt/homebrew/Cellar").exists() {
            PathBuf::from("/opt/homebrew/Cellar")
        } else if PathBuf::from("/usr/local/Cellar").exists() {
            PathBuf::from("/usr/local/Cellar")
        } else {
            return Ok(());
        };

        // Packages to check for old versions
        let packages_to_check = [
            ("python@3.11", "🐍", "Python 3.11"),
            ("python@3.12", "🐍", "Python 3.12"),
            ("python@3.13", "🐍", "Python 3.13"),
            ("python", "🐍", "Python"),
            ("node", "📦", "Node.js"),
            ("ruby", "💎", "Ruby"),
            ("go", "🐹", "Go"),
            ("rust", "🦀", "Rust"),
            ("openjdk", "☕", "OpenJDK"),
            ("openjdk@17", "☕", "OpenJDK 17"),
            ("openjdk@21", "☕", "OpenJDK 21"),
            ("php", "🐘", "PHP"),
            ("php@8.2", "🐘", "PHP 8.2"),
            ("php@8.3", "🐘", "PHP 8.3"),
            ("perl", "🐪", "Perl"),
            ("lua", "🌙", "Lua"),
            ("erlang", "📡", "Erlang"),
            ("elixir", "💧", "Elixir"),
            ("dotnet", "🔷", ".NET"),
        ];

        for (package, icon, display_name) in packages_to_check {
            let package_dir = cellar_base.join(package);
            if !package_dir.exists() {
                continue;
            }

            // Get all versions
            let mut versions: Vec<_> = std::fs::read_dir(&package_dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .collect()
                })
                .unwrap_or_default();

            if versions.len() <= 1 {
                continue; // Only one version, nothing to clean
            }

            // Sort by modification time (newest last)
            versions.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).ok();
                let b_time = b.metadata().and_then(|m| m.modified()).ok();
                a_time.cmp(&b_time)
            });

            // Flag all but the latest version
            for old_version in versions.iter().take(versions.len() - 1) {
                let path = old_version.path();
                let version_name = old_version.file_name().to_string_lossy().to_string();
                let (size, file_count) = calculate_dir_size(&path)?;

                if size > 5_000_000 {
                    // 5MB minimum
                    items.push(CleanableItem {
                        name: format!("{} {} (old Homebrew version)", display_name, version_name),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Old Version".to_string(),
                        icon,
                        path: path.clone(),
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&path),
                        description: "Old Homebrew version. Run 'brew cleanup' to remove all old versions.",
                        safe_to_delete: SafetyLevel::Safe,
                        clean_command: Some("brew cleanup".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect uv cache and tools
    fn detect_uv_cache(&self, items: &mut Vec<CleanableItem>) -> Result<()> {
        // uv cache location
        let uv_cache = self.home.join(".cache/uv");
        if uv_cache.exists() {
            let (size, file_count) = calculate_dir_size(&uv_cache)?;
            if size > 100_000_000 {
                // 100MB minimum
                items.push(CleanableItem {
                    name: "uv cache".to_string(),
                    category: "Binary Analysis".to_string(),
                    subcategory: "Package Cache".to_string(),
                    icon: "🐍",
                    path: uv_cache,
                    size,
                    file_count: Some(file_count),
                    last_modified: None,
                    description: "uv package manager cache. Safe to clean, packages will be re-downloaded.",
                    safe_to_delete: SafetyLevel::Safe,
                    clean_command: Some("uv cache clean".to_string()),
                });
            }
        }

        // uv managed Python versions
        let uv_python = self.home.join(".local/share/uv/python");
        if uv_python.exists() {
            if let Ok(entries) = std::fs::read_dir(&uv_python) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let version = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size > 50_000_000 {
                        items.push(CleanableItem {
                            name: format!("Python {} (uv managed)", version),
                            category: "Binary Analysis".to_string(),
                            subcategory: "uv".to_string(),
                            icon: "🐍",
                            path: path.clone(),
                            size,
                            file_count: Some(file_count),
                            last_modified: get_mtime(&path),
                            description: "Python version managed by uv. Can be reinstalled with 'uv python install'.",
                            safe_to_delete: SafetyLevel::SafeWithCost,
                            clean_command: None,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if Homebrew Node is installed alongside a version manager
    fn check_homebrew_node_duplicate(&self, items: &mut Vec<CleanableItem>) -> Result<()> {
        let homebrew_node = PathBuf::from("/opt/homebrew/bin/node");
        let homebrew_node_intel = PathBuf::from("/usr/local/bin/node");

        let node_path = if homebrew_node.exists() {
            Some(homebrew_node)
        } else if homebrew_node_intel.exists()
            && self.is_homebrew_managed(&homebrew_node_intel)
        {
            Some(homebrew_node_intel)
        } else {
            None
        };

        if let Some(brew_node) = node_path {
            // Check if there's also a version manager node
            let has_fnm = self.home.join("Library/Application Support/fnm/node-versions").exists()
                || self.home.join(".local/share/fnm/node-versions").exists();
            let has_nvm = self.home.join(".nvm/versions/node").exists();
            let has_volta = self.home.join(".volta/tools/image/node").exists();

            if has_fnm || has_nvm || has_volta {
                // There's a duplicate
                let manager = if has_fnm {
                    "fnm"
                } else if has_nvm {
                    "nvm"
                } else {
                    "Volta"
                };

                // Get Homebrew node cellar path for size calculation
                let cellar_path = if brew_node.starts_with("/opt/homebrew") {
                    PathBuf::from("/opt/homebrew/Cellar/node")
                } else {
                    PathBuf::from("/usr/local/Cellar/node")
                };

                if cellar_path.exists() {
                    let (size, file_count) = calculate_dir_size(&cellar_path)?;

                    items.push(CleanableItem {
                        name: format!("Homebrew Node (duplicate with {})", manager),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Duplicate Source".to_string(),
                        icon: "📦",
                        path: cellar_path,
                        size,
                        file_count: Some(file_count),
                        last_modified: None,
                        description: "Homebrew Node.js installed alongside a version manager. Consider removing one.",
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: Some("brew uninstall node".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect stale configurations in shell rc files
    ///
    /// SAFETY: These items use a non-existent marker path to prevent accidental
    /// deletion of shell rc files. Users must manually edit their config files.
    fn detect_stale_configs(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        let rc_files = [
            self.home.join(".zshrc"),
            self.home.join(".bashrc"),
            self.home.join(".bash_profile"),
            self.home.join(".profile"),
        ];

        for rc_file in rc_files {
            if !rc_file.exists() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&rc_file) {
                // Check for NVM config without installation
                if (content.contains("NVM_DIR") || content.contains("nvm.sh"))
                    && !self.home.join(".nvm").exists()
                {
                    items.push(CleanableItem {
                        name: format!(
                            "[MANUAL] NVM stale config in {}",
                            rc_file.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Stale Config".to_string(),
                        icon: "⚠️",
                        // SAFETY: Use non-existent path to prevent accidental deletion of rc file
                        path: self.home.join(".devsweep-manual-edit-required"),
                        size: 0,
                        file_count: None,
                        last_modified: get_mtime(&rc_file),
                        description: "NVM in shell rc but not installed. MANUAL EDIT REQUIRED - DO NOT DELETE!",
                        safe_to_delete: SafetyLevel::Dangerous,
                        clean_command: Some(format!(
                            "Manual: Edit {} and remove NVM_DIR/nvm.sh lines",
                            rc_file.display()
                        )),
                    });
                }

                // Check for pyenv config without installation
                if (content.contains("PYENV_ROOT") || content.contains("pyenv init"))
                    && !self.home.join(".pyenv").exists()
                {
                    items.push(CleanableItem {
                        name: format!(
                            "[MANUAL] pyenv stale config in {}",
                            rc_file.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Stale Config".to_string(),
                        icon: "⚠️",
                        // SAFETY: Use non-existent path to prevent accidental deletion of rc file
                        path: self.home.join(".devsweep-manual-edit-required"),
                        size: 0,
                        file_count: None,
                        last_modified: get_mtime(&rc_file),
                        description: "pyenv in shell rc but not installed. MANUAL EDIT REQUIRED - DO NOT DELETE!",
                        safe_to_delete: SafetyLevel::Dangerous,
                        clean_command: Some(format!(
                            "Manual: Edit {} and remove PYENV_ROOT/pyenv init lines",
                            rc_file.display()
                        )),
                    });
                }

                // Check for rbenv config without installation
                if (content.contains("RBENV_ROOT") || content.contains("rbenv init"))
                    && !self.home.join(".rbenv").exists()
                {
                    items.push(CleanableItem {
                        name: format!(
                            "[MANUAL] rbenv stale config in {}",
                            rc_file.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        category: "Binary Analysis".to_string(),
                        subcategory: "Stale Config".to_string(),
                        icon: "⚠️",
                        // SAFETY: Use non-existent path to prevent accidental deletion of rc file
                        path: self.home.join(".devsweep-manual-edit-required"),
                        size: 0,
                        file_count: None,
                        last_modified: get_mtime(&rc_file),
                        description: "rbenv in shell rc but not installed. MANUAL EDIT REQUIRED - DO NOT DELETE!",
                        safe_to_delete: SafetyLevel::Dangerous,
                        clean_command: Some(format!(
                            "Manual: Edit {} and remove RBENV_ROOT/rbenv init lines",
                            rc_file.display()
                        )),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Convert analysis results to cleanable items for TUI display
    pub fn to_cleanable_items(&self, result: &BinaryAnalysisResult) -> Vec<CleanableItem> {
        let mut items = Vec::new();

        // Add duplicate groups
        for group in &result.duplicates {
            // Skip if only system binary or all are active
            if group.instances.iter().all(|i| i.is_active || i.source == BinarySource::System) {
                continue;
            }

            // Find non-active instances that could be removed
            let removable: Vec<&BinaryInstance> = group
                .instances
                .iter()
                .filter(|i| !i.is_active && i.source != BinarySource::System)
                .collect();

            if removable.is_empty() {
                continue;
            }

            for instance in removable {
                let description = match &group.recommendation {
                    DuplicateRecommendation::ConflictingManagers { managers } => {
                        let names: Vec<&str> = managers.iter().map(|m| m.name()).collect();
                        format!(
                            "Duplicate from {}. Also installed via: {}",
                            instance.source.name(),
                            names.join(", ")
                        )
                    }
                    DuplicateRecommendation::RemoveDuplicateSource { source } => {
                        format!("Duplicate installation from {}", source.name())
                    }
                    DuplicateRecommendation::RemoveOldVersions { versions } => {
                        format!(
                            "Old version. Newer versions available: {}",
                            versions.join(", ")
                        )
                    }
                    _ => "Duplicate binary".to_string(),
                };

                // Get directory size for version manager installs
                let (size, path_to_clean) =
                    if let Some(version_dir) = self.get_version_install_dir(&instance.path) {
                        let (dir_size, _) = calculate_dir_size(&version_dir).unwrap_or((0, 0));
                        (dir_size, version_dir)
                    } else {
                        (instance.binary_size, instance.resolved_path.clone())
                    };

                let clean_command = self.get_clean_command(instance);

                // Leak the description string to get a static reference
                // This is safe because we're in a controlled context
                let description_static: &'static str = Box::leak(description.into_boxed_str());

                items.push(CleanableItem {
                    name: format!(
                        "{} {} ({})",
                        group.command,
                        instance.version.as_deref().unwrap_or(""),
                        instance.source.name()
                    ),
                    category: "Binary Analysis".to_string(),
                    subcategory: "Duplicates".to_string(),
                    icon: self.get_icon_for_command(&group.command),
                    path: path_to_clean,
                    size,
                    file_count: None,
                    last_modified: get_mtime(&instance.path),
                    description: description_static,
                    safe_to_delete: group.safety,
                    clean_command,
                });
            }
        }

        // Add unused managers
        items.extend(result.unused_managers.clone());

        // Add stale configs
        items.extend(result.stale_configs.clone());

        // Sort by size
        items.sort_by(|a, b| b.size.cmp(&a.size));

        items
    }

    /// Get the installation directory for a version manager install
    fn get_version_install_dir(&self, binary_path: &Path) -> Option<PathBuf> {
        let path_str = binary_path.to_string_lossy();

        // fnm: ~/.local/share/fnm/node-versions/v22.22.0/
        if path_str.contains("fnm/node-versions/") {
            let parts: Vec<&str> = path_str.split("fnm/node-versions/").collect();
            if parts.len() > 1 {
                let version = parts[1].split('/').next()?;
                #[cfg(target_os = "macos")]
                let base = self.home.join("Library/Application Support/fnm/node-versions");
                #[cfg(not(target_os = "macos"))]
                let base = self.home.join(".local/share/fnm/node-versions");
                return Some(base.join(version));
            }
        }

        // nvm: ~/.nvm/versions/node/v22.22.0/
        if path_str.contains(".nvm/versions/node/") {
            let parts: Vec<&str> = path_str.split(".nvm/versions/node/").collect();
            if parts.len() > 1 {
                let version = parts[1].split('/').next()?;
                return Some(self.home.join(".nvm/versions/node").join(version));
            }
        }

        // pyenv: ~/.pyenv/versions/3.12.0/
        if path_str.contains(".pyenv/versions/") {
            let parts: Vec<&str> = path_str.split(".pyenv/versions/").collect();
            if parts.len() > 1 {
                let version = parts[1].split('/').next()?;
                return Some(self.home.join(".pyenv/versions").join(version));
            }
        }

        // Homebrew Cellar: /opt/homebrew/Cellar/node/22.0.0/
        if path_str.contains("/Cellar/") {
            let parts: Vec<&str> = path_str.split("/Cellar/").collect();
            if parts.len() > 1 {
                let rest = parts[1];
                let components: Vec<&str> = rest.split('/').collect();
                if components.len() >= 2 {
                    let package = components[0];
                    let version = components[1];
                    let cellar_base = if path_str.contains("/opt/homebrew/") {
                        PathBuf::from("/opt/homebrew/Cellar")
                    } else {
                        PathBuf::from("/usr/local/Cellar")
                    };
                    return Some(cellar_base.join(package).join(version));
                }
            }
        }

        None
    }

    /// Get clean command for a binary instance
    fn get_clean_command(&self, instance: &BinaryInstance) -> Option<String> {
        match instance.source {
            BinarySource::Homebrew => Some(format!(
                "brew uninstall {}",
                instance.command
            )),
            BinarySource::Nvm => instance
                .version
                .as_ref()
                .map(|v| format!("nvm uninstall {}", v)),
            BinarySource::Fnm => instance
                .version
                .as_ref()
                .map(|v| format!("fnm uninstall {}", v)),
            BinarySource::Pyenv => instance
                .version
                .as_ref()
                .map(|v| format!("pyenv uninstall -f {}", v)),
            BinarySource::Rbenv => instance
                .version
                .as_ref()
                .map(|v| format!("rbenv uninstall -f {}", v)),
            BinarySource::Cargo => Some(format!("cargo uninstall {}", instance.command)),
            BinarySource::Pipx => Some(format!("pipx uninstall {}", instance.command)),
            BinarySource::Npm => Some(format!("npm uninstall -g {}", instance.command)),
            _ => None,
        }
    }

    /// Get icon for a command
    fn get_icon_for_command(&self, command: &str) -> &'static str {
        if command.starts_with("python") || command.starts_with("pip") {
            "🐍"
        } else if command.starts_with("node")
            || command.starts_with("npm")
            || command.starts_with("npx")
        {
            "📦"
        } else if command.starts_with("ruby") || command.starts_with("gem") {
            "💎"
        } else if command.starts_with("go") {
            "🐹"
        } else if command.starts_with("rust") || command.starts_with("cargo") {
            "🦀"
        } else if command.starts_with("java") {
            "☕"
        } else {
            "🔧"
        }
    }
}

impl Default for BinaryAnalyzer {
    fn default() -> Self {
        Self::new().expect("BinaryAnalyzer requires home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_analyzer_creation() {
        let analyzer = BinaryAnalyzer::new();
        assert!(analyzer.is_some());
    }

    #[test]
    fn test_source_detection() {
        let analyzer = BinaryAnalyzer::new().unwrap();

        // System paths
        assert_eq!(
            analyzer.determine_source(Path::new("/usr/bin/python3")),
            BinarySource::System
        );
        assert_eq!(
            analyzer.determine_source(Path::new("/bin/bash")),
            BinarySource::System
        );

        // Homebrew paths
        assert_eq!(
            analyzer.determine_source(Path::new("/opt/homebrew/bin/node")),
            BinarySource::Homebrew
        );
        assert_eq!(
            analyzer.determine_source(Path::new("/opt/homebrew/Cellar/python@3.12/3.12.0/bin/python3")),
            BinarySource::Homebrew
        );

        // Cargo
        let home = dirs::home_dir().unwrap();
        let cargo_path = home.join(".cargo/bin/rg");
        assert_eq!(
            analyzer.determine_source(&cargo_path),
            BinarySource::Cargo
        );
    }

    #[test]
    fn test_symlink_resolution() {
        let analyzer = BinaryAnalyzer::new().unwrap();

        // Test with a known symlink
        let python_path = Path::new("/usr/bin/python3");
        if python_path.exists() {
            let resolved = analyzer.resolve_symlink_chain(python_path);
            assert!(resolved.exists());
        }
    }

    #[test]
    fn test_binary_analysis() {
        if let Some(analyzer) = BinaryAnalyzer::new() {
            let result = analyzer.analyze().unwrap();

            println!("Found {} binaries", result.binaries.len());
            println!("Found {} duplicate groups", result.duplicates.len());
            println!("Found {} unused managers", result.unused_managers.len());
            println!("Found {} stale configs", result.stale_configs.len());
            println!(
                "Potential savings: {} bytes",
                result.potential_savings
            );

            for dup in &result.duplicates {
                println!(
                    "  Duplicate: {} ({} instances)",
                    dup.command,
                    dup.instances.len()
                );
                for inst in &dup.instances {
                    println!(
                        "    - {} ({}) {}",
                        inst.path.display(),
                        inst.source.name(),
                        if inst.is_active { "[ACTIVE]" } else { "" }
                    );
                }
            }
        }
    }
}
