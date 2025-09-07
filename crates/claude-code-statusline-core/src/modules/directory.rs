//! Directory module for displaying the current working directory
//!
//! This module shows the current directory path with home directory
//! abbreviation (~) and optional truncation for long paths.

use super::{Module, ModuleConfig};
use crate::types::context::Context;
use std::path::Path;

/// Module that displays the current working directory
///
/// Features:
/// - Home directory abbreviation (e.g., `/home/user` → `~`)
/// - Path truncation for long directories
/// - Repository-relative paths (when in git repos)
/// - ANSI color styling support
///
/// # Configuration
///
/// ```toml
/// [directory]
/// format = "[$path]($style)"
/// style = "bold cyan"
/// truncation_length = 3
/// truncate_to_repo = true
/// ```
pub struct DirectoryModule;

impl DirectoryModule {
    /// Create a new DirectoryModule instance
    pub fn new() -> Self {
        Self
    }

    /// Create from Context (kept for compatibility)
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }

    /// Resolve user's home directory, preferring HOME env var when present
    fn resolve_home_dir(&self) -> Option<std::path::PathBuf> {
        match std::env::var("HOME") {
            Ok(home) if !home.is_empty() => Some(std::path::PathBuf::from(home)),
            _ => dirs::home_dir(),
        }
    }

    /// Abbreviate home directory to ~ (cross-platform)
    fn abbreviate_home(&self, path: &Path) -> String {
        if let Some(home) = self.resolve_home_dir() {
            if let Ok(relative) = path.strip_prefix(&home) {
                if relative.as_os_str().is_empty() {
                    return "~".to_string();
                }
                return format!("~/{}", relative.display());
            }
        }
        path.display().to_string()
    }
}

impl Default for DirectoryModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DirectoryModule {
    fn name(&self) -> &str {
        "directory"
    }

    fn should_display(&self, _context: &Context, config: &dyn ModuleConfig) -> bool {
        // Check if the module is disabled in config
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::DirectoryConfig>()
        {
            return !cfg.disabled;
        }
        true // Default to displaying if no config found
    }

    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String {
        // Try to use module-specific formatting if available
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::DirectoryConfig>()
        {
            // If truncate_to_repo is enabled and we're inside a repo, construct
            // a repository-relative path: `<repo-name>/<sub/dirs>`, truncated to
            // at most `truncation_length` segments, always keeping the repo name.
            let mut repo_root: Option<std::path::PathBuf> = None;

            if cfg.truncate_to_repo {
                #[cfg(feature = "git")]
                {
                    if let Ok(repo) = context.repo() {
                        if let Some(wd) = repo.workdir() {
                            if context.current_dir.starts_with(wd) {
                                repo_root = Some(wd.to_path_buf());
                            }
                        }
                    }
                }
                // Fallback discovery without git feature: look for a `.git` directory or file (worktrees)
                if repo_root.is_none() {
                    let mut p = context.current_dir.as_path();
                    loop {
                        let dot_git = p.join(".git");
                        // In worktrees, `.git` can be a file; treat either as a repository marker
                        if dot_git.is_dir() || dot_git.is_file() {
                            repo_root = Some(p.to_path_buf());
                            break;
                        }
                        match p.parent() {
                            Some(parent) => p = parent,
                            None => break,
                        }
                    }
                }
            }

            let path_str = if let Some(root) = repo_root {
                // repo name
                let repo_name = root
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.display().to_string());

                // relative components from repo root to current dir
                let mut segments: Vec<String> = vec![repo_name];
                if let Ok(rel) = context.current_dir.strip_prefix(&root) {
                    use std::path::Component;
                    for c in rel.components() {
                        if let Component::Normal(os) = c {
                            let s = os.to_string_lossy().to_string();
                            if !s.is_empty() {
                                segments.push(s);
                            }
                        }
                    }
                }

                // Truncate to at most `truncation_length` segments, preserving repo name
                let tl = std::cmp::max(1, cfg.truncation_length);
                if segments.len() > tl {
                    let keep_tail = tl.saturating_sub(1);
                    if keep_tail == 0 {
                        // Only show repo name when nothing else is kept
                        segments[0].clone()
                    } else {
                        let start = segments.len() - keep_tail;
                        let tail = &segments[start..];
                        let mut out = String::with_capacity(segments[0].len() + 1 + 4 * keep_tail);
                        out.push_str(&segments[0]); // repo name
                        out.push('/');
                        if !cfg.truncation_symbol.is_empty() {
                            out.push_str(&cfg.truncation_symbol);
                        }
                        out.push_str(&tail.join("/"));
                        out
                    }
                } else {
                    segments.join("/")
                }
            } else {
                // Fallback to home abbreviation (legacy behavior)
                self.abbreviate_home(&context.current_dir)
            };

            use std::collections::HashMap;
            let mut tokens: HashMap<&str, String> = HashMap::new();
            tokens.insert("path", path_str.clone());
            return crate::style::render_with_style_template(cfg.format(), &tokens, cfg.style());
        }

