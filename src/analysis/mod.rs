//! Analysis modules for deeper codebase insights
//!
//! This module contains analysis tools that go beyond simple cleanup:
//! - Git repository health and optimization
//! - Stale project detection
//! - Duplicate dependency detection

pub mod git;
pub mod stale;
pub mod duplicates;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A recommendation from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Type of recommendation
    pub kind: RecommendationKind,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected path
    pub path: PathBuf,
    /// Potential space savings in bytes
    pub potential_savings: u64,
    /// Command to fix (if applicable)
    pub fix_command: Option<String>,
    /// Risk level
    pub risk: RiskLevel,
}

/// Types of recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationKind {
    /// Git repository optimization
    GitOptimization,
    /// Git LFS cache cleanup
    GitLfsCache,
    /// Stale project that hasn't been touched
    StaleProject,
    /// Duplicate dependencies
    DuplicateDependency,
}

/// Risk level for a recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No risk, purely optimization
    None,
    /// Low risk, easily reversible
    Low,
    /// Medium risk, may require rebuild
    Medium,
    /// High risk, may lose data
    High,
}

impl RiskLevel {
    /// Get color hint for display
    pub fn color_hint(&self) -> &'static str {
        match self {
            Self::None => "green",
            Self::Low => "blue",
            Self::Medium => "yellow",
            Self::High => "red",
        }
    }

    /// Get symbol for display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::None => "✓",
            Self::Low => "○",
            Self::Medium => "~",
            Self::High => "!",
        }
    }
}

impl Recommendation {
    /// Format potential savings for display
    pub fn savings_display(&self) -> String {
        format_size(self.potential_savings)
    }
}

/// Format bytes as human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
