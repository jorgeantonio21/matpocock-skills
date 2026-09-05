#!/usr/bin/env bash
# Overlay a scenario tree on the pinned base package.
set -euo pipefail

materialize() {
  local source=$1 destination=$2
  local here
  here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
  rm -rf "$destination"
  mkdir -p "$destination"
  rsync -a "$here/fixtures/base/" "$destination/"
  rsync -a "$source/" "$destination/"
}
