//! Git status module for displaying repository state
//!
//! This module shows the current state of the git repository including
//! modified files, staged changes, and branch divergence.

use super::{Module, ModuleConfig};
use crate::types::context::Context;

/// Module that summarizes Git working tree and index state
///
/// Displays indicators for:
/// - Modified files (working tree changes)
/// - Staged files (index changes)
/// - Untracked files
/// - Branch ahead/behind status relative to upstream
/// - Conflicted files during merge
///
/// # Configuration
///
/// ```toml
/// [git_status]
/// format = "[$all_status$ahead_behind]($style)"
/// style = "bold red"
/// conflicted = "="
/// ahead = "⇡"
/// behind = "⇣"
/// diverged = "⇕"
/// untracked = "?"
/// stashed = "$"
/// modified = "!"
/// staged = "+"
/// renamed = "»"
/// deleted = "✘"
/// disabled = false
/// ```
///
/// # Display Format
///
/// Shows compact symbols for repository state, e.g.:
/// - `[!]` - Has modified files
/// - `[+]` - Has staged changes
/// - `[⇡3]` - Ahead by 3 commits
/// - `[⇣2]` - Behind by 2 commits
pub struct GitStatusModule;

impl GitStatusModule {
    pub fn new() -> Self {
        Self
    }

    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }
}

impl Default for GitStatusModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for GitStatusModule {
    fn name(&self) -> &str {
        "git_status"
    }

    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool {
        // disabled フラグ
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::GitStatusConfig>()
            && cfg.disabled
        {
            return false;
        }
        context.repo().is_ok()
    }

    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String {
        let mut repo = match context.repo() {
            Ok(r) => r,
            Err(_) => return String::new(),
        };

        // Resolve config
        let cfg = match config
            .as_any()
            .downcast_ref::<crate::types::config::GitStatusConfig>()
        {
            Some(c) => c,
            None => return String::new(),
        };

        // Count statuses (index vs worktree)
        let mut conflicted = 0u32;
        // presence only (count stashes)
        let mut deleted = 0u32; // staged deletions
        let mut renamed = 0u32; // staged renames
        let mut modified = 0u32; // working tree modifications (unstaged)
        let mut typechanged = 0u32; // staged type changes
        let mut staged = 0u32; // staged (added/modified/renamed/deleted)
        let mut untracked = 0u32; // untracked files

        // Statuses
        if let Ok(stats) = repo.statuses(None) {
            use git2::Status;
            for s in stats.iter().map(|e| e.status()) {
                if s.intersects(Status::CONFLICTED) {
                    conflicted += 1;
                    continue;
                }
                if s.intersects(Status::WT_NEW) {
                    untracked += 1;
                }
                if s.intersects(Status::WT_MODIFIED) {
                    modified += 1;
                }
                if s.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED) {
                    staged += 1;
                }
                if s.intersects(Status::INDEX_RENAMED) {
                    renamed += 1;
                    staged += 1;
                }
                if s.intersects(Status::INDEX_DELETED) {
                    deleted += 1;
                    staged += 1;
                }
                if s.intersects(Status::INDEX_TYPECHANGE) {
                    typechanged += 1;
                    staged += 1;
                }
            }
        }

        // Stash presence (count stashes)
        let mut stash_count = 0u32;
        let _ = repo.stash_foreach(|_, _, _| {
            stash_count += 1;
            true
        });
        let stashed = stash_count;

        // Ahead/behind/diverged
        let mut ahead_behind = String::new();
        if let Ok(head) = repo.head()
            && head.is_branch()
            && let Some(local_oid) = head.target()
        {
            let shorthand = head.shorthand().unwrap_or("");
            if let Ok(local_branch) = repo.find_branch(shorthand, git2::BranchType::Local)
                && let Ok(up_branch) = local_branch.upstream()
                && let Some(up_oid) = up_branch.get().target()
                && let Ok((ahead, behind)) = repo.graph_ahead_behind(local_oid, up_oid)
            {
                if ahead > 0 && behind > 0 {
                    if !cfg.symbols.diverged.is_empty() {
                        ahead_behind = cfg.symbols.diverged.clone();
                    }
                } else if ahead > 0 {
                    if !cfg.symbols.ahead.is_empty() {
                        ahead_behind = format!("{}{}", cfg.symbols.ahead, ahead);
                    }
                } else if behind > 0 && !cfg.symbols.behind.is_empty() {
                    ahead_behind = format!("{}{}", cfg.symbols.behind, behind);
                }
            }
        }

        // Compose $all_status: conflicted stashed deleted renamed modified typechanged staged untracked
        let mut all_status = String::new();
        let mut push_sym = |sym: &str, count: u32| {
            if count > 0 && !sym.is_empty() {
                use std::fmt::Write as _;
                let _ = write!(all_status, "{sym}{count}");
            }
        };

        push_sym(&cfg.symbols.conflicted, conflicted);
        push_sym(&cfg.symbols.stashed, stashed);
        push_sym(&cfg.symbols.deleted, deleted);
        push_sym(&cfg.symbols.renamed, renamed);
        push_sym(&cfg.symbols.modified, modified);
        push_sym(&cfg.symbols.typechanged, typechanged);
        push_sym(&cfg.symbols.staged, staged);
        push_sym(&cfg.symbols.untracked, untracked);

        // If repository is completely clean (no status symbols and no ahead/behind),
        // suppress the entire module output to avoid showing empty parentheses like `()`.
        if all_status.is_empty() && ahead_behind.is_empty() {
            return String::new();
        }

