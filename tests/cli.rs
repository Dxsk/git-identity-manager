use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const CONFIG: &str = r#"{
  "identities": [
    { "label": "Personal", "name": "Me", "email": "me@example.com",
      "signingKey": "ssh-ed25519 AAAAtest", "remotes": ["github\\.com[:/]me/"] },
    { "label": "Work", "name": "Me At Work", "email": "me@corp.example",
      "remotes": ["gitlab\\.corp[:/]"] }
  ]
}"#;

struct Sandbox {
    dir: TempDir,
    repo: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("identities.json"), CONFIG).unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir(&repo).unwrap();
        let sb = Sandbox { dir, repo };
        sb.git(&["init", "-q"]);
        sb
    }

    fn repo(&self) -> &Path {
        &self.repo
    }

    fn isolate(&self, cmd: &mut Command) {
        cmd.current_dir(self.repo())
            .env(
                "GIT_IDENTITY_CONFIG",
                self.dir.path().join("identities.json"),
            )
            .env("GIT_CONFIG_GLOBAL", self.dir.path().join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1");
    }

    fn git(&self, args: &[&str]) -> String {
        let mut cmd = Command::new("git");
        self.isolate(&mut cmd);
        let out = cmd.args(args).output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_with_editor(args, "true")
    }

    /// `editor` is a shell command Git runs with the file path appended.
    fn run_with_editor(&self, args: &[&str], editor: &str) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-identity"));
        self.isolate(&mut cmd);
        cmd.env("GIT_EDITOR", editor).args(args).output().unwrap()
    }

    fn config(&self) -> PathBuf {
        self.dir.path().join("identities.json")
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn list_shows_identities() {
    let sb = Sandbox::new();
    let out = sb.run(&["--list"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("Personal | Me <me@example.com> [signing: ssh-ed25519 AAAA...]"));
    assert!(text.contains("Work | Me At Work <me@corp.example> (remotes: gitlab\\.corp[:/])"));
}

#[test]
fn use_sets_identity_and_signing() {
    let sb = Sandbox::new();
    assert!(sb.run(&["use", "personal"]).status.success());
    assert_eq!(
        sb.git(&["config", "--local", "user.email"]),
        "me@example.com"
    );
    assert_eq!(sb.git(&["config", "--local", "gpg.format"]), "ssh");
    assert_eq!(sb.git(&["config", "--local", "commit.gpgsign"]), "true");

    // Switching to an identity without a key clears the signing config.
    assert!(sb.run(&["use", "Work"]).status.success());
    assert_eq!(sb.git(&["config", "--local", "user.name"]), "Me At Work");
    assert_eq!(sb.git(&["config", "--local", "user.signingkey"]), "");
    assert_eq!(sb.git(&["config", "--local", "commit.gpgsign"]), "");
}

#[test]
fn auto_uses_origin_remote() {
    let sb = Sandbox::new();
    sb.git(&["remote", "add", "origin", "git@gitlab.corp:team/repo.git"]);
    assert!(sb.run(&["auto"]).status.success());
    assert_eq!(
        sb.git(&["config", "--local", "user.email"]),
        "me@corp.example"
    );
}

#[test]
fn unset_and_current() {
    let sb = Sandbox::new();
    sb.run(&["use", "Personal"]);
    assert!(stdout(&sb.run(&["--current"])).contains("me@example.com"));

    assert!(sb.run(&["--unset"]).status.success());
    assert_eq!(sb.git(&["config", "--local", "user.name"]), "");
    assert_eq!(sb.git(&["config", "--local", "gpg.format"]), "");
    assert!(stdout(&sb.run(&["--current"])).contains("No local identity set"));
}

#[test]
fn hook_is_installed_once() {
    let sb = Sandbox::new();
    assert!(sb.run(&["--hook"]).status.success());
    assert!(stdout(&sb.run(&["--hook"])).contains("already installed"));
    let hook = fs::read_to_string(sb.repo().join(".git/hooks/post-checkout")).unwrap();
    assert!(hook.starts_with("#!/bin/sh\n"));
    assert_eq!(hook.matches("# git-identity reminder").count(), 1);
}

#[test]
fn add_and_remove() {
    let sb = Sandbox::new();
    let out = sb.run(&[
        "add",
        "Open Source",
        "--name",
        "Me",
        "--email=oss@example.com",
        "-r",
        "codeberg\\.org[:/]me/",
        "-r",
        "github\\.com[:/]me-oss/",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("git identity use \"Open Source\""));

    let list = stdout(&sb.run(&["ls"]));
    assert!(list.contains("Open Source | Me <oss@example.com>"));
    assert!(list.contains("codeberg\\.org[:/]me/, github\\.com[:/]me-oss/"));

    // Duplicate labels and missing fields are refused without a terminal.
    assert!(
        !sb.run(&["add", "open source", "-n", "x", "-e", "x@y"])
            .status
            .success()
    );
    assert!(!sb.run(&["new", "Other", "--name", "Me"]).status.success());

    assert!(sb.run(&["rm", "open source"]).status.success());
    assert!(!stdout(&sb.run(&["list"])).contains("Open Source"));
    assert!(!sb.run(&["rm", "open source"]).status.success());
}

#[test]
fn add_creates_the_config_file() {
    let sb = Sandbox::new();
    fs::remove_file(sb.dir.path().join("identities.json")).unwrap();
    assert!(
        sb.run(&["add", "Solo", "-n", "Me", "-e", "me@example.com"])
            .status
            .success()
    );
    let raw = fs::read_to_string(sb.dir.path().join("identities.json")).unwrap();
    assert!(raw.contains("\"label\": \"Solo\""));
    assert!(!raw.contains("signingKey"));
}

#[test]
fn edit_saves_valid_changes() {
    let sb = Sandbox::new();
    // `-i.bak` rather than `-i`: BSD sed on macOS needs a backup suffix.
    let out = sb.run_with_editor(&["edit"], "sed -i.bak s/Work/Job/");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("Config saved: 2 identities"));
    assert!(stdout(&out).contains(&format!("Editing {}", sb.config().display())));
    assert!(stdout(&sb.run(&["ls"])).contains("Job | Me At Work"));

    let out = sb.run_with_editor(&["e"], "true");
    assert!(stdout(&out).contains("No changes."));
}

#[test]
fn edit_restores_the_config_when_the_result_is_invalid() {
    let sb = Sandbox::new();
    let out = sb.run_with_editor(&["edit"], "echo broken >");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not valid"));

    assert_eq!(fs::read_to_string(sb.config()).unwrap(), CONFIG);
    let rejected = sb.dir.path().join("identities.json.rejected");
    assert_eq!(fs::read_to_string(rejected).unwrap().trim(), "broken");
}

#[test]
fn edit_creates_a_missing_config() {
    let sb = Sandbox::new();
    fs::remove_file(sb.config()).unwrap();
    let out = sb.run(&["edit"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Config created:"));
    assert!(
        fs::read_to_string(sb.config())
            .unwrap()
            .contains("\"identities\"")
    );
}

#[test]
fn shortcuts() {
    let sb = Sandbox::new();
    assert!(sb.run(&["sw", "Work"]).status.success());
    assert!(stdout(&sb.run(&["whoami"])).contains("me@corp.example"));
    assert!(stdout(&sb.run(&["cur"])).contains("me@corp.example"));
    assert!(stdout(&sb.run(&["h"])).contains("Usage: git identity"));
}

#[test]
fn errors_exit_nonzero() {
    let sb = Sandbox::new();
    assert!(!sb.run(&["use", "nobody"]).status.success());
    assert!(!sb.run(&["--bogus"]).status.success());
}
