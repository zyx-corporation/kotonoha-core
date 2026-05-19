//! Git adapter — content-lineage anchors for MeaningDelta (M1).
//!
//! Uses the `git` CLI via subprocess (no libgit2). Issue: [#23](https://github.com/zyx-corporation/kotonoha-core/issues/23).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolved Git repository context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepoContext {
    pub root: PathBuf,
    pub commit: String,
    pub branch: Option<String>,
    pub detached: bool,
}

/// Porcelain-derived working tree summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingTreeStatus {
    pub dirty: bool,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
}

/// Unified diff text from Git.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffText {
    pub text: String,
}

/// Git subprocess or parse failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    NotARepository,
    CommandFailed { argv: String, detail: String },
    InvalidUtf8,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotARepository => f.write_str("not a git repository"),
            GitError::CommandFailed { argv, detail } => {
                write!(f, "git {argv} failed: {detail}")
            }
            GitError::InvalidUtf8 => f.write_str("git output is not valid UTF-8"),
        }
    }
}

impl std::error::Error for GitError {}

/// Discover repository root and HEAD from `start` (or current directory).
pub fn discover_repo(start: Option<&Path>) -> Result<GitRepoContext, GitError> {
    let start = start
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = git_output(
        &start,
        &["rev-parse", "--show-toplevel"],
        "rev-parse --show-toplevel",
    )?;
    let root = PathBuf::from(root.trim());
    let commit = git_output_optional(&root, &["rev-parse", "HEAD"], "rev-parse HEAD")
        .unwrap_or_else(|| "(no commits yet)".to_string());
    let branch = git_output(
        &root,
        &["branch", "--show-current"],
        "branch --show-current",
    )
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());
    let detached = branch.is_none();
    Ok(GitRepoContext {
        root,
        commit,
        branch,
        detached,
    })
}

/// Working tree status via `git status --porcelain`.
pub fn working_tree_status(ctx: &GitRepoContext) -> Result<WorkingTreeStatus, GitError> {
    let out = git_output(&ctx.root, &["status", "--porcelain"], "status --porcelain")?;
    let mut staged = 0usize;
    let mut unstaged = 0usize;
    let mut untracked = 0usize;
    for line in out.lines() {
        if line.is_empty() {
            continue;
        }
        let bytes = line.as_bytes();
        if bytes.len() < 2 {
            continue;
        }
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        if x == '?' && y == '?' {
            untracked += 1;
            continue;
        }
        if x != ' ' {
            staged += 1;
        }
        if y != ' ' {
            unstaged += 1;
        }
    }
    let dirty = staged + unstaged + untracked > 0;
    Ok(WorkingTreeStatus {
        dirty,
        staged_count: staged,
        unstaged_count: unstaged,
        untracked_count: untracked,
    })
}

/// Unified diff for unstaged changes (`git diff`), optionally scoped to `path` (repo-relative).
pub fn diff_unstaged(ctx: &GitRepoContext, path: Option<&Path>) -> Result<GitDiffText, GitError> {
    let mut args = vec!["diff"];
    let rel = path.map(|p| path_relative_to_root(ctx, p)).transpose()?;
    if let Some(ref r) = rel {
        args.push(r.as_str());
    }
    let text = git_output(&ctx.root, &args, "diff")?;
    Ok(GitDiffText { text })
}

/// Repo-relative path for `path` (must be inside the repository root).
pub fn path_relative_to_root(ctx: &GitRepoContext, path: &Path) -> Result<String, GitError> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        ctx.root.join(path)
    };
    let rel = abs
        .strip_prefix(&ctx.root)
        .map_err(|_| GitError::CommandFailed {
            argv: "path".into(),
            detail: format!("path {} is outside repository root", path.display()),
        })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn git_output_optional(cwd: &Path, args: &[&str], label: &str) -> Option<String> {
    git_output(cwd, args, label).ok()
}

fn git_output(cwd: &Path, args: &[&str], label: &str) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| GitError::CommandFailed {
            argv: label.to_string(),
            detail: e.to_string(),
        })?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim_end().to_string())
            .map_err(|_| GitError::InvalidUtf8)
    } else if label.contains("show-toplevel") {
        Err(GitError::NotARepository)
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(GitError::CommandFailed {
            argv: label.to_string(),
            detail: if detail.is_empty() {
                format!("exit {:?}", output.status.code())
            } else {
                detail
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;

    fn run_git(dir: &Path, args: &[&str]) {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git");
        assert!(status.success(), "git {:?} in {}", args, dir.display());
    }

    #[test]
    fn discover_and_diff_in_temp_repo() {
        if Command::new("git").arg("--version").output().is_err() {
            eprintln!("skip discover_and_diff_in_temp_repo: git not installed");
            return;
        }
        let base = std::env::temp_dir().join(format!("kotonoha-git-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        run_git(&base, &["init"]);
        run_git(&base, &["config", "user.email", "test@example.invalid"]);
        run_git(&base, &["config", "user.name", "kotonoha test"]);
        fs::write(base.join("a.txt"), "hello\n").unwrap();
        run_git(&base, &["add", "a.txt"]);
        run_git(&base, &["commit", "-m", "init"]);

        let ctx = discover_repo(Some(&base)).expect("discover");
        assert!(!ctx.commit.is_empty());
        assert!(!ctx.detached || ctx.branch.is_some());

        fs::write(base.join("a.txt"), "hello\nworld\n").unwrap();
        let status = working_tree_status(&ctx).expect("status");
        assert!(status.dirty);

        let diff = diff_unstaged(&ctx, Some(Path::new("a.txt"))).expect("diff");
        assert!(diff.text.contains("world"));

        let _ = fs::remove_dir_all(&base);
    }
}
