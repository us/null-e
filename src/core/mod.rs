//! Core types and traits for null-e
//!
//! This module contains the fundamental abstractions used throughout the application:
//! - Project detection and representation
//! - Artifact types and metadata
//! - Scanner and cleaner traits

mod artifact;
mod cleaner;
mod project;
mod scanner;

pub use artifact::*;
pub use cleaner::*;
pub use project::*;
pub use scanner::*;
