# git-identity-manager

A simple CLI tool to switch between Git identities (`user.name` / `user.email`) per repository using [fzf](https://github.com/junegunn/fzf).

## Why?

If you use multiple Git accounts (personal, work, open-source), it's easy to commit with the wrong identity. This tool lets you pick the right one interactively and applies it to the current repo's local config.

## Installation

### Nix (flake)

Add the input to your flake:

```nix
{
  inputs.git-identity-manager.url = "github:Dxsk/git-identity-manager";
}
```

Then add it to your packages (e.g. in home-manager):

```nix
home.packages = [ inputs.git-identity-manager.packages.${system}.default ];
```

Or try it directly:

```bash
nix run github:Dxsk/git-identity-manager
```

> Dependencies (`jq`, `fzf`, `git`) are bundled automatically by Nix.

### Make

```bash
make install            # installs to ~/.local/bin/git-identity
make install PREFIX=/usr/local  # or specify a custom prefix
make uninstall
```

### Stow

Clone the repo into your dotfiles directory and stow it:

```bash
cd ~/dotfiles
git clone git@github.com:Dxsk/git-identity-manager.git
stow git-identity-manager -t ~/.local/bin --ignore='README.*|LICENSE|flake.*|identities.*|\.git'
```

### Manual

Dependencies: [jq](https://jqlang.github.io/jq/), [fzf](https://github.com/junegunn/fzf)

```bash
# symlink to a directory in your PATH
ln -s "$(pwd)/git-identity.sh" ~/.local/bin/git-identity

# or alias in your shell rc
echo 'alias git-identity="/path/to/git-identity.sh"' >> ~/.zshrc
```

## Setup

Create your identities config file:

```bash
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/git-identity"
cp identities.example.json "${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json"
$EDITOR "${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json"
```

## Usage

```bash
git-identity              # Interactive identity picker
git-identity --list       # List all configured identities
git-identity --current    # Show current local identity
git-identity --unset      # Remove local identity (falls back to global)
git-identity --hook       # Install a post-checkout reminder hook
git-identity --help       # Show help
```

## Configuration

By default the script looks for the config at:

```
${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json
```

Override it with the `GIT_IDENTITY_CONFIG` environment variable:

```bash
export GIT_IDENTITY_CONFIG="$HOME/my-identities.json"
```

### Identity fields

| Field | Required | Description |
|---|---|---|
| `label` | yes | Display name for the identity |
| `name` | yes | Git `user.name` |
| `email` | yes | Git `user.email` |
| `signingKey` | no | GPG/SSH signing key — sets `user.signingkey`, enables `commit.gpgsign`, and auto-detects `gpg.format` (`ssh` for keys starting with `ssh-` or `key::`, `openpgp` otherwise) |
| `remotes` | no | Regex patterns to match remote URLs — used for auto-suggestion |

### Auto-detection

If an identity has `remotes` patterns, the tool matches them against the current repo's `origin` URL and pre-fills the fzf prompt with the suggested identity.

Example config:

```json
{
  "identities": [
    {
      "label": "Work",
      "name": "Your Name",
      "email": "you@company.com",
      "signingKey": "key_id",
      "remotes": ["github\\.com[:/]your-org/"]
    }
  ]
}
```

### Post-checkout hook

Run `git-identity --hook` inside a repo to install a `post-checkout` hook that reminds you to set an identity when none is configured locally.

## License

[MIT](LICENSE)
