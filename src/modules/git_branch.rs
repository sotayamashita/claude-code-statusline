use super::{Module, ModuleConfig};
use crate::types::context::Context;
use std::process::Command;

/// Module that displays the current Git branch or short SHA when detached
pub struct GitBranchModule;

impl GitBranchModule {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }
}

impl Default for GitBranchModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for GitBranchModule {
    fn name(&self) -> &str {
        "git_branch"
    }

    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool {
        // disabled フラグを確認
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::GitBranchConfig>()
        {
            if cfg.disabled {
                return false;
            }
        }

        // Git リポジトリ配下のみ表示（git2 失敗時は git コマンドでフォールバック）
        if git2::Repository::discover(&context.current_dir).is_ok() {
            return true;
        }
        // Fallback: `git -C <cwd> rev-parse --is-inside-work-tree`
        if let Ok(out) = Command::new("git")
            .args([
                "-C",
                context.current_dir.to_string_lossy().as_ref(),
                "rev-parse",
                "--is-inside-work-tree",
            ])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                return s.trim() == "true";
            }
        }
        false
    }

    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String {
        // Try git2 first
        let value = match git2::Repository::discover(&context.current_dir) {
            Ok(repo) => {
                if let Ok(head) = repo.head() {
                    if head.is_branch() {
                        head.shorthand().unwrap_or("").to_string()
                    } else if let Some(oid) = head.target() {
                        let s = oid.to_string();
                        s.chars().take(7).collect()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        };

        let value = if value.is_empty() {
            // Fallback using `git` command
            let cwd = context.current_dir.to_string_lossy().to_string();
            // Try branch name first
            if let Ok(out) = Command::new("git")
                .args(["-C", &cwd, "rev-parse", "--abbrev-ref", "HEAD"])
                .output()
            {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !s.is_empty() && s != "HEAD" {
                        s
                    } else {
                        // Detached HEAD -> short sha
                        if let Ok(out2) = Command::new("git")
                            .args(["-C", &cwd, "rev-parse", "--short", "HEAD"])
                            .output()
                        {
                            if out2.status.success() {
                                String::from_utf8_lossy(&out2.stdout).trim().to_string()
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            value
        };

        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::GitBranchConfig>()
        {
            use std::collections::HashMap;
            let mut tokens = HashMap::new();
            tokens.insert("branch", value.clone());
            tokens.insert("symbol", cfg.symbol.clone());
            return crate::style::render_with_style_template(cfg.format(), &tokens, cfg.style());
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::context::Context;
    use rstest::*;

    // git2 と tempfile を利用して一時リポジトリを構築
    use git2::{Repository, Signature};
    use std::fs::{File, create_dir_all};
    use std::io::Write as _; // for file writing
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // Helper: ClaudeInput -> Context 生成
    fn make_context(cwd: &str) -> Context {
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
                project_dir: Some(cwd.to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    // Helper: 空コミットを作り main ブランチをセット
    fn init_repo_with_branch(path: &Path, _branch: &str) -> Repository {
        let repo = Repository::init(path).expect("init repo");

        // 初回コミット
        let sig = Signature::now("Tester", "tester@example.com").unwrap();
        let mut index = repo.index().unwrap();

        // 何かしらファイルを作って add
        let file_path = path.join("README.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test").unwrap();
        file.sync_all().unwrap();

        index.add_path(Path::new("README.md")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let commit_id = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
        let commit = repo.find_commit(commit_id).unwrap();

        // ここではデフォルトブランチ（master/main）に任せる
        // 明示的に借用を解放
        drop(commit);
        drop(tree);

        repo
    }

    // Helper: Detached HEAD を作る
    fn detach_head(repo: &Repository) {
        let head = repo.head().unwrap();
        let target = head.target().unwrap();
        repo.set_head_detached(target).unwrap();
    }

    #[fixture]
    fn temp_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        (dir, root)
    }

    #[rstest]
    fn repo_outside_should_not_display() {
        let tmp = tempdir().unwrap();
        let outside = tmp.path().join("outside");
        create_dir_all(&outside).unwrap();

        let ctx = make_context(outside.to_str().unwrap());

        // 未実装モジュールを前提に、型・メソッドの存在をテストで宣言
        let module = crate::modules::git_branch::GitBranchModule::new();
        let show = module.should_display(&ctx, &ctx.config.git_branch);
        assert!(!show);
    }

    #[rstest]
    fn repo_inside_on_main_should_display_branch(temp_repo: (tempfile::TempDir, PathBuf)) {
        let (_d, root) = temp_repo;
        let repo = init_repo_with_branch(&root, "main");

        let ctx = make_context(root.to_str().unwrap());
        let module = crate::modules::git_branch::GitBranchModule::new();
        assert!(module.should_display(&ctx, &ctx.config.git_branch));

        let rendered = module.render(&ctx, &ctx.config.git_branch);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        assert!(plain.contains("main") || plain.contains("master"));
        drop(repo);
    }

    #[rstest]
    fn detached_head_renders_short_sha(temp_repo: (tempfile::TempDir, PathBuf)) {
        let (_d, root) = temp_repo;
        let repo = init_repo_with_branch(&root, "main");
        detach_head(&repo);

        let ctx = make_context(root.to_str().unwrap());
        let module = crate::modules::git_branch::GitBranchModule::new();
        let rendered = module.render(&ctx, &ctx.config.git_branch);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        // Extract the last whitespace-separated token (branch or short SHA)
        let last = plain.split_whitespace().last().unwrap_or("");
        assert!(last.len() >= 7 && last.len() <= 8);
        assert!(last.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[rstest]
    fn disabled_flag_hides_output(temp_repo: (tempfile::TempDir, PathBuf)) {
        let (_d, root) = temp_repo;
        let _repo = init_repo_with_branch(&root, "main");
        let mut ctx = make_context(root.to_str().unwrap());

        // disabled = true を設定
        ctx.config.git_branch.disabled = true;

        let module = crate::modules::git_branch::GitBranchModule::new();
        assert!(!module.should_display(&ctx, &ctx.config.git_branch));
    }
}
