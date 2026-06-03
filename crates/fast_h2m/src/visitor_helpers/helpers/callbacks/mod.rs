//! Callback management for visitor pattern.
//!
//! The callback implementation is split between `state`, `content`, and
//! `traversal`. This module preserves the callback namespace for callers that
//! want to import visitor callback primitives from one place.

pub use super::content::VisitorDispatch;
pub use super::state::build_node_context;
pub use super::traversal::dispatch_visitor;
