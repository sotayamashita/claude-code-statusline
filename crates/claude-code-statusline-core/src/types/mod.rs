//! Type definitions and data structures
//!
//! This module contains all the core type definitions used throughout
//! the claude-code-statusline application. Types are organized into submodules by
//! their domain.
//!
//! # Submodules
//!
//! * [`claude`] - Types for Claude Code JSON input
//! * [`config`] - Configuration structure definitions
//! * [`context`] - Runtime context that combines input and config

/// Types for parsing Claude Code JSON input
pub mod claude;

/// Configuration type definitions for TOML parsing
pub mod config;

/// Runtime context that combines input and configuration
pub mod context;
