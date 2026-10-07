mod config;
mod git;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use console::{Term, style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, FuzzySelect, Input};

use config::{Config, Identity};

const SIGNING_KEYS: [&str; 3] = ["user.signingkey", "commit.gpgsign", "gpg.format"];

const HOOK_MARKER: &str = "# git-identity reminder";
const HOOK_BODY: &str = r#"# git-identity reminder
if [ -z "$(git config --local user.name 2>/dev/null)" ]; then
  echo ""
  echo "[git-identity] No local identity set. Run 'git identity' to pick one."
fi
"#;

fn usage() -> String {
    format!(
        "git-identity {}
Switch between Git identities per repository.

Usage: git identity [COMMAND]

Commands:                                                        Shortcuts
  (none)            Pick an identity for this repository
  use <label>       Apply the identity with this label           sw, switch
  auto              Apply the identity matching the origin remote
  list              List your identities                         ls
  current           Show the identity set on this repository     cur, whoami
  unset             Remove it (the global config applies again)
  add [<label>]     Add an identity, asking for what is missing  new, append
                      -n, --name <name>
                      -e, --email <email>
                      -k, --signing-key <key>
                      -r, --remote <regex>  (repeatable)
  remove [<label>]  Remove an identity from the config           rm
  edit              Open the config in Git's editor               e
  hook            Install a post-checkout reminder hook
  init              Create a config file from the example template
  path              Print the config file path
  help              Show this help message                       h
  version           Show the version

The --list, --current, --unset, --hook, --help and --version forms are also accepted.
Note: `git identity --help` is intercepted by Git; use `git identity help` instead.
Config: {}",
        env!("CARGO_PKG_VERSION"),
        config::path().display()
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let result = match args.as_slice() {
        [] => cmd_select(),
        ["use" | "switch" | "sw", label] => cmd_use(label),
        ["auto" | "--auto"] => cmd_auto(),
        ["list" | "--list" | "ls"] => cmd_list(),
        ["current" | "--current" | "cur" | "whoami"] => cmd_current(),
        ["add" | "new" | "append", rest @ ..] => cmd_add(rest),
        ["remove" | "rm"] => cmd_remove(None),
        ["remove" | "rm", label] => cmd_remove(Some(label)),
        ["unset" | "--unset"] => cmd_unset(),
        ["hook" | "--hook"] => cmd_hook(),
        ["edit" | "e"] => cmd_edit(),
        ["init"] => cmd_init(),
        ["path"] => {
            println!("{}", config::path().display());
            Ok(())
        }
        ["help" | "h" | "-h" | "--help"] => {
            println!("{}", usage());
            Ok(())
        }
        ["version" | "-V" | "--version"] => {
            println!("git-identity {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => Err(format!(
            "Unknown arguments: {}\n\n{}",
            args.join(" "),
            usage()
        )),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{}", style(msg).red().for_stderr());
            ExitCode::FAILURE
        }
    }
}

fn cmd_list() -> Result<(), String> {
    let cfg = config::load()?;
    for id in &cfg.identities {
        let mut line = id.entry();
        if let Some(key) = &id.signing_key {
            let short: String = key.chars().take(16).collect();
            line.push_str(&format!(" [signing: {short}...]"));
        }
        if !id.remotes.is_empty() {
            line.push_str(&format!(" (remotes: {})", id.remotes.join(", ")));
        }
        println!("{line}");
    }
    Ok(())
}

fn cmd_current() -> Result<(), String> {
    git::require_repo()?;
    let name = git::get("user.name");
    let email = git::get("user.email");
    let signing = git::get("user.signingkey");

    if name.is_none() && email.is_none() {
        println!(
            "{}",
            style("No local identity set. Using global config.").yellow()
        );
        return Ok(());
    }

    let not_set = || "(not set)".to_owned();
    println!("{}", style("Current local identity:").bold());
    println!(
        "  user.name       = {}",
        style(name.unwrap_or_else(not_set)).green()
    );
    println!(
        "  user.email      = {}",
        style(email.unwrap_or_else(not_set)).green()
    );
    if let Some(key) = signing {
        println!("  user.signingkey = {}", style(key).green());
    }
    Ok(())
}

