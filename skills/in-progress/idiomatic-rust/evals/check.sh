#!/usr/bin/env bash
# The checks that need no model run. Everything here must hold before a paid run means anything.
#
# 1. examples/ compiles, passes its tests, passes the LINTS.md check in both passes, and is
#    rustfmt-clean. Every Rust block in SKILL.md, INVARIANTS.md, RUNTIME.md, and CRATES.md is a
#    verbatim excerpt of a module there.
# 2. Every scenario's start/ passes its own tests and the check in both passes, so a run's
#    findings are the agent's and not the fixture's. A scenario that plants lint-caught patterns
#    on purpose says so in an expect-start-findings file, and its start/ skips the check.
# 3. For every scenario with verify/tests/: the external tests do not pass on start/ (the planted
#    defect, or the API the prompt asks for, is missing) and pass on reference/.
# 4. score.sh gives the verdicts test_score.py pins, against a stub cargo.
#
# usage: check.sh [scenario...]   (every scenario by default)
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
# shellcheck source=flags.sh
source "$here/flags.sh"
read_check_flags "$skill_dir/LINTS.md"

failures=0
fail() {
  echo "FAIL: $*" >&2
  failures=$((failures + 1))
}

check_crate() {
  # check_crate <crate> <lint|no-lint> [extra clippy args...]: tests, fmt, and (with `lint`) both
  # check passes
  local crate=$1 lint=$2
  shift 2
  local name
  name=${crate#"$skill_dir"/}
  if ! (cd "$crate" && cargo test --quiet >/dev/null 2>&1); then
    fail "$name: cargo test"
  fi
  if [[ $lint == lint ]]; then
    if ! (cd "$crate" && cargo clippy --no-deps --all-features -- "${flags[@]}" "$@" >/dev/null 2>&1); then
      fail "$name: the check command (lib)"
    fi
    if ! (cd "$crate" && cargo clippy --no-deps --all-targets --all-features -- "${flags[@]}" "$@" \
      -A clippy::unwrap_used -A clippy::panic_in_result_fn >/dev/null 2>&1); then
      fail "$name: the check command (all targets)"
    fi
  fi
  if rustup run nightly rustfmt --version >/dev/null 2>&1; then
    if ! (cd "$crate" && cargo +nightly fmt --check >/dev/null 2>&1); then
      fail "$name: cargo +nightly fmt --check"
    fi
  else
    echo "note: no nightly toolchain, fmt --check skipped for $name"
  fi
}

external_passes() {
  # external_passes <crate> <scenario-dir>: 0 when the scenario's external tests pass on a scratch
  # copy of the crate, 1 otherwise (a compile failure counts as not passing)
  local crate=$1 scenario_dir=$2 scratch
  local -a tests=("$scenario_dir"/verify/tests/*.rs) names=()
  scratch=$(mktemp -d)
  rsync -a --exclude target "$crate/" "$scratch/"
  mkdir -p "$scratch/tests"
  cp "${tests[@]}" "$scratch/tests/"
  local test
  for test in "${tests[@]}"; do
    names+=(--test "$(basename "$test" .rs)")
  done
  local status=0
  (cd "$scratch" && CARGO_TARGET_DIR="$crate/target" cargo test --quiet "${names[@]}" >/dev/null 2>&1) || status=$?
  rm -r "$scratch"
  return "$status"
}

echo "== examples/"
check_crate "$skill_dir/examples" lint
python3 "$here/snippets.py" "$skill_dir/examples/src" \
  "$skill_dir/SKILL.md" "$skill_dir/INVARIANTS.md" "$skill_dir/RUNTIME.md" "$skill_dir/CRATES.md" ||
  fail "a Rust block in the guidance files is not an excerpt of examples/"

echo "== score.sh"
if output=$(python3 "$here/test_score.py" 2>&1); then
  echo "test_score.py passes against the stub cargo"
else
  echo "$output" >&2
  fail "test_score.py"
fi

if (($# > 0)); then
  scenarios=("$@")
else
  scenarios=()
  while IFS= read -r dir; do
    scenarios+=("$(basename "$dir")")
  done < <(find "$here/scenarios" -mindepth 1 -maxdepth 1 -type d | sort)
fi

for scenario in "${scenarios[@]}"; do
  scenario_dir="$here/scenarios/$scenario"
  read_scenario_flags "$scenario_dir"
  echo "== $scenario"
  # A start/ that is a symlink to another scenario's start/ was checked under that scenario.
  if [[ -L $scenario_dir/start ]]; then
    echo "start/ is a link to $(readlink "$scenario_dir/start"); skipped"
    continue
  fi
  if [[ -f $scenario_dir/expect-start-findings ]]; then
    echo "start/ skips the check: $(<"$scenario_dir/expect-start-findings")"
    check_crate "$scenario_dir/start" no-lint
  else
    check_crate "$scenario_dir/start" lint "${scenario_flags[@]}"
  fi
  if [[ -d $scenario_dir/reference ]]; then
    check_crate "$scenario_dir/reference" lint "${scenario_flags[@]}"
  fi
  if compgen -G "$scenario_dir/verify/tests/*.rs" >/dev/null; then
    if external_passes "$scenario_dir/start" "$scenario_dir"; then
      fail "$scenario: the external tests pass on start/, so they catch nothing"
    else
      echo "external tests do not pass on start/ (expected)"
    fi
    if [[ -d $scenario_dir/reference ]]; then
      if external_passes "$scenario_dir/reference" "$scenario_dir"; then
        echo "external tests pass on reference/"
      else
        fail "$scenario: the external tests do not pass on reference/"
      fi
    else
      fail "$scenario: verify/tests/ without a reference/ to prove the tests can pass"
    fi
  fi
done

if ((failures > 0)); then
  echo "check.sh: $failures failure(s)" >&2
  exit 1
fi
echo "check.sh: all checks pass"
