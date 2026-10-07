<div align="center">

# git-identity

**Pick the right Git identity for each repository, and stop committing with the wrong email.**

[![Forgejo CI](https://forge.infrasouveraine.fr/dxsk/git-identity-manager/badges/workflows/ci.yml/badge.svg?label=Forgejo%20CI)](https://forge.infrasouveraine.fr/dxsk/git-identity-manager/actions?workflow=ci.yml)
[![GitHub CI](https://img.shields.io/github/actions/workflow/status/Dxsk/git-identity-manager/ci.yml?branch=main&label=GitHub%20CI&logo=github)](https://github.com/Dxsk/git-identity-manager/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Dxsk/git-identity-manager?logo=git&logoColor=white)](https://github.com/Dxsk/git-identity-manager/releases/latest)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-informational)](#installation)
[![License](https://img.shields.io/github/license/Dxsk/git-identity-manager)](LICENSE)

[Forgejo (main repository)](https://forge.infrasouveraine.fr/dxsk/git-identity-manager) · [GitHub mirror](https://github.com/Dxsk/git-identity-manager)

</div>

---

If you have a personal account, a work account and maybe a few open-source ones, sooner or later you push a commit signed with the wrong address. `git-identity` keeps your identities in one file and sets the right `user.name`, `user.email` and signing key on each repository, either from a quick fuzzy picker or automatically from the remote URL.

It is a single small binary. The only thing it needs at runtime is `git`.

```console
$ git identity
? Select git identity (suggested: Work) ›
❯ Work | Jane Doe <jane@company.com>
  Personal | Jane Doe <jane@example.com>

Identity set for this repo:
  user.name       = Jane Doe
  user.email      = jane@company.com
```

## Installation

Prebuilt binaries for every release are on the [GitHub releases page](https://github.com/Dxsk/git-identity-manager/releases), which has every platform. The [Forgejo releases](https://forge.infrasouveraine.fr/dxsk/git-identity-manager/releases) carry the Linux and Windows x86_64 builds.

<details open>
<summary><b>Windows</b></summary>

<br>

Download and run `git-identity-x86_64-setup.exe` (or the `aarch64` one on ARM devices).

The installer works per user, so it does not ask for admin rights. It adds the program to your `PATH` and shows up in *Apps & features* if you want to remove it later.

For a silent install, pass `/S`:

```powershell
.\git-identity-x86_64-setup.exe /S
```

If you would rather not install anything, grab the portable `.zip` and put `git-identity.exe` somewhere on your `PATH`.

</details>

<details open>
<summary><b>Linux and macOS</b></summary>

<br>

```sh
curl -fsSL https://raw.githubusercontent.com/Dxsk/git-identity-manager/main/install.sh | sh
```

The script downloads the archive that matches your system, checks it against the release's `SHA256SUMS`, and puts the binary in `~/.local/bin`. You can change that with `PREFIX`, and pin a release with `VERSION`:

```sh
curl -fsSL https://raw.githubusercontent.com/Dxsk/git-identity-manager/main/install.sh | PREFIX=/usr/local VERSION=v1.1.0 sh
```

</details>

<details>
<summary><b>From source</b></summary>

<br>

You need a [Rust toolchain](https://rustup.rs).

```sh
make install                    # builds and installs to ~/.local/bin/git-identity
make install PREFIX=/usr/local  # or anywhere else
make uninstall
```

Cargo works too:

```sh
cargo install --git https://github.com/Dxsk/git-identity-manager
```

</details>

<details>
<summary><b>Verifying a download</b></summary>

<br>

Every release ships two checksum files:

- `SHA256SUMS` lists the archives and installers you download.
- `SHA256SUMS-binaries` lists the `git-identity` binary inside each archive, as `<target>/git-identity`. Use it to check a binary you already extracted or installed.

On Linux and macOS:

```sh
sha256sum --ignore-missing -c SHA256SUMS   # macOS: shasum -a 256 --ignore-missing -c SHA256SUMS
```

On Windows:

```powershell
(Get-FileHash .\git-identity-x86_64-setup.exe).Hash.ToLower()
(Get-FileHash "$env:LOCALAPPDATA\Programs\git-identity\git-identity.exe").Hash.ToLower()
```

Then compare the result with the matching line in the checksum file.

</details>

## Getting started

Add your first identity. Run it inside a repository and it suggests your current name, email and a remote pattern based on `origin`:

```sh
git identity add
```

Or do it in one line, for scripts and dotfiles:

```sh
git identity add Work --name "Jane Doe" --email jane@company.com --remote 'github\.com[:/]my-company/'
```

From then on, run `git identity` inside any repository to choose who you are there. If you prefer editing the file by hand, `git identity init` writes an example config and `git identity path` tells you where it is.

## Usage

The binary is called `git-identity`, so Git picks it up as a subcommand. `git identity` and `git-identity` do the same thing.

| Command | Shortcuts | What it does |
|---|---|---|
| `git identity` | | Opens the fuzzy picker |
| `git identity use <label>` | `sw`, `switch` | Applies an identity by its label (case does not matter) |
| `git identity auto` | | Applies the identity whose `remotes` match `origin` |
| `git identity list` | `ls` | Lists your identities |
| `git identity current` | `cur`, `whoami` | Shows the identity set on this repository |
| `git identity unset` | | Removes it, so the global config applies again |
| `git identity add [<label>]` | `new`, `append` | Adds an identity (see below) |
| `git identity remove [<label>]` | `rm` | Removes an identity, or lets you pick one |
| `git identity hook` | | Installs a reminder hook (see below) |
| `git identity init` | | Creates the config file from a template |
| `git identity path` | | Prints the config file location |
| `git identity help` | `h` | Shows the built-in help |

The old `--list`, `--current`, `--unset` and `--hook` flags still work, so existing aliases and scripts keep running.

<details>
<summary><b>Options for <code>add</code></b></summary>

<br>

| Option | |
|---|---|
| `-n`, `--name <name>` | Value for `user.name` |
| `-e`, `--email <email>` | Value for `user.email` |
| `-k`, `--signing-key <key>` | Signing key |
| `-r`, `--remote <regex>` | Remote pattern, can be given several times |

Run `add` with no options and it walks you through every field. If you pass some options, it only asks for what is still missing. Outside a terminal it never asks, and fails if the label, name or email is missing.

`add` and `remove` rewrite the config file but keep any extra fields you added by hand.

</details>

> [!NOTE]
> Use `git identity help` (or `git-identity --help`) to see the help. Git catches `git identity --help` before it reaches the tool and looks for a manual page instead, which does not exist.

## Configuration

<details open>
<summary><b>Config file format</b></summary>

<br>

```json
{
  "identities": [
    {
      "label": "Work",
      "name": "Jane Doe",
      "email": "jane@company.com",
      "signingKey": "ssh-ed25519 AAAA...",
      "remotes": ["github\\.com[:/]my-company/"]
    },
    {
      "label": "Personal",
      "name": "Jane Doe",
      "email": "jane@example.com"
    }
  ]
}
```

| Field | Required | Description |
|---|:---:|---|
| `label` | ✓ | The name you see in the picker and pass to `use` |
| `name` | ✓ | Value for `user.name` |
| `email` | ✓ | Value for `user.email` |
| `signingKey` | | GPG or SSH key used to sign commits. Setting it also turns on `commit.gpgsign` and picks `gpg.format` for you: `ssh` when the key starts with `ssh-` or `key::`, `openpgp` otherwise. |
| `remotes` | | Regular expressions checked against the `origin` URL. The first identity that matches is suggested in the picker and used by `auto`. |

</details>

<details>
<summary><b>Where the config lives</b></summary>

<br>

`git-identity` looks for the file in this order and uses the first one that applies:

1. The path in `GIT_IDENTITY_CONFIG`
2. `$XDG_CONFIG_HOME/git-identity/identities.json`, if `XDG_CONFIG_HOME` is set
3. The default location for your system:
   - Windows: `%APPDATA%\git-identity\identities.json`
   - Linux and macOS: `~/.config/git-identity/identities.json`

</details>

<details>
<summary><b>Picking the identity automatically</b></summary>

<br>

When an identity has `remotes` patterns and one of them matches the repository's `origin` URL, the picker opens with that identity already selected. You only have to press Enter.

To skip the picker altogether, for example in a script or a Git hook, use:

```sh
git identity auto
```

It fails with a clear message when no pattern matches, so it is safe to chain.

</details>

<details>
<summary><b>Reminder hook</b></summary>

<br>

Inside a repository, `git identity hook` adds a small `post-checkout` hook. After each checkout, it prints a reminder if no local identity is set yet.

The hook is a plain POSIX `sh` script. Git for Windows runs these out of the box, so it behaves the same on every platform. If you already have a `post-checkout` hook, the reminder is appended to it instead of replacing it.

</details>

## Development

<details>
<summary><b>Building and testing</b></summary>

<br>

```sh
make build  # optimized binary in target/release/
make test   # unit and integration tests
make lint   # rustfmt and clippy
```

The integration tests run the real binary against throwaway repositories, with your global Git config kept out of the way.

To build the Windows installer yourself, install [NSIS](https://nsis.sourceforge.io) and run:

```powershell
cargo build --release
makensis /DVERSION=1.1.0 installer\git-identity.nsi
```

The setup file ends up in `dist\`.

</details>

<details>
<summary><b>CI and releases</b></summary>

<br>

The project lives on [Forgejo](https://forge.infrasouveraine.fr/dxsk/git-identity-manager) and is push-mirrored to [GitHub](https://github.com/Dxsk/git-identity-manager). Each side has its own pipeline:

| | Forgejo (`.forgejo/workflows/`) | GitHub mirror (`.github/workflows/`) |
|---|---|---|
| On every push | Format, lint and tests on Linux | Format and lint, tests on Linux, macOS and Windows |
| On a `v*` tag | Linux x86_64 and aarch64, Windows x86_64 and its installer, all cross-compiled from one Linux runner | The same, plus macOS (Intel and Apple Silicon) and Windows ARM |

Both releases include `SHA256SUMS` and `SHA256SUMS-binaries`.

</details>

<details>
<summary><b>Cutting a release</b></summary>

<br>

1. Describe the changes under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md).
2. Run the release target with the new version:

   ```sh
   make release V=1.1.0
   ```

   It sets the version in `Cargo.toml` and `Cargo.lock`, turns the `Unreleased` heading into `1.1.0 (date)`, then commits and creates the `v1.1.0` tag. Nothing is pushed yet, so you can still check the result.
3. Push to Forgejo:

   ```sh
   git push --follow-tags
   ```

The tag starts the Forgejo release, and the mirror carries it to GitHub, which starts the GitHub one. Both refuse to run if the tag does not match the version in `Cargo.toml`.

Since the mirror overwrites GitHub on every sync, never commit or tag directly on GitHub.

</details>

## License

[MIT](LICENSE)
