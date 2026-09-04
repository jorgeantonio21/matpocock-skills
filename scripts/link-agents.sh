#!/usr/bin/env bash
set -euo pipefail

# Dev-only, for maintainers of this fork. Links every agent definition in
# agents/ into the local Claude Code agent directory (~/.claude/agents). Each
# entry is a symlink into this repo, so a `git pull` keeps installed agents
# current. Re-run after adding, removing, or renaming an agent.
#
# Sibling of link-skills.sh, which stays byte-identical to upstream.

REPO="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$HOME/.claude/agents"

# If $DEST is a symlink that resolves into this repo, the per-agent symlinks
# would land back inside the working copy. Bail out instead.
if [ -L "$DEST" ]; then
  resolved="$(readlink -f "$DEST")"
  case "$resolved" in
  "$REPO" | "$REPO"/*)
    echo "error: $DEST is a symlink into this repo ($resolved)." >&2
    echo "Remove it and re-run; the script will recreate it as a real dir." >&2
    exit 1
    ;;
  esac
fi

mkdir -p "$DEST"

for src in "$REPO"/agents/*.md; do
  name="$(basename "$src")"
  target="$DEST/$name"

  if [ -e "$target" ] && [ ! -L "$target" ]; then
    echo "error: $target exists and is not a symlink; move it aside and re-run." >&2
    exit 1
  fi

  ln -sfn "$src" "$target"
  echo "linked $name -> $src"
done
