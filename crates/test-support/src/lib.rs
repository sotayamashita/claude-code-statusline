#![allow(clippy::new_without_default)]
// dev-only helpers; allow `new()` without `Default` for builder-like types
// and test utilities in this crate.
// This keeps test code concise without adding unnecessary impls.

//! Shared test utilities for the workspace
//!
//! This crate exposes common builders, fixtures and CLI helpers
//! for integration tests across workspace crates.

pub mod common;
