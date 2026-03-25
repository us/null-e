//! Git integration for safety checks
//!
//! Provides functionality to detect uncommitted changes and protect
//! users from accidentally deleting unsaved work.

mod protection;
mod status;

pub use protection::*;
pub use status::*;
