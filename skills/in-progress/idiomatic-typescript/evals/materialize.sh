#!/usr/bin/env bash
# Prepare evaluation trees and guidance copies.
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

prepare_acceptance() {
  local scenario_dir=$1 tree=$2 here
  here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
  if [[ -d $scenario_dir/verify/tests ]]; then
    rsync -a "$scenario_dir/verify/tests/" "$tree/tests/" || return
  fi
  if [[ -d $scenario_dir/verify/type-tests ]]; then
    mkdir -p "$tree/type-tests" || return
    rsync -a "$scenario_dir/verify/type-tests/" "$tree/type-tests/" || return
  fi
  # Keep the candidate's module settings, but always check source and acceptance types.
  jq '{extends: "./tsconfig.json", include: .include, exclude: [],
      compilerOptions: {strict: true, noCheck: false, noEmit: true}}' \
    "$here/fixtures/base/tsconfig.json" >"$tree/.acceptance-typecheck.json" || return
  # Acceptance owns its commands; preserve the candidate's package exports and dependencies.
  jq --slurpfile base "$here/fixtures/base/package.json" \
    '.scripts = ($base[0].scripts + {typecheck: "tsc -p .acceptance-typecheck.json"})' \
    "$tree/package.json" >"$tree/.acceptance-package.json" || return
  mv "$tree/.acceptance-package.json" "$tree/package.json"
}

copy_skill() {
  local source=$1 destination=$2
  rsync -a --exclude examples --exclude evals "$source/" "$destination/"
}
