use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Template written by `git identity init`.
pub const EXAMPLE: &str = include_str!("../identities.example.json");

// Unknown keys land in `extra`, so `add` and `remove` can rewrite the file
// without dropping anything the user put there by hand.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub identities: Vec<Identity>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub label: String,
    pub name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_key: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remotes: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Identity {
    /// `label | name <email>`, the line shown in the picker.
    pub fn entry(&self) -> String {
        format!("{} | {} <{}>", self.label, self.name, self.email)
    }

    pub fn matches_remote(&self, url: &str) -> bool {
        self.remotes
            .iter()
            .any(|pattern| match Regex::new(pattern) {
                Ok(re) => re.is_match(url),
                Err(err) => {
                    eprintln!(
                        "warning: invalid remote pattern {pattern:?} in {:?}: {err}",
                        self.label
                    );
                    false
                }
            })
    }
}

/// Resolution order: `GIT_IDENTITY_CONFIG`, then `$XDG_CONFIG_HOME`, then the
/// platform default (`%APPDATA%` on Windows, `~/.config` elsewhere).
pub fn path() -> PathBuf {
    if let Some(p) = env::var_os("GIT_IDENTITY_CONFIG").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    let base = env::var_os("XDG_CONFIG_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(platform_config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("git-identity").join("identities.json")
}

#[cfg(windows)]
fn platform_config_dir() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(not(windows))]
fn platform_config_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
}

pub fn load() -> Result<Config, String> {
    load_existing()?.ok_or_else(|| {
        format!(
            "Config not found: {}\nRun `git identity add` or `git identity init`, or set GIT_IDENTITY_CONFIG to point to your identities.json",
            path().display()
        )
    })
}

