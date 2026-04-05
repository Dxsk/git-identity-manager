#!/usr/bin/env bash
set -euo pipefail

CONFIG_FILE="${GIT_IDENTITY_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/git-identity/identities.json}"

usage() {
  cat <<EOF
Usage: git-identity [OPTION]

Options:
  --list        List all configured identities
  --current     Show the current local identity
  --unset       Remove local identity (falls back to global)
  --hook        Install a post-checkout hook that reminds you to set an identity
  -h, --help    Show this help message

Without options, opens an interactive fzf prompt to select an identity.
EOF
}

require_config() {
  if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "Config not found: $CONFIG_FILE"
    echo "Create it or set GIT_IDENTITY_CONFIG to point to your identities.json"
    exit 1
  fi
}

require_repo() {
  if ! git rev-parse --is-inside-work-tree &>/dev/null; then
    echo "Not inside a git repository."
    exit 1
  fi
}

require_jq() {
  if ! command -v jq &>/dev/null; then
    echo "jq is required. Install it first."
    exit 1
  fi
}

cmd_list() {
  require_config
  require_jq
  jq -r '.identities[] | "\(.label) | \(.name) <\(.email)>" + (if .signingKey then " [signing: \(.signingKey[0:16])...]" else "" end) + (if .remotes then " (remotes: \(.remotes | join(", ")))" else "" end)' "$CONFIG_FILE"
}

cmd_current() {
  require_repo
  local name email signing
  name=$(git config --local user.name 2>/dev/null || true)
  email=$(git config --local user.email 2>/dev/null || true)
  signing=$(git config --local user.signingkey 2>/dev/null || true)

  if [[ -z "$name" && -z "$email" ]]; then
    echo "No local identity set. Using global config."
    return
  fi

  echo "Current local identity:"
  echo "  user.name       = ${name:-(not set)}"
  echo "  user.email      = ${email:-(not set)}"
  [[ -n "$signing" ]] && echo "  user.signingkey = $signing"
}

cmd_unset() {
  require_repo
  git config --local --unset user.name 2>/dev/null || true
  git config --local --unset user.email 2>/dev/null || true
  git config --local --unset user.signingkey 2>/dev/null || true
  git config --local --unset commit.gpgsign 2>/dev/null || true
  echo "Local identity removed. Falling back to global config."
}

cmd_hook() {
  require_repo
  local hook_dir
  hook_dir=$(git rev-parse --git-dir)/hooks
  local hook_file="$hook_dir/post-checkout"

  if [[ -f "$hook_file" ]] && grep -q "git-identity" "$hook_file"; then
    echo "Hook already installed: $hook_file"
    return
  fi

  mkdir -p "$hook_dir"

  if [[ -f "$hook_file" ]]; then
    echo "" >> "$hook_file"
  else
    echo "#!/usr/bin/env bash" > "$hook_file"
    chmod +x "$hook_file"
  fi

  cat >> "$hook_file" << 'HOOK'
# git-identity reminder
if [[ -z "$(git config --local user.name 2>/dev/null)" ]]; then
  echo ""
  echo "[git-identity] No local identity set. Run 'git-identity' to pick one."
fi
HOOK

  echo "Post-checkout hook installed: $hook_file"
}

apply_identity() {
  local label="$1"

  require_jq
  require_config

  local name email signing_key
  name=$(jq -r --arg l "$label" '.identities[] | select(.label == $l) | .name' "$CONFIG_FILE")
  email=$(jq -r --arg l "$label" '.identities[] | select(.label == $l) | .email' "$CONFIG_FILE")
  signing_key=$(jq -r --arg l "$label" '.identities[] | select(.label == $l) | .signingKey // empty' "$CONFIG_FILE")

  # Show previous identity
  local current_name current_email
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

  if [[ -n "$signing_key" ]]; then
    git config --local user.signingkey "$signing_key"
    git config --local commit.gpgsign true
  else
    git config --local --unset user.signingkey 2>/dev/null || true
    git config --local --unset commit.gpgsign 2>/dev/null || true
  fi

  echo "Identity set for this repo:"
  echo "  user.name       = $name"
  echo "  user.email      = $email"
  [[ -n "$signing_key" ]] && echo "  user.signingkey = $signing_key"
}

cmd_select() {
  require_config
  require_jq
  require_repo

  if ! command -v fzf &>/dev/null; then
    echo "fzf is required. Install it first."
    exit 1
  fi

  # Auto-detect: suggest identity based on remote URL
  local suggestion=""
  local remote_url
  remote_url=$(git remote get-url origin 2>/dev/null || true)
  if [[ -n "$remote_url" ]]; then
    suggestion=$(jq -r --arg url "$remote_url" '
      .identities[] | select(.remotes != null) |
      select(.remotes[] as $r | $url | test($r)) |
      .label' "$CONFIG_FILE" | head -1)
  fi

  local prompt="Select git identity: "
  [[ -n "$suggestion" ]] && prompt="Select git identity (suggested: $suggestion): "

  # Build selection list
  local entries
  entries=$(jq -r '.identities[] | "\(.label) | \(.name) <\(.email)>"' "$CONFIG_FILE")

  local selected
  selected=$(echo "$entries" | fzf --prompt="$prompt" --height=~50% --reverse --query="${suggestion}") || exit 0

  local label
  label=$(echo "$selected" | cut -d'|' -f1 | xargs)

  apply_identity "$label"
}

case "${1:-}" in
  --list)    cmd_list ;;
  --current) cmd_current ;;
  --unset)   cmd_unset ;;
  --hook)    cmd_hook ;;
  -h|--help) usage ;;
  "")        cmd_select ;;
  *)         echo "Unknown option: $1"; usage; exit 1 ;;
esac
