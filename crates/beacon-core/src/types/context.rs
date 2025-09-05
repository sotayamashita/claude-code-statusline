//! Runtime context module
//!
//! This module provides the Context structure that combines Claude Code input
//! with application configuration and provides memoized access to expensive
//! operations like git repository discovery and directory scanning.

use crate::config::Config;
use crate::types::claude::ClaudeInput;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "git")]
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
static REPO_DISCOVER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Central context structure that holds all runtime data and configuration
///
/// The Context combines Claude Code input with application configuration
/// and provides cached access to expensive operations. It uses OnceLock
/// for memoization to ensure operations like git repository discovery
/// only happen once per execution.
///
/// # Memoization
///
/// - Git repository discovery is cached using OnceLock
/// - Directory contents scanning is cached using OnceLock
/// - Both operations are thread-safe and only executed once
pub struct Context {
    /// Raw input from Claude Code
    pub input: ClaudeInput,

    /// Application configuration
    #[allow(dead_code)]
    pub config: Config,

    /// Current working directory (processed)
    pub current_dir: PathBuf,

    /// Project root directory (e.g., git repository root)
    /// Will be populated in Phase 2 when git support is added
    #[allow(dead_code)]
    pub project_root: Option<PathBuf>,

    /// Memoized git repository handle for current directory
    #[cfg(feature = "git")]
    repo: OnceLock<Result<Mutex<git2::Repository>, git2::Error>>,

    /// Memoized directory contents for current working directory
    #[allow(dead_code)]
    dir_contents: OnceLock<Result<DirContents, io::Error>>,
}

impl Context {
    /// Create a new Context from ClaudeInput and Config
    pub fn new(input: ClaudeInput, config: Config) -> Self {
        let current_dir = PathBuf::from(&input.cwd);

        // For now, project_root is the same as workspace.project_dir if available
        let project_root = input
            .workspace
            .as_ref()
            .and_then(|ws| ws.project_dir.as_ref())
            .map(PathBuf::from);

        Self {
            input,
            config,
            current_dir,
            project_root,
            #[cfg(feature = "git")]
            repo: OnceLock::new(),
            dir_contents: OnceLock::new(),
        }
    }

    /// Get the current directory as a string
    #[allow(dead_code)]
    pub fn current_dir_str(&self) -> &str {
        &self.input.cwd
    }

    /// Get the model display name
    pub fn model_display_name(&self) -> &str {
        &self.input.model.display_name
    }

