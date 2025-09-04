//! Engine - Public API facade for rendering status lines
//!
//! This module introduces a thin `Engine` wrapper that will later move
//! to `beacon-core` per the refactoring plan. For now, it delegates to
//! existing internals to keep behavior identical while exposing a stable
//! entrypoint for library users and tests.

use crate::Config;
use crate::debug::DebugLogger;
use crate::modules::render_module_with_timeout;
use crate::parser::{extract_modules_from_format, parse_format};
use crate::types::claude::ClaudeInput;
use crate::types::context::Context;
use anyhow::Result;
use std::collections::HashMap;

/// Rendering engine that produces a status line from input and config.
pub struct Engine {
    config: Config,
}

impl Engine {
    /// Construct a new engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Render a status line string from the provided Claude input.
    pub fn render(&self, input: &ClaudeInput) -> Result<String> {
        let logger = DebugLogger::new(self.config.debug);
        let context = Context::new(input.clone(), self.config.clone());

        let format = &context.config.format;
        let module_names = extract_modules_from_format(format);
        let mut module_outputs: HashMap<String, String> = HashMap::new();

        for name in &module_names {
            if name == "character" {
                continue;
            }
            if let Some(out) = render_module_with_timeout(name, &context, &logger) {
                module_outputs.insert(name.clone(), out);
            }
        }

        Ok(parse_format(format, &context, &module_outputs))
    }
}
