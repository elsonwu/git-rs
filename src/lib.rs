//! Git-RS: Educational Git Implementation
//! 
//! This library provides a minimal Git implementation in Rust designed for learning
//! Git internals and understanding how version control systems work.

pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod cli;

pub use domain::*;
pub use application::*;

/// Main error type for git-rs operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