        // No config found: return plain abbreviated path
        self.abbreviate_home(&context.current_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::context::Context;
    use rstest::*;
    use std::fs::create_dir_all;
    use std::sync::{Mutex, OnceLock};

    /// Fixture for creating test contexts
    #[fixture]
    fn test_context() -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/Users/test/projects".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/Users/test/projects".to_string(),
                project_dir: Some("/Users/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    /// Helper to create context with specific cwd
    fn context_with_cwd(cwd: &str) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: cwd.to_string(),
                project_dir: Some("/Users/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    #[rstest]
    fn test_directory_module(test_context: Context) {
        let module = DirectoryModule::new();
        assert_eq!(module.name(), "directory");
        assert!(module.should_display(&test_context, &test_context.config.directory));
    }

    #[rstest]
    #[case("/Users/test", "~")]
    #[case("/Users/test/projects", "~/projects")]
    #[case("/Users/test/Documents/code", "~/Documents/code")]
    fn test_home_directory_abbreviation(#[case] cwd: &str, #[case] expected: &str) {
        let module = DirectoryModule::new();
        // Serialize HOME mutation to avoid test flakiness in parallel runs
        static HOME_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = HOME_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        // Save and set HOME environment variable
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", "/Users/test");
        }

        let context = context_with_cwd(cwd);
        let rendered = module.render(&context, &context.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        assert_eq!(plain, expected);

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[rstest]
    #[case("/var/www/html", "/var/www/html")]
    #[case("/tmp/test", "/tmp/test")]
    #[case("/usr/local/bin", "/usr/local/bin")]
    fn test_non_home_paths(#[case] cwd: &str, #[case] expected: &str) {
        let module = DirectoryModule::new();
        let context = context_with_cwd(cwd);
        let rendered = module.render(&context, &context.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        assert_eq!(plain, expected);
    }

    #[cfg(feature = "git")]
    fn init_git_repo(root: &std::path::Path) -> git2::Repository {
        use git2::Repository;
        let repo = Repository::init(root).unwrap();
        // initial commit for a valid repo
        let sig = git2::Signature::now("Tester", "tester@example.com").unwrap();
        std::fs::write(root.join("README.md"), b"init\n").unwrap();
        let mut idx = repo.index().unwrap();
        idx.add_path(std::path::Path::new("README.md")).unwrap();
        let tree_id = idx.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let head = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
        drop(tree);
        let c0 = repo.find_commit(head).unwrap();
        let _ = repo.branch("main", &c0, true).ok();
        drop(c0);
        let _ = repo.set_head("refs/heads/main");
        repo
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn repo_root_displays_repo_name_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        create_dir_all(root).unwrap();
        let _repo = init_git_repo(root);

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: root.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: root.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 3;
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(plain, repo_name);
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn repo_subdir_includes_repo_and_tail_segments() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let _repo = init_git_repo(root);
        let sub = root.join("src").join("module");
        create_dir_all(&sub).unwrap();

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: sub.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: sub.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 3; // repo + 2
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(
            plain,
            format!("{repo}/{a}/{b}", repo = repo_name, a = "src", b = "module")
        );
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn truncation_length_preserves_repo_and_tails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let _repo = init_git_repo(root);
        let deep = root.join("a").join("b").join("c").join("d");
        create_dir_all(&deep).unwrap();

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: deep.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: deep.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 2; // repo + last 1
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(
            plain,
            format!("{repo}/{tail}", repo = repo_name, tail = "d")
        );
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn repo_truncation_inserts_symbol_between_repo_and_tail() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let _repo = init_git_repo(root);
        let deep = root.join("a").join("b").join("c").join("d");
        create_dir_all(&deep).unwrap();

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: deep.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: deep.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 2; // repo + last 1
        cfg.directory.truncation_symbol = "…/".to_string();
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(plain, format!("{}/…/{}", repo_name, "d"));
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn no_symbol_when_not_truncated_in_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let _repo = init_git_repo(root);
        let sub = root.join("src").join("module");
        create_dir_all(&sub).unwrap();

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: sub.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: sub.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 3; // repo + 2 -> exactly fits
        cfg.directory.truncation_symbol = "…/".to_string();
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(plain, format!("{}/{}/{}", repo_name, "src", "module"));
    }

    #[rstest]
    fn fallback_detects_git_file_worktree() {
        // Simulate a Git worktree-like layout where `.git` is a file, not a directory.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sub = root.join("src").join("module");
        std::fs::create_dir_all(&sub).unwrap();
        // Create a `.git` file at the root to emulate worktree behavior
        std::fs::write(root.join(".git"), b"gitdir: /path/to/real/gitdir\n").unwrap();

        let input = crate::types::claude::ClaudeInput {
            hook_event_name: None,
            session_id: "test".into(),
            transcript_path: None,
            cwd: sub.to_string_lossy().to_string(),
            model: crate::types::claude::ModelInfo {
                id: "id".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(crate::types::claude::WorkspaceInfo {
                current_dir: sub.to_string_lossy().to_string(),
                project_dir: Some(root.to_string_lossy().to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = crate::config::Config::default();
        cfg.directory.truncate_to_repo = true;
        cfg.directory.truncation_length = 3; // repo + 2
        let ctx = crate::types::context::Context::new(input, cfg);

        let module = DirectoryModule::new();
        let rendered = module.render(&ctx, &ctx.config.directory);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        let repo_name = root.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(
            plain,
            format!("{repo}/{a}/{b}", repo = repo_name, a = "src", b = "module")
        );
    }
}