fn cmd_unset() -> Result<(), String> {
    git::require_repo()?;
    for key in ["user.name", "user.email"].into_iter().chain(SIGNING_KEYS) {
        git::unset(key);
    }
    println!(
        "{} Falling back to global config.",
        style("Local identity removed.").green()
    );
    Ok(())
}

fn cmd_hook() -> Result<(), String> {
    git::require_repo()?;
    let dir = git::hooks_dir().ok_or("Could not locate the hooks directory.")?;
    let file = dir.join("post-checkout");

    let existing = fs::read_to_string(&file).ok();
    if existing.as_deref().is_some_and(|s| s.contains(HOOK_MARKER)) {
        println!(
            "{} {}",
            style("Hook already installed:").yellow(),
            file.display()
        );
        return Ok(());
    }

    let content = match existing {
        Some(prev) => format!("{}\n\n{HOOK_BODY}", prev.trim_end()),
        None => format!("#!/bin/sh\n{HOOK_BODY}"),
    };
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    fs::write(&file, content).map_err(|e| format!("{}: {e}", file.display()))?;
    make_executable(&file)?;

    println!(
        "{} {}",
        style("Post-checkout hook installed:").green(),
        file.display()
    );
    Ok(())
}

#[cfg(unix)]
fn make_executable(file: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(file).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(file, perms).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn make_executable(_: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn cmd_init() -> Result<(), String> {
    let path = config::path();
    if path.exists() {
        return Err(format!("Config already exists: {}", path.display()));
    }
    write_example(&path)?;
    println!("Edit it with `git identity edit` to add your identities.");
    Ok(())
}

fn write_example(path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(path, config::EXAMPLE).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("{} {}", style("Config created:").green(), path.display());
    Ok(())
}

/// Opens the config in Git's editor and validates it afterwards. An invalid
/// file can be reopened; if the user gives up, the previous version is put
/// back and the rejected edit is kept next to it so nothing is lost.
fn cmd_edit() -> Result<(), String> {
    let path = config::path();
    if !path.exists() {
        write_example(&path)?;
    }
    let read = |p: &Path| fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let original = read(&path)?;

    loop {
        git::edit(&path)?;
        let edited = read(&path)?;
        let err = match config::parse(&edited) {
            Ok(cfg) => {
                if edited == original {
                    println!("No changes.");
                } else {
                    let n = cfg.identities.len();
                    let plural = if n == 1 { "identity" } else { "identities" };
                    println!(
                        "{} {n} {plural} in {}",
                        style("Config saved:").green(),
                        path.display()
                    );
                }
                return Ok(());
            }
            Err(err) => err,
        };

        eprintln!(
            "{} {err}",
            style("The config is not valid:").red().for_stderr()
        );
        let retry = interactive()
            && Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Open the editor again?")
                .default(true)
                .interact()
                .map_err(|e| e.to_string())?;
        if !retry {
            let mut rejected = path.clone().into_os_string();
            rejected.push(".rejected");
            let rejected = PathBuf::from(rejected);
            fs::write(&rejected, &edited).map_err(|e| format!("{}: {e}", rejected.display()))?;
            fs::write(&path, &original).map_err(|e| format!("{}: {e}", path.display()))?;
            return Err(format!(
                "Changes discarded, the previous config is back.\nYour edit was saved to {}",
                rejected.display()
            ));
        }
    }
}

fn cmd_use(label: &str) -> Result<(), String> {
    let cfg = config::load()?;
    git::require_repo()?;
    let id = cfg
        .find(label)
        .ok_or_else(|| format!("No identity labelled {label:?}. See `git identity list`."))?;
    apply(id)
}

fn cmd_auto() -> Result<(), String> {
    let cfg = config::load()?;
    git::require_repo()?;
    let url = git::origin_url().ok_or("No origin remote to match against.")?;
    let idx = cfg
        .suggest(&url)
        .ok_or_else(|| format!("No identity matches remote {url}."))?;
    apply(&cfg.identities[idx])
}

fn cmd_select() -> Result<(), String> {
    let cfg = config::load()?;
    git::require_repo()?;
    if cfg.identities.is_empty() {
        return Err("No identities configured.".into());
    }
    if !interactive() {
        return Err(
            "Interactive selection needs a terminal. Use `git identity use <label>`.".into(),
        );
    }

    let suggestion = git::origin_url().and_then(|url| cfg.suggest(&url));
    let prompt = match suggestion {
        Some(i) => format!(
            "Select git identity (suggested: {})",
            cfg.identities[i].label
        ),
        None => "Select git identity".to_owned(),
    };

    match pick(&cfg, &prompt, suggestion)? {
        Some(i) => apply(&cfg.identities[i]),
        None => Ok(()),
    }
}

fn interactive() -> bool {
    Term::stderr().is_term()
}

/// Fuzzy picker over all identities; `None` when the user presses Esc.
fn pick(cfg: &Config, prompt: &str, default: Option<usize>) -> Result<Option<usize>, String> {
    let items: Vec<String> = cfg.identities.iter().map(Identity::entry).collect();
    FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .default(default.unwrap_or(0))
        .max_length(15)
        .interact_opt()
        .map_err(|e| e.to_string())
}

#[derive(Default)]
struct AddArgs {
    label: Option<String>,
    name: Option<String>,
    email: Option<String>,
    signing_key: Option<String>,
    remotes: Vec<String>,
    has_options: bool,
}

fn parse_add(args: &[&str]) -> Result<AddArgs, String> {
    let mut out = AddArgs::default();
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        if !arg.starts_with('-') {
            if out.label.replace(arg.to_owned()).is_some() {
                return Err(format!(
                    "Unexpected argument {arg:?}. Quote the label if it has spaces."
                ));
            }
            continue;
        }
        // Accept both `--name value` and `--name=value`.
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_owned())),
            None => (arg, None),
        };
        let value = inline
            .or_else(|| it.next().map(|v| v.to_string()))
            .ok_or_else(|| format!("{flag} needs a value."))?;
        match flag {
            "-n" | "--name" => out.name = Some(value),
            "-e" | "--email" => out.email = Some(value),
            "-k" | "--signing-key" => out.signing_key = Some(value),
            "-r" | "--remote" => out.remotes.push(value),
            _ => return Err(format!("Unknown option {flag:?} for add.")),
        }
        out.has_options = true;
    }
    Ok(out)
}