    /// Get memoized git repository for current directory (if available).
    /// Uses OnceLock to avoid repeated `git2::Repository::discover` calls.
    #[cfg(feature = "git")]
    pub fn repo(&self) -> Result<MutexGuard<'_, git2::Repository>, &git2::Error> {
        let res = self.repo.get_or_init(|| {
            #[cfg(test)]
            REPO_DISCOVER_COUNT.fetch_add(1, Ordering::Relaxed);
            git2::Repository::discover(&self.current_dir).map(Mutex::new)
        });
        match res {
            Ok(repo) => Ok(repo.lock().unwrap()),
            Err(err) => Err(err),
        }
    }

    /// Get memoized directory contents for current directory.
    #[allow(dead_code)]
    pub fn dir_contents(&self) -> Result<&DirContents, &io::Error> {
        let res = self.dir_contents.get_or_init(|| {
            #[cfg(test)]
            DIR_SCAN_COUNT.fetch_add(1, Ordering::Relaxed);
            DirContents::scan(&self.current_dir)
        });
        match res {
            Ok(dc) => Ok(dc),
            Err(err) => Err(err),
        }
    }

    #[cfg(test)]
    pub fn test_repo_discover_count() -> usize {
        REPO_DISCOVER_COUNT.load(Ordering::Relaxed)
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use rstest::rstest;
    use std::fs::File;
    use std::io::Write as _;
    #[cfg(feature = "git")]
    use std::path::Path;
    use tempfile::tempdir;

    #[cfg(feature = "git")]
    use git2::Repository as GitRepository;

    /// Helper to create test ClaudeInput
    fn create_claude_input(cwd: &str, model: &str, workspace: Option<(&str, &str)>) -> ClaudeInput {
        ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: format!("claude-{}", model.to_lowercase()),
                display_name: model.to_string(),
            },
            workspace: workspace.map(|(current, project)| WorkspaceInfo {
                current_dir: current.to_string(),
                project_dir: Some(project.to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        }
    }

    #[rstest]
    #[case("/test/dir", "Opus")]
    #[case("/another/dir", "Sonnet")]
    #[case("/home/user", "Haiku")]
    fn test_context_creation(#[case] cwd: &str, #[case] model: &str) {
        let input = create_claude_input(cwd, model, Some((cwd, "/test/project")));
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), cwd);
        assert_eq!(context.model_display_name(), model);
    }

    #[rstest]
    #[case("/test/dir", "/test/project", Some(PathBuf::from("/test/project")))]
    #[case(
        "/another/dir",
        "/another/project",
        Some(PathBuf::from("/another/project"))
    )]
    fn test_context_with_workspace(
        #[case] cwd: &str,
        #[case] project: &str,
        #[case] expected_root: Option<PathBuf>,
    ) {
        let input = create_claude_input(cwd, "Opus", Some((cwd, project)));
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), cwd);
        assert_eq!(context.project_root, expected_root);
    }

    #[rstest]
    fn test_context_without_workspace() {
        let input = create_claude_input("/another/dir", "Sonnet", None);
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), "/another/dir");
        assert_eq!(context.model_display_name(), "Sonnet");
        assert_eq!(context.project_root, None);
    }

    #[cfg(feature = "git")]
    #[rstest]
    fn test_repo_memoization_once() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let repo = GitRepository::init(root).unwrap();
        // initial commit so that repository is valid
        let sig = git2::Signature::now("Tester", "tester@example.com").unwrap();
        let mut f = File::create(root.join("README.md")).unwrap();
        writeln!(f, "init").unwrap();
        f.sync_all().unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let _ = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        let root_str = root.to_str().unwrap();
        let input = create_claude_input(root_str, "Opus", Some((root_str, root_str)));
        // The helper above expects (current, project); pass same path for both.
        let context = Context::new(input, Config::default());

        // First call: discover runs (global counter may be affected by other tests)
        drop(context.repo().unwrap());
        let after_first = Context::test_repo_discover_count();
        // Second call: should be memoized (no increment for the same Context)
        drop(context.repo().unwrap());
        assert_eq!(Context::test_repo_discover_count(), after_first);
    }

    #[rstest]
    fn test_dir_contents_memoization_once() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        // create files and a directory
        fs::create_dir(root.join("subdir")).unwrap();
        let mut f1 = File::create(root.join("a.txt")).unwrap();
        writeln!(f1, "hello").unwrap();
        f1.sync_all().unwrap();
        let mut f2 = File::create(root.join("b.rs")).unwrap();
        writeln!(f2, "fn main(){{}} ").unwrap();
        f2.sync_all().unwrap();

        let root_str = root.to_str().unwrap();
        let input = create_claude_input(root_str, "Opus", Some((root_str, root_str)));
        let context = Context::new(input, Config::default());

        // Baseline counter
        let baseline = Context::test_dir_scan_count();
        // First call triggers scan
        let dc1 = context.dir_contents().unwrap();
        assert!(dc1.contains_file("a.txt"));
        assert!(dc1.contains_file("b.rs"));
        assert!(dc1.folders.contains("subdir"));
        assert_eq!(Context::test_dir_scan_count(), baseline + 1);

        // Second call uses memoized result (no increment)
        let dc2 = context.dir_contents().unwrap();
        assert!(std::ptr::eq(dc1 as *const _, dc2 as *const _));
        assert_eq!(Context::test_dir_scan_count(), baseline + 1);
    }
}

// tests moved to bottom of file

impl Clone for Context {
    fn clone(&self) -> Self {
        // Reconstruct a fresh Context from cloned input and config.
        // Memoized fields are intentionally not copied (per-execution cache only).
        Self::new(self.input.clone(), self.config.clone())
    }
}

// placeholder removed

/// Directory contents summary for quick lookups
///
/// Provides a snapshot of directory contents including files,
/// folders, and file extensions for efficient queries.
#[allow(dead_code)]
pub struct DirContents {
    #[allow(dead_code)]
    pub files: HashSet<String>,
    #[allow(dead_code)]
    pub folders: HashSet<String>,
    #[allow(dead_code)]
    pub extensions: HashSet<String>,
}

#[allow(dead_code)]
impl DirContents {
    /// Scan non-recursively the given directory.
    pub fn scan(dir: &Path) -> io::Result<Self> {
        let mut files = HashSet::new();
        let mut folders = HashSet::new();
        let mut extensions = HashSet::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy().to_string();
            if file_type.is_dir() {
                folders.insert(name);
            } else if file_type.is_file() {
                if let Some(ext) = Path::new(&name).extension().and_then(|e| e.to_str()) {
                    extensions.insert(ext.to_string());
                }
                files.insert(name);
            }
        }
        Ok(Self {
            files,
            folders,
            extensions,
        })
    }

    pub fn contains_file(&self, name: &str) -> bool {
        self.files.contains(name)
    }
}

#[cfg(test)]
static DIR_SCAN_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
impl Context {
    pub fn test_dir_scan_count() -> usize {
        DIR_SCAN_COUNT.load(Ordering::Relaxed)
    }
}
