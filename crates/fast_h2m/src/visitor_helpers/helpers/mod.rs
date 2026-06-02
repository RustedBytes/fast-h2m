//! Helper functions for visitor pattern integration.
//!
//! This module provides efficient utilities for building visitor contexts,
//! dispatching visitor callbacks, and handling visitor results during the
//! HTML→Markdown conversion process.
//!
//! # Design Goals
//!
//! - **Zero allocation when possible**: Reuse existing data structures
//! - **Minimal overhead**: Inline hot paths, avoid unnecessary clones
//! - **Type safety**: Leverage Rust's type system for correct visitor handling
//! - **Ergonomics**: Reduce boilerplate for common visitor patterns
//!
//! # Usage
//!
//! These helpers are designed to be used within the converter module during
//! the DOM traversal. They bridge the gap between the internal conversion
//! state and the public visitor API.

pub mod callbacks;
pub mod content;
pub mod state;
pub mod traversal;

pub use content::VisitorDispatch;
pub use state::build_node_context;
pub use traversal::dispatch_visitor;
