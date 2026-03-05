#!/usr/bin/env bash
set -euo pipefail

REPO="the-waste-land/podcast-cli"
BIN_NAME="podcast-cli"
WORKFLOW_FILE="release.yml"
CARGO_TOML="Cargo.toml"
DIST_DIR="dist"

log() {
  printf '[release] %s\n' "$*"
}

die() {
  printf '[release] ERROR: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat <<USAGE
Usage: ./release.sh <tag>

Example:
  ./release.sh v0.2.0
USAGE
}

require_cmd() {
  local cmd
  for cmd in "$@"; do
    command -v "$cmd" >/dev/null 2>&1 || die "missing required command: $cmd"
  done
}

ensure_clean_git_tree() {
  git diff --quiet || die "working tree has unstaged changes, please commit/stash first"
  git diff --cached --quiet || die "index has staged changes, please commit/stash first"
}

parse_version_from_tag() {
  local tag="$1"
  [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]] || die "invalid tag '$tag' (expected like v0.2.0)"
  printf '%s' "${tag#v}"
}

read_cargo_version() {
  awk '
    BEGIN { in_pkg=0 }
    /^\[package\]$/ { in_pkg=1; next }
    /^\[/ && $0 != "[package]" { in_pkg=0 }
    in_pkg && /^version[[:space:]]*=/ {
      gsub(/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/, "")
      gsub(/"[[:space:]]*$/, "")
      print
      exit
    }
  ' "$CARGO_TOML"
}

update_cargo_version() {
  local new_version="$1"
  local tmp
  tmp="$(mktemp)"

  awk -v ver="$new_version" '
    BEGIN { in_pkg=0; replaced=0 }

    /^\[package\]$/ {
      in_pkg=1
      print
      next
    }

    /^\[/ && $0 != "[package]" {
      in_pkg=0
    }

    {
      if (in_pkg && $0 ~ /^version[[:space:]]*=/ && replaced == 0) {
        sub(/"[^"]+"/, "\"" ver "\"")
        replaced=1
      }
      print
    }

    END {
      if (replaced == 0) {
        print "failed to find [package].version in Cargo.toml" > "/dev/stderr"
        exit 1
      }
    }
  ' "$CARGO_TOML" > "$tmp"

  mv "$tmp" "$CARGO_TOML"
}

check_tag_not_exists() {
  local tag="$1"
  git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "local tag already exists: $tag"
  git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1 && die "remote tag already exists: $tag"
}

wait_for_run_id() {
  local tag="$1"
  local tries=60
  local sleep_seconds=5
  local run_id

  for ((i=1; i<=tries; i++)); do
    run_id="$(gh run list \
      --repo "$REPO" \
      --workflow "$WORKFLOW_FILE" \
      --limit 30 \
      --json databaseId,headBranch,event \
      --jq ".[] | select(.headBranch == \"$tag\" and .event == \"push\") | .databaseId" \
      | head -n1 || true)"

    if [[ -n "$run_id" ]]; then
      printf '%s' "$run_id"
      return 0
    fi

    log "waiting for workflow run to appear ($i/$tries)..."
    sleep "$sleep_seconds"
  done

  return 1
}

wait_for_release() {
  local tag="$1"
  local tries=60
  local sleep_seconds=5

  for ((i=1; i<=tries; i++)); do
    if gh release view "$tag" --repo "$REPO" >/dev/null 2>&1; then
      return 0
    fi
    log "waiting for GitHub Release $tag ($i/$tries)..."
    sleep "$sleep_seconds"
  done

  return 1
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
        *) die "unsupported Linux arch for auto-install: $arch" ;;
      esac
      ;;
    *)
      die "auto-install only supports Linux x86_64 for this repo workflow; current OS: $os"
      ;;
  esac
}

install_binary_from_dist() {
  local tag="$1"
  local target asset tmp_dir

  target="$(detect_target)"
  asset="$DIST_DIR/${BIN_NAME}-${tag}-${target}.tar.gz"
  [[ -f "$asset" ]] || die "expected asset not found: $asset"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  tar -xzf "$asset" -C "$tmp_dir"
  [[ -x "$tmp_dir/$BIN_NAME" ]] || die "binary not found in archive: $asset"

  mkdir -p "$HOME/.cargo/bin"
  install -m 0755 "$tmp_dir/$BIN_NAME" "$HOME/.cargo/bin/$BIN_NAME"
  log "installed $BIN_NAME to $HOME/.cargo/bin/$BIN_NAME"
}

main() {
  local tag version current_version run_id

  [[ $# -eq 1 ]] || { usage; exit 1; }
  tag="$1"

  require_cmd git gh awk tar install mktemp uname

  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "must run inside a git repository"
  cd "$(git rev-parse --show-toplevel)"

  [[ -f "$CARGO_TOML" ]] || die "$CARGO_TOML not found"
  gh auth status >/dev/null 2>&1 || die "gh is not authenticated; run: gh auth login"

  ensure_clean_git_tree
  check_tag_not_exists "$tag"

  version="$(parse_version_from_tag "$tag")"
  current_version="$(read_cargo_version)"
  [[ -n "$current_version" ]] || die "failed to read current version from $CARGO_TOML"

  if [[ "$current_version" == "$version" ]]; then
    die "$CARGO_TOML already has version $version; refusing to create empty release commit"
  fi

  log "updating version: $current_version -> $version"
  update_cargo_version "$version"

  git add "$CARGO_TOML"
  git commit -m "chore(release): $version"

  git tag -a "$tag" -m "Release $tag"
  git push origin "$tag"
  log "pushed tag $tag"

  run_id="$(wait_for_run_id "$tag")" || die "failed to find workflow run for tag $tag"
  log "watching workflow run #$run_id"
  gh run watch "$run_id" --repo "$REPO" --interval 10 --exit-status

  wait_for_release "$tag" || die "GitHub Release $tag not found after workflow finished"

  mkdir -p "$DIST_DIR"
  gh release download "$tag" --repo "$REPO" --dir "$DIST_DIR" --clobber
  log "downloaded release assets to $DIST_DIR/"

  install_binary_from_dist "$tag"
  log "release flow completed for $tag"
}

main "$@"
