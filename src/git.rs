use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
}

pub fn require_repo() -> Result<(), String> {
    match run(&["rev-parse", "--is-inside-work-tree"]).as_deref() {
        Some("true") => Ok(()),
        _ => Err("Not inside a git repository.".into()),
    }
}

pub fn get(key: &str) -> Option<String> {
    run(&["config", "--local", key]).filter(|v| !v.is_empty())
}

/// Value Git would actually use, whatever scope it comes from.
pub fn get_effective(key: &str) -> Option<String> {
    run(&["config", key]).filter(|v| !v.is_empty())
}

pub fn set(key: &str, value: &str) -> Result<(), String> {
    run(&["config", "--local", key, value])
        .map(drop)
        .ok_or_else(|| format!("git config --local {key} failed"))
}

/// Unset a local key; a key that is not set is not an error.
pub fn unset(key: &str) {
    run(&["config", "--local", "--unset", key]);
}

pub fn origin_url() -> Option<String> {
    run(&["remote", "get-url", "origin"])
}

/// Opens `file` in Git's editor (GIT_EDITOR, core.editor, VISUAL, EDITOR).
/// `git config --edit` only launches the editor, so it works for any file and
/// handles editor commands with arguments or spaces on every platform.
pub fn edit(file: &Path) -> Result<(), String> {
    let status = Command::new("git")
        .args(["config", "--file"])
        .arg(file)
        .arg("--edit")
        .status()
        .map_err(|e| format!("Could not run git: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("The editor exited with an error.".into())
    }
}

pub fn hooks_dir() -> Option<PathBuf> {
    run(&["rev-parse", "--git-path", "hooks"]).map(PathBuf::from)
}
