//! Git integration for safety checks
//!
//! Provides functionality to detect uncommitted changes and protect
//! users from accidentally deleting unsaved work.

mod status;
mod protection;

pub use status::*;
pub use protection::*;