fn ask(prompt: &str, initial: Option<String>, optional: bool) -> Result<String, String> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt(prompt)
        .allow_empty(optional);
    // Prefilled rather than a default, so the user can edit or clear it.
    if let Some(text) = initial {
        input = input.with_initial_text(text);
    }
    input
        .interact_text()
        .map(|s| s.trim().to_owned())
        .map_err(|e| e.to_string())
}

fn cmd_add(args: &[&str]) -> Result<(), String> {
    let args = parse_add(args)?;
    let mut cfg = config::load_existing()?.unwrap_or_default();
    let tty = interactive();
    // With no options at all, also ask for the optional fields.
    let wizard = tty && !args.has_options;

    let required =
        |value: Option<String>, prompt: &str, flag: &str, initial: Option<String>| match value {
            Some(v) => Ok(v),
            None if tty => ask(prompt, initial, false),
            None => Err(format!(
                "Missing {flag}. Pass it, or run the command in a terminal to be prompted."
            )),
        };
    let label = required(args.label, "Label", "<label>", None)?;
    let name = required(args.name, "Name", "--name", git::get_effective("user.name"))?;
    let email = required(
        args.email,
        "Email",
        "--email",
        git::get_effective("user.email"),
    )?;

    let (signing_key, remotes) = if wizard {
        let key = ask("Signing key (empty for none)", None, true)?;
        let suggested = git::origin_url().and_then(|url| config::remote_pattern(&url));
        let remotes = ask(
            "Remote patterns, space separated (empty for none)",
            suggested,
            true,
        )?;
        (
            Some(key).filter(|k| !k.is_empty()),
            remotes.split_whitespace().map(str::to_owned).collect(),
        )
    } else {
        (args.signing_key, args.remotes)
    };

    cfg.add(Identity {
        label,
        name,
        email,
        signing_key,
        remotes,
        ..Default::default()
    })?;
    cfg.save()?;

    let id = cfg.identities.last().expect("identity was just added");
    println!("{} {}", style("Identity added:").green(), id.entry());
    println!("Apply it with: git identity use {}", quote_label(&id.label));
    Ok(())
}

