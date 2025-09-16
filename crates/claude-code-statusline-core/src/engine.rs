//! Engine - Public API facade for rendering status lines
//!
//! This module introduces a thin `Engine` wrapper that will later move
//! to `claude-code-statusline-core` per the refactoring plan. For now, it delegates to
//! existing internals to keep behavior identical while exposing a stable
//! entrypoint for library users and tests.

use crate::Config;
use crate::debug::DebugLogger;
use crate::error::CoreError;
use crate::modules::render_module_with_timeout;
use crate::parser::extract_modules_from_format;
use crate::types::claude::ClaudeInput;
use crate::types::context::Context;
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
    pub fn render(&self, input: &ClaudeInput) -> Result<String, CoreError> {
        let logger = DebugLogger::new(self.config.debug);
        let context = Context::new(input.clone(), self.config.clone());

        let format = &context.config.format;
        let module_names = extract_modules_from_format(format);

        // Render modules (optionally in parallel when feature enabled)
        fn normalize_module_output(mut value: String) -> String {
            if value.ends_with("\x1b[0m") {
                value.truncate(value.len().saturating_sub(4));
            }
            value
        }

        let module_outputs: HashMap<String, String> = {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                module_names
                    .par_iter()
                    .filter_map(|name| {
                        if name == "character" {
                            return None;
                        }
                        render_module_with_timeout(name, &context, &logger)
                            .map(|out| (name.clone(), normalize_module_output(out)))
                    })
                    .collect()
            }

            #[cfg(not(feature = "parallel"))]
            {
                let mut map = HashMap::new();
                for name in &module_names {
                    if name == "character" {
                        continue;
                    }
                    if let Some(out) = render_module_with_timeout(name, &context, &logger) {
                        map.insert(name.clone(), normalize_module_output(out));
                    }
                }
                map
            }
        };

        // Replace tokens anywhere and apply top-level bracket styles like
        // [text](fg:.. bg:..), matching Starship-style presets.
        let mut tokens: HashMap<&str, String> = HashMap::new();
        for (k, v) in &module_outputs {
            tokens.insert(k.as_str(), v.clone());
        }
        let rendered = crate::style::render_with_style_template(format, &tokens, "");
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::config::{ClaudeModelConfig, DirectoryConfig};

    #[test]
    fn engine_renders_default_format() {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".into(),
            transcript_path: None,
            cwd: "/tmp".into(),
            model: ModelInfo {
                id: "claude-opus".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/tmp".into(),
                project_dir: Some("/tmp".into()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let cfg = Config::default();
        let engine = Engine::new(cfg);
        let out = engine.render(&input).expect("render ok");
        let plain = String::from_utf8(strip_ansi_escapes::strip(out)).unwrap();
        assert!(plain.contains("/tmp"));
        assert!(plain.contains("Opus"));
    }

    #[test]
    fn engine_preserves_background_across_segments() {
        let cfg = Config {
            format: "$directory[](fg:prev_bg bg:#222324)$claude_model".into(),
            directory: DirectoryConfig {
                format: "[DIR]($style)".into(),
                style: "fg:#FFFFFF bg:#111213".into(),
                ..DirectoryConfig::default()
            },
            claude_model: ClaudeModelConfig {
                format: "[CL]($style)".into(),
                style: "fg:prev_bg bg:#333435".into(),
                ..ClaudeModelConfig::default()
            },
            ..Config::default()
        };

        let engine = Engine::new(cfg);
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".into(),
            transcript_path: None,
            cwd: "/tmp".into(),
            model: ModelInfo {
                id: "claude-opus".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/tmp".into(),
                project_dir: Some("/tmp".into()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };

        let rendered = engine.render(&input).expect("render ok");
        let without_final_reset = rendered.strip_suffix("\u{1b}[0m").expect("final reset");
        assert!(without_final_reset.contains("DIR"));
        assert!(without_final_reset.contains("CL"));
        // Directory background applied once
        assert!(without_final_reset.contains("48;2;17;18;19"));
        // Powerline separator uses previous background as foreground
        assert!(without_final_reset.contains("38;2;17;18;19"));
        // Ensure that no intermediate reset is injected
        assert_eq!(without_final_reset.matches("\u{1b}[0m").count(), 0);
    }
}