        // Tokens for template
        use std::collections::HashMap;
        let mut tokens = HashMap::new();
        tokens.insert("all_status", all_status);
        tokens.insert("ahead_behind", ahead_behind);
        tokens.insert("style", cfg.style.clone());

        crate::style::render_with_style_template(cfg.format(), &tokens, cfg.style())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::context::Context;
    use git2::{BranchType, Repository, Signature};
    use rstest::*;
    use std::fs::{File, create_dir_all};
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn make_context(cwd: &str) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: "claude-opus".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: cwd.to_string(),
                project_dir: Some(cwd.to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    fn initial_commit(repo: &Repository, path: &Path) -> git2::Oid {
        let sig = Signature::now("Tester", "tester@example.com").unwrap();
        // create file
        let file_path = path.join("README.md");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "init").unwrap();
        f.sync_all().unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        // Persist index to disk so status reflects a clean state
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap()
    }

    #[fixture]
    fn temp_repo() -> (tempfile::TempDir, PathBuf, Repository) {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let repo = Repository::init(&root).unwrap();
        let c0 = initial_commit(&repo, &root);
        // create main branch if needed and point HEAD to it
        let commit0 = repo.find_commit(c0).unwrap();
        let main_exists = repo.find_branch("main", BranchType::Local).is_ok();
        if !main_exists {
            let _ = repo.branch("main", &commit0, true).unwrap();
        }
        drop(commit0);
        let _ = repo.set_head("refs/heads/main");
        (dir, root, repo)
    }

    #[rstest]
    fn repo_outside_should_not_display() {
        let tmp = tempdir().unwrap();
        let outside = tmp.path().join("outside");
        create_dir_all(&outside).unwrap();

        let ctx = make_context(outside.to_str().unwrap());
        let module = GitStatusModule::new();
        let show = module.should_display(&ctx, &ctx.config.git_status);
        assert!(!show);
    }

    #[rstest]
    fn renders_counts_and_ahead(temp_repo: (tempfile::TempDir, PathBuf, Repository)) {
        use strip_ansi_escapes::strip;
        let (_d, root, repo) = temp_repo;

        // ahead: create local branch upstream at current commit
        let head = repo.head().unwrap();
        let head_commit = repo.find_commit(head.target().unwrap()).unwrap();
        // create an "upstream" branch at the current commit (will be behind after next commit)
        let _ = repo.branch("upstream", &head_commit, true).unwrap();
        // make an extra commit on main so it's ahead by 1
        let sig = Signature::now("Tester", "tester@example.com").unwrap();
        // create a tracked file and commit it as the second commit
        let mut tracked = File::create(root.join("tracked.txt")).unwrap();
        writeln!(tracked, "t1").unwrap();
        tracked.sync_all().unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("tracked.txt")).unwrap();
        let tree_id2 = index.write_tree().unwrap();
        let tree2 = repo.find_tree(tree_id2).unwrap();
        let _c1 = repo
            .commit(Some("HEAD"), &sig, &sig, "second", &tree2, &[&head_commit])
            .unwrap();
        let mut main = repo.find_branch("main", BranchType::Local).unwrap();
        main.set_upstream(Some("upstream")).unwrap();

        // Now create working-tree changes relative to HEAD:
        // staged: add a new file to index (but do not commit)
        let mut f1 = File::create(root.join("staged.txt")).unwrap();
        writeln!(f1, "staged").unwrap();
        f1.sync_all().unwrap();
        let mut index2 = repo.index().unwrap();
        index2.add_path(Path::new("staged.txt")).unwrap();
        index2.write().unwrap();

        // modified: modify the tracked file without staging
        let mut tracked2 = File::create(root.join("tracked.txt")).unwrap();
        writeln!(tracked2, "t2").unwrap();
        tracked2.sync_all().unwrap();

        // untracked: create without adding
        let mut f3 = File::create(root.join("untracked.txt")).unwrap();
        writeln!(f3, "u").unwrap();
        f3.sync_all().unwrap();

        let ctx = make_context(root.to_str().unwrap());
        let module = GitStatusModule::new();
        assert!(module.should_display(&ctx, &ctx.config.git_status));
        let rendered = module.render(&ctx, &ctx.config.git_status);
        let plain = String::from_utf8(strip(rendered)).unwrap();
        // expect substrings: +1 (staged), !1 (modified), ?1 (untracked), ⇡1 (ahead)
        assert!(plain.contains("+1"));
        assert!(plain.contains("!1"));
        assert!(plain.contains("?1"));
        // ahead may be computed; assert presence when upstream is set
        assert!(plain.contains("⇡1"));
    }

    #[rstest]
    fn disabled_flag_hides_output(temp_repo: (tempfile::TempDir, PathBuf, Repository)) {
        let (_d, root, _repo) = temp_repo;
        let mut ctx = make_context(root.to_str().unwrap());
        ctx.config.git_status.disabled = true;
        let module = GitStatusModule::new();
        assert!(!module.should_display(&ctx, &ctx.config.git_status));
    }

    #[rstest]
    fn clean_repo_renders_nothing(temp_repo: (tempfile::TempDir, PathBuf, Repository)) {
        use strip_ansi_escapes::strip;
        let (_d, root, _repo) = temp_repo;
        let ctx = make_context(root.to_str().unwrap());
        let module = GitStatusModule::new();
        let rendered = module.render(&ctx, &ctx.config.git_status);
        let plain = String::from_utf8(strip(rendered)).unwrap();
        println!("clean repo git_status plain='{plain}'");
        assert!(plain.is_empty());
    }
}
