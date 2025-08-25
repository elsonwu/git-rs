//! Git-RS: Educational Git Implementation
//!
//! This library provides a minimal Git implementation in Rust designed for learning
//! Git internals and understanding how version control systems work.

pub mod application;
pub mod cli;
pub mod domain;
pub mod infrastructure;

pub use application::*;
pub use domain::*;

/// Main error type for git-rs operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
