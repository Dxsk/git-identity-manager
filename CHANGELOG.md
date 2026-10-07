# Changelog

## Unreleased

Rewritten in Rust. The tool is now a single binary that only needs `git`, and runs on Linux, macOS and Windows.

### Added

- `add` and `remove` commands to manage identities without editing the file, with an interactive mode that suggests your current name, email and a remote pattern
- `use <label>`, `auto`, `init`, `path` and `version` commands
- Shortcuts: `ls`, `sw`/`switch`, `cur`/`whoami`, `new`/`append`, `rm`, `h`
- Windows support, with a per-user installer that adds itself to `PATH`
- `install.sh` for Linux and macOS, which verifies the download checksum
- `SHA256SUMS` and `SHA256SUMS-binaries` attached to every release
- Releases built on Forgejo and on the GitHub mirror

### Changed

- The interactive picker is built in, so `fzf` and `jq` are no longer needed
- On Windows the config lives in `%APPDATA%\git-identity\identities.json`
- The reminder hook is now plain POSIX `sh`
- Errors go to stderr

### Removed

- The bash script
- The Nix flake
- release-please (releases are now cut by pushing a tag, see `make release`)


## 1.0.0 (2026-06-15)


### Features

* add automatic gpg.format detection for SSH signing keys ([61fe787](https://github.com/Dxsk/git-identity-manager/commit/61fe7879dbff117ec4f67595d44ad84c087cad9b))
* add CLI options, signing key support, auto-detection, and Makefile ([00062f1](https://github.com/Dxsk/git-identity-manager/commit/00062f15c73090dc0894a2bf49689baf7122ed05))
* add conditional colors, JSON validation, and update README ([17d3e90](https://github.com/Dxsk/git-identity-manager/commit/17d3e9096a3222b0034e9c0fb26e220a1e82531b))
* add git-identity script, flake, and documentation ([ad529b7](https://github.com/Dxsk/git-identity-manager/commit/ad529b7c28eaa8d9ce9939561482d180099b7f14))
