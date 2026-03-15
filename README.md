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

Inside any git repository:

```bash
git-identity
```

An interactive fzf prompt will appear. Select an identity and it will be applied via `git config --local`.

## Configuration

By default the script looks for the config at:

```
${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json
```

Override it with the `GIT_IDENTITY_CONFIG` environment variable:

```bash
export GIT_IDENTITY_CONFIG="$HOME/my-identities.json"
```

## License

[MIT](LICENSE)
