//! Error handling for DevSweep
//!
//! Provides a comprehensive error type hierarchy with context support
//! and user-friendly error messages.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for DevSweep operations
#[derive(Error, Debug)]
pub enum DevSweepError {
    // ═══════════════════════════════════════════════════════════════
    // I/O Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path not found: {}", .0.display())]
    PathNotFound(PathBuf),

    #[error("Permission denied: {}", .0.display())]
    PermissionDenied(PathBuf),

    #[error("Path is not a directory: {}", .0.display())]
    NotADirectory(PathBuf),

    // ═══════════════════════════════════════════════════════════════
    // Scanner Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("Scan interrupted by user")]
    ScanInterrupted,

    #[error("Scan timeout after {0} seconds")]
    ScanTimeout(u64),

    // ═══════════════════════════════════════════════════════════════
    // Plugin Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Plugin '{plugin}' error: {message}")]
    Plugin { plugin: String, message: String },

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Plugin '{0}' failed to initialize")]
    PluginInitFailed(String),

    // ═══════════════════════════════════════════════════════════════
    // Git Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Git error: {0}")]
    Git(String),

    #[error("Uncommitted changes detected in: {}", .0.display())]
    UncommittedChanges(PathBuf),

    #[error("Not a git repository: {}", .0.display())]
    NotAGitRepo(PathBuf),

    // ═══════════════════════════════════════════════════════════════
    // Docker Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Docker daemon not available. Is Docker running?")]
    DockerNotAvailable,

    #[error("Docker operation timed out")]
    DockerTimeout,

    // ═══════════════════════════════════════════════════════════════
    // Trash Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Trash error: {0}")]
    Trash(String),

    #[error("Cannot restore '{0}': original location exists")]
    RestoreConflict(String),

    #[error("Cannot restore '{0}': {1}")]
    RestoreFailed(String, String),

    // ═══════════════════════════════════════════════════════════════
    // Clean Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Clean operation blocked: {0}")]
    CleanBlocked(String),

    #[error("Failed to clean {}: {}", .path.display(), .reason)]
    CleanFailed { path: PathBuf, reason: String },

    #[error("Partial clean failure: {succeeded} succeeded, {failed} failed")]
    PartialCleanFailure { succeeded: usize, failed: usize },

    // ═══════════════════════════════════════════════════════════════
    // Config Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid config file at {}: {}", .path.display(), .reason)]
    ConfigParse { path: PathBuf, reason: String },

    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(String),

    // ═══════════════════════════════════════════════════════════════
    // TUI Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Terminal not supported")]
    TerminalNotSupported,

    // ═══════════════════════════════════════════════════════════════
    // Serialization Errors
    // ═══════════════════════════════════════════════════════════════
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    // ═══════════════════════════════════════════════════════════════
    // Generic
    // ═══════════════════════════════════════════════════════════════
    #[error("{0}")]
    Other(String),

    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<DevSweepError>,
    },
}

impl DevSweepError {
    /// Create a plugin error with plugin name and message
    pub fn plugin(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Plugin {
            plugin: plugin.into(),
            message: message.into(),
        }
    }

    /// Create an error with additional context
    pub fn with_context(self, context: impl Into<String>) -> Self {
        Self::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Check if this error is recoverable (operation can continue)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::PathNotFound(_)
                | Self::PermissionDenied(_)
                | Self::ScanInterrupted
                | Self::DockerNotAvailable
                | Self::NotAGitRepo(_)
                | Self::Plugin { .. }
        )
    }

    /// Check if this error is a user-caused interruption
    pub fn is_user_interrupt(&self) -> bool {
        matches!(self, Self::ScanInterrupted)
    }

    /// Get a suggested action for the user
    pub fn suggested_action(&self) -> Option<&'static str> {
        match self {
            Self::PermissionDenied(_) => Some("Try running with elevated permissions (sudo)"),
            Self::UncommittedChanges(_) => Some("Commit or stash your changes first, or use --force"),
            Self::DockerNotAvailable => Some("Start Docker Desktop or the Docker daemon"),
            Self::RestoreFailed(_, _) => Some("Check the trash directory or restore manually"),
            Self::NotAGitRepo(_) => Some("Initialize a git repository or use --no-git-check"),
            Self::ConfigParse { .. } => Some("Check your config file syntax"),
            Self::InvalidPattern(_) => Some("Check glob pattern syntax"),
            _ => None,
        }
    }

    /// Get the error code for CLI exit status
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ScanInterrupted => 130, // Standard SIGINT exit code
            Self::PermissionDenied(_) => 126,
            Self::PathNotFound(_) | Self::NotADirectory(_) => 127,
            Self::Config(_) | Self::ConfigParse { .. } => 78, // EX_CONFIG
            Self::UncommittedChanges(_) | Self::CleanBlocked(_) => 1,
            _ => 1,
        }
    }
}

/// Result type alias for DevSweep operations
pub type Result<T> = std::result::Result<T, DevSweepError>;

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> Result<T>;

    /// Add path context to an error
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T, E: Into<DevSweepError>> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.into().with_context(context))
    }

    fn with_path(self, path: impl Into<PathBuf>) -> Result<T> {
        let path = path.into();
        self.map_err(|e| {
            let err = e.into();
            match &err {
                DevSweepError::Io(io_err) => match io_err.kind() {
                    std::io::ErrorKind::NotFound => DevSweepError::PathNotFound(path),
                    std::io::ErrorKind::PermissionDenied => DevSweepError::PermissionDenied(path),
                    _ => err,
                },
                _ => err,
            }
        })
    }
}

/// Extension trait for Option types
pub trait OptionExt<T> {
    /// Convert None to an error with message
    fn ok_or_err(self, msg: impl Into<String>) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_err(self, msg: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| DevSweepError::Other(msg.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_recoverable() {
        assert!(DevSweepError::PathNotFound(PathBuf::from("/test")).is_recoverable());
        assert!(DevSweepError::ScanInterrupted.is_recoverable());
        assert!(!DevSweepError::Other("fatal".into()).is_recoverable());
    }

    #[test]
    fn test_error_suggested_action() {
        let err = DevSweepError::UncommittedChanges(PathBuf::from("/test"));
        assert!(err.suggested_action().is_some());

        let err = DevSweepError::Other("generic".into());
        assert!(err.suggested_action().is_none());
    }

    #[test]
    fn test_error_with_context() {
        let err = DevSweepError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let with_ctx = err.with_context("reading config");

        assert!(matches!(with_ctx, DevSweepError::WithContext { .. }));
    }
}
