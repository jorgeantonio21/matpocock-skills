#!/usr/bin/env bash
# Validate fixtures, examples, hidden acceptance tests, and scorer behavior without a model run.
# usage: ./check.sh [scenario...]
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
# shellcheck source=materialize.sh
source "$here/materialize.sh"
failures=0

expected_node=$(<"$here/fixtures/base/.node-version")
actual_node=$(node -p 'process.versions.node')
if [[ $actual_node != "$expected_node" ]]; then
  echo "FAIL: Node $expected_node is required, found $actual_node" >&2
  exit 1
fi

fail() {
  echo "FAIL: $*" >&2
  failures=$((failures + 1))
}

check_tree() {
  local source=$1 label=$2 tree
  tree=$(mktemp -d)
  materialize "$source" "$tree"
  if ! (cd "$tree" && npm ci --ignore-scripts >/dev/null 2>&1 && npm test >/dev/null 2>&1); then
    fail "$label"
  fi
  rm -rf "$tree"
}

external_passes() {
  local source=$1 scenario_dir=$2 tree
  tree=$(mktemp -d)
  materialize "$source" "$tree"
  local status=0
  (prepare_acceptance "$scenario_dir" "$tree" && cd "$tree" \
    && npm ci --ignore-scripts && npm test) >/dev/null 2>&1 || status=$?
  rm -rf "$tree"
  return "$status"
}

echo "== examples"
if ! (cd "$skill_dir/examples" && npm ci --ignore-scripts >/dev/null && npm run check); then
  fail "examples"
fi

echo "== harness regressions"
if output=$(python3 -m unittest discover -s "$here" -p 'test_*.py' 2>&1); then
  echo "$output"
else
  echo "$output" >&2
  fail "harness regression tests"
fi

if (($# > 0)); then
  scenarios=("$@")
else
  scenarios=()
  while IFS= read -r path; do scenarios+=("$(basename "$path")"); done \
    < <(find "$here/scenarios" -mindepth 1 -maxdepth 1 -type d | sort)
fi

for scenario in "${scenarios[@]}"; do
  scenario_dir="$here/scenarios/$scenario"
  echo "== $scenario"
  check_tree "$scenario_dir/start" "$scenario start"
  if [[ -d $scenario_dir/reference ]]; then
    check_tree "$scenario_dir/reference" "$scenario reference"
  fi
  if [[ -d $scenario_dir/verify ]]; then
    if external_passes "$scenario_dir/start" "$scenario_dir"; then
      fail "$scenario acceptance tests pass on start"
    else
      echo "acceptance tests reject start (expected)"
    fi
    if [[ -d $scenario_dir/reference ]]; then
      if external_passes "$scenario_dir/reference" "$scenario_dir"; then
        echo "acceptance tests pass on reference"
      else
        fail "$scenario acceptance tests reject reference"
      fi
    else
      fail "$scenario has acceptance tests without a reference"
    fi
  fi
done

if ((failures > 0)); then
  echo "check.sh: $failures failure(s)" >&2
  exit 1
fi
echo "check.sh: all checks pass"
