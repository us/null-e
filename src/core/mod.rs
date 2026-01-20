//! Core types and traits for DevSweep
//!
//! This module contains the fundamental abstractions used throughout the application:
//! - Project detection and representation
//! - Artifact types and metadata
//! - Scanner and cleaner traits

mod project;
mod artifact;
mod scanner;
mod cleaner;

pub use project::*;
pub use artifact::*;
pub use scanner::*;
pub use cleaner::*;
