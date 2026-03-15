#!/usr/bin/env bash
set -euo pipefail

CONFIG_FILE="${GIT_IDENTITY_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json}"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "Config not found: $CONFIG_FILE"
  echo "Create it or set GIT_IDENTITY_CONFIG to point to your identities.json"
  exit 1
fi

if ! command -v jq &>/dev/null; then
  echo "jq is required. Install it first."
  exit 1
fi

if ! command -v fzf &>/dev/null; then
  echo "fzf is required. Install it first."
  exit 1
fi

if ! git rev-parse --is-inside-work-tree &>/dev/null; then
  echo "Not inside a git repository."
  exit 1
fi

# Build selection list: "label | name <email>"
entries=$(jq -r '.identities[] | "\(.label) | \(.name) <\(.email)>"' "$CONFIG_FILE")

selected=$(echo "$entries" | fzf --prompt="Select git identity: " --height=~50% --reverse) || exit 0

# Parse name and email directly from the selected line
# Format: "label | name <email>"
name_email="${selected#*| }"
name="${name_email% <*}"
email="${name_email#*<}"
email="${email%>}"

# Show current identity if set
current_name=$(git config --local user.name 2>/dev/null || true)
current_email=$(git config --local user.email 2>/dev/null || true)
if [[ -n "$current_name" || -n "$current_email" ]]; then
  echo "Previous identity:"
  echo "  user.name  = ${current_name:-(not set)}"
  echo "  user.email = ${current_email:-(not set)}"
  echo ""
fi

git config --local user.name "$name"
git config --local user.email "$email"

echo "Identity set for this repo:"
echo "  user.name  = $name"
echo "  user.email = $email"
