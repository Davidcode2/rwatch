//! Dummy data generation module
//!
//! Provides realistic test data for all API endpoints.
//! Enabled via --dummy flag or DUMMY_MODE environment variable.

pub mod data;
pub mod handlers;
pub mod state;

pub use handlers::*;
pub use state::DummyState;
