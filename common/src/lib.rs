//! # Rwatch Common Library
//!
//! This crate contains shared types and protocols used by both the agent and TUI.
//! Keeping these in a shared library ensures type safety and DRY principles.

pub mod health;
pub mod memory;
pub mod memory_display;
