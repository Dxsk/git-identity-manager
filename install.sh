#!/bin/sh
# Install the latest prebuilt git-identity binary (Linux / macOS).
#
#   curl -fsSL https://raw.githubusercontent.com/Dxsk/git-identity-manager/main/install.sh | sh
#
# Environment:
#   PREFIX   install prefix (default: ~/.local, binary goes to $PREFIX/bin)
#   VERSION  release tag to install, e.g. v1.1.0 (default: latest)
set -eu

REPO="Dxsk/git-identity-manager"
PREFIX="${PREFIX:-$HOME/.local}"
VERSION="${VERSION:-latest}"

err() { echo "error: $*" >&2; exit 1; }

case "$(uname -s)" in
  Linux)  os=unknown-linux-musl ;;
  Darwin) os=apple-darwin ;;
  *) err "unsupported OS $(uname -s); on Windows use the setup .exe from the releases page" ;;
esac

case "$(uname -m)" in
  x86_64|amd64)  arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *) err "unsupported architecture $(uname -m)" ;;
esac

asset="git-identity-$arch-$os.tar.gz"
if [ "$VERSION" = latest ]; then
  base="https://github.com/$REPO/releases/latest/download"
else
  base="https://github.com/$REPO/releases/download/$VERSION"
fi

download() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$2" "$1"
  else
    err "curl or wget is required"
  fi
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "Downloading $base/$asset"
download "$base/$asset" "$tmp/$asset"
download "$base/SHA256SUMS" "$tmp/SHA256SUMS"

# The file name may carry a leading `*` (binary mode marker) depending on the tool.
expected=$(awk -v f="$asset" '{ sub(/^\*/, "", $2) } $2 == f { print $1 }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || err "$asset is not listed in SHA256SUMS"
actual=$(sha256 "$tmp/$asset")
[ "$expected" = "$actual" ] || err "checksum mismatch for $asset (expected $expected, got $actual)"
echo "Checksum verified"

tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$PREFIX/bin"
install -m755 "$tmp/git-identity" "$PREFIX/bin/git-identity"

echo "Installed $PREFIX/bin/git-identity"
case ":$PATH:" in
  *":$PREFIX/bin:"*) ;;
  *) echo "note: $PREFIX/bin is not in your PATH" ;;
esac
echo "Run 'git identity add' to set up your first identity."