fn cmd_remove(label: Option<&str>) -> Result<(), String> {
    let mut cfg = config::load()?;
    if cfg.identities.is_empty() {
        return Err("No identities configured.".into());
    }
    let label = match label {
        Some(l) => l.to_owned(),
        None if interactive() => match pick(&cfg, "Remove which identity?", None)? {
            Some(i) => cfg.identities[i].label.clone(),
            None => return Ok(()),
        },
        None => return Err("Pass the label of the identity to remove.".into()),
    };
    let removed = cfg
        .remove(&label)
        .ok_or_else(|| format!("No identity labelled {label:?}. See `git identity list`."))?;
    cfg.save()?;
    println!("{} {}", style("Identity removed:").green(), removed.entry());
    Ok(())
}

fn quote_label(label: &str) -> String {
    if label.contains(char::is_whitespace) {
        format!("\"{label}\"")
    } else {
        label.to_owned()
    }
}

fn apply(id: &Identity) -> Result<(), String> {
    let prev_name = git::get("user.name");
    let prev_email = git::get("user.email");
    if prev_name.is_some() || prev_email.is_some() {
        let not_set = || "(not set)".to_owned();
        println!("{}", style("Previous identity:").dim());
        println!(
            "  user.name  = {}",
            style(prev_name.unwrap_or_else(not_set)).dim()
        );
        println!(
            "  user.email = {}",
            style(prev_email.unwrap_or_else(not_set)).dim()
        );
        println!();
    }

    git::set("user.name", &id.name)?;
    git::set("user.email", &id.email)?;

    match id.signing_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => {
            git::set("user.signingkey", key)?;
            git::set("commit.gpgsign", "true")?;
            git::set("gpg.format", gpg_format(key))?;
        }
        None => SIGNING_KEYS.into_iter().for_each(git::unset),
    }

    println!("{}", style("Identity set for this repo:").green());
    println!("  user.name       = {}", style(&id.name).bold());
    println!("  user.email      = {}", style(&id.email).bold());
    if let Some(key) = &id.signing_key {
        println!("  user.signingkey = {}", style(key).bold());
    }
    Ok(())
}

/// `ssh` for SSH keys (literal or `key::` prefixed), `openpgp` otherwise.
fn gpg_format(key: &str) -> &'static str {
    if key.starts_with("ssh-") || key.starts_with("key::") {
        "ssh"
    } else {
        "openpgp"
    }
}

#[cfg(test)]
mod tests {
    use super::gpg_format;

    #[test]
    fn detects_gpg_format() {
        assert_eq!(gpg_format("ssh-ed25519 AAAAC3Nza"), "ssh");
        assert_eq!(gpg_format("key::ssh-ed25519 AAAA"), "ssh");
        assert_eq!(gpg_format("ABCDEF1234567890"), "openpgp");
    }
}
