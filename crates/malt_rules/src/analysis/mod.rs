//! Shared analysis infrastructure for lint rules.
//!
//! Provides reusable analysis passes that multiple rules can share:
//! - [`symbols`]: Per-scope variable definition and usage tracking.
//! - [`metadata`]: Function/class metadata extraction from parse trees.
//! - [`typing`]: Conservative one-pass type inference for variables.

pub mod control_flow;
pub mod metadata;
pub mod symbols;
pub mod typing;