/// Like `load`, but a missing file is `Ok(None)` rather than an error.
pub fn load_existing() -> Result<Option<Config>, String> {
    let path = path();
    match fs::read_to_string(&path) {
        Ok(raw) => parse(&raw)
            .map(Some)
            .map_err(|err| format!("Invalid config {}: {err}", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("{}: {err}", path.display())),
    }
}

pub fn parse(raw: &str) -> Result<Config, serde_json::Error> {
    serde_json::from_str(raw)
}

impl Config {
    pub fn save(&self) -> Result<(), String> {
        let path = path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        json.push('\n');
        fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn find(&self, label: &str) -> Option<&Identity> {
        self.identities
            .iter()
            .find(|id| id.label.eq_ignore_ascii_case(label))
    }

    /// Index of the first identity whose `remotes` match `url`.
    pub fn suggest(&self, url: &str) -> Option<usize> {
        self.identities.iter().position(|id| id.matches_remote(url))
    }

    pub fn add(&mut self, id: Identity) -> Result<(), String> {
        if id.label.trim().is_empty() {
            return Err("The label cannot be empty.".into());
        }
        if self.find(&id.label).is_some() {
            return Err(format!(
                "An identity labelled {:?} already exists.",
                id.label
            ));
        }
        if id.name.trim().is_empty() {
            return Err("The name cannot be empty.".into());
        }
        if !id.email.contains('@') {
            return Err(format!(
                "{:?} does not look like an email address.",
                id.email
            ));
        }
        for pattern in &id.remotes {
            Regex::new(pattern).map_err(|e| format!("Invalid remote pattern {pattern:?}: {e}"))?;
        }
        self.identities.push(id);
        Ok(())
    }

    pub fn remove(&mut self, label: &str) -> Option<Identity> {
        let idx = self
            .identities
            .iter()
            .position(|id| id.label.eq_ignore_ascii_case(label))?;
        Some(self.identities.remove(idx))
    }
}

/// Builds a `remotes` pattern matching the host and owner of a remote URL, so
/// `git@github.com:me/repo.git` gives `github\.com[:/]me/`.
pub fn remote_pattern(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let rest = rest.split_once('@').map_or(rest, |(_, r)| r);
    let split = rest.find([':', '/'])?;
    let (host, path) = (&rest[..split], &rest[split + 1..]);
    // An explicit port, as in ssh://git@host:2222/owner/repo, becomes optional
    // in the pattern so the same owner also matches over HTTPS.
    let (path, port) = match path.split_once('/') {
        Some((port, p)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => {
            (p, r"(?::\d+)?")
        }
        _ => (path, ""),
    };
    let owner = path.split('/').next().filter(|o| !o.is_empty())?;
    if host.is_empty() {
        return None;
    }
    Some(format!(
        "{}{port}[:/]{}/",
        regex_lite::escape(host),
        regex_lite::escape(owner)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_parses() {
        let cfg = parse(EXAMPLE).unwrap();
        assert_eq!(cfg.identities.len(), 2);
        assert_eq!(
            cfg.identities[0].signing_key.as_deref(),
            Some("ABCDEF1234567890")
        );
        assert!(cfg.identities[1].signing_key.is_none());
    }

    #[test]
    fn missing_required_field_is_rejected() {
        assert!(parse(r#"{"identities":[{"label":"x","name":"y"}]}"#).is_err());
    }

    #[test]
    fn suggests_by_remote() {
        let cfg = parse(EXAMPLE).unwrap();
        assert_eq!(cfg.suggest("git@github.com:your-org/repo.git"), Some(1));
        assert_eq!(cfg.suggest("https://gitlab.com/your-org/repo"), Some(1));
        assert_eq!(
            cfg.suggest("https://github.com/your-username/repo"),
            Some(0)
        );
        assert_eq!(cfg.suggest("https://example.com/other/repo"), None);
    }

    #[test]
    fn find_is_case_insensitive() {
        let cfg = parse(EXAMPLE).unwrap();
        assert_eq!(cfg.find("work").unwrap().label, "Work");
        assert!(cfg.find("nope").is_none());
    }

    #[test]
    fn unknown_fields_survive_a_round_trip() {
        let raw =
            r#"{"identities":[{"label":"a","name":"n","email":"a@b","note":"keep"}],"version":2}"#;
        let json = serde_json::to_string(&parse(raw).unwrap()).unwrap();
        assert!(json.contains(r#""note":"keep""#));
        assert!(json.contains(r#""version":2"#));
        assert!(!json.contains("signingKey"));
    }

    #[test]
    fn add_validates() {
        let mut cfg = parse(EXAMPLE).unwrap();
        let id = |label: &str, email: &str, remote: &str| Identity {
            label: label.into(),
            name: "Me".into(),
            email: email.into(),
            remotes: vec![remote.into()],
            ..Default::default()
        };
        assert!(cfg.add(id("work", "x@y", "a")).is_err(), "duplicate label");
        assert!(cfg.add(id("New", "nope", "a")).is_err(), "bad email");
        assert!(cfg.add(id("New", "x@y", "(")).is_err(), "bad regex");
        assert!(cfg.add(id("New", "x@y", "a")).is_ok());
        assert_eq!(cfg.remove("NEW").unwrap().label, "New");
        assert!(cfg.remove("New").is_none());
    }

    #[test]
    fn builds_remote_patterns() {
        let cases = [
            ("git@github.com:Dxsk/repo.git", r"github\.com[:/]Dxsk/"),
            ("https://github.com/Dxsk/repo", r"github\.com[:/]Dxsk/"),
            (
                "ssh://git@forge.example:2222/me/repo.git",
                r"forge\.example(?::\d+)?[:/]me/",
            ),
            (
                "https://user@gitlab.com/group/sub/repo",
                r"gitlab\.com[:/]group/",
            ),
        ];
        for (url, want) in cases {
            let got = remote_pattern(url).unwrap();
            assert_eq!(got, want, "{url}");
            assert!(Regex::new(&got).unwrap().is_match(url), "{url}");
        }
        assert_eq!(remote_pattern("/srv/git/repo"), None);
    }
}
