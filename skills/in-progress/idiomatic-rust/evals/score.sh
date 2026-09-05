#!/usr/bin/env bash
# Mechanical scores for every run of one scenario: the crate's own tests, the scenario's external
# semantic tests, the LINTS.md check command in both passes, the size of the diff against start/,
# and pattern counts. Prints Markdown to stdout. Each run also gets a score.json beside its
# meta.json, which `analyze.py matrix` aggregates, and cargo's own output goes to score.log.
#
# A run that does not compile scores BUILD FAILED. A run where cargo itself fails before it
# produces any diagnostic (no toolchain, a broken manifest, a dependency that did not download)
# scores INCOMPLETE. Neither is ever reported as zero findings or zero failures.
#
# usage: score.sh <scenario>
set -euo pipefail

if (($# != 1)); then
  echo "usage: score.sh <scenario>" >&2
  exit 2
fi
scenario=$1
here=$(cd "$(dirname "$0")" && pwd)
scenario_dir="$here/scenarios/$scenario"
if [[ ! -d $scenario_dir ]]; then
  echo "score.sh: no scenario '$scenario' under $here/scenarios" >&2
  exit 2
fi

# shellcheck source=flags.sh
source "$here/flags.sh"
read_check_flags "$here/../LINTS.md"
read_scenario_flags "$scenario_dir"

sum_field() {
  # sum_field <n>: the sum of awk field n over stdin, 0 when stdin is empty
  awk -v n="$1" '{ s += $n } END { print s + 0 }'
}

count() {
  # count <pattern> <dir>: matches in .rs files under dir, 0 when none
  { rg -c --no-messages "$1" "$2" --glob '*.rs' || true; } | awk -F: '{ print $NF }' | sum_field 1
}

tests_passed() {
  # tests_passed <output>: passed tests summed over every `test result:` line
  { rg -o 'test result: ok\. [0-9]+ passed' <<<"$1" || true; } | sum_field 4
}

tests_failed() {
  # tests_failed <output>: failed tests summed over every `test result:` line
  { rg -o 'test result: FAILED\. [0-9]+ passed; [0-9]+ failed' <<<"$1" || true; } | sum_field 6
}

failed_names() {
  # failed_names <output>: the names of the tests that failed, comma separated
  { rg -o '^test \S+ \.\.\. FAILED' <<<"$1" || true; } | awk '{ print $2 }' | paste -sd, -
}

classify_tests() {
  # classify_tests <status> <output>: sets `text` to "pass (n tests)", "FAIL (n failed: names)",
  # "BUILD FAILED", or "INCOMPLETE (cargo exited n)", and `verdict` and `detail` for score.json.
  # The functions in this file set variables rather than print, so no result is lost to a subshell.
  local status=$1 output=$2
  if rg -q 'test result: FAILED' <<<"$output"; then
    verdict=fail
    detail="$(tests_failed "$output") failed: $(failed_names "$output")"
    text="FAIL ($detail)"
  elif ((status == 0)); then
    verdict=pass
    detail="$(tests_passed "$output") tests"
    text="pass ($detail)"
  elif rg -q 'error\[E[0-9]{4}\]|could not compile' <<<"$output"; then
    verdict=build_failed
    detail="see score.log"
    text="BUILD FAILED"
  else
    verdict=incomplete
    detail="cargo exited $status; see score.log"
    text="INCOMPLETE ($detail)"
  fi
}

lints() {
  # lints <tree> <log> [clippy args...]: sets `text` to "N (lint a, lint b, ...)" from the JSON
  # messages, "BUILD FAILED (rustc errors: n)" when rustc reports an error that is not a lint, or
  # "INCOMPLETE (cargo exited n)" when cargo fails without a single diagnostic, and `verdict`.
  # A tree that does not compile, or a scorer that cannot run, then never scores as clean.
  local tree=$1 log=$2
  shift 2
  local messages status=0 errors findings
  # Under -D warnings the exit status is non-zero for a lint finding too, so the messages, not
  # the status alone, tell a build failure from a finding.
  messages=$(cd "$tree" && cargo clippy --no-deps --all-features --message-format=json "$@" \
    2>>"$log") || status=$?
  errors=$(jq -rs '
    [ .[] | select(.reason == "compiler-message") | .message
      | select(.level == "error")
      | select(
          (.code == null and (.message | startswith("aborting due to") | not))
          or (.code != null and (.code.code | test("^E[0-9]{4}$")))
        )
    ] | length' <<<"$messages")
  if ((errors > 0)); then
    verdict=build_failed
    text="BUILD FAILED (rustc errors: $errors)"
    return
  fi
  findings=$(jq -r 'select(.reason == "compiler-message") | .message | select(.code != null) | .code.code' \
    <<<"$messages" | sort | uniq -c | sort -rn)
  if [[ -z $findings ]]; then
    if ((status != 0)); then
      verdict=incomplete
      text="INCOMPLETE (cargo exited $status; see score.log)"
    else
      verdict=clean
      text="0 ()"
    fi
    return
  fi
  verdict=findings
  text=$(awk '
    { n += $1; parts = parts (parts ? ", " : "") $2 " " $1 }
    END { printf "%d (%s)", n, parts }
  ' <<<"$findings")
}

external() {
  # external <tree> <log>: the scenario's verify/tests/*.rs run against a scratch copy of the tree,
  # so the results tree stays as the agent left it. Sets `text`, `verdict`, and `detail`; "none"
  # when the scenario has no external tests.
  local tree=$1 log=$2 scratch output status=0
  local -a tests=("$scenario_dir"/verify/tests/*.rs)
  if [[ ! -f ${tests[0]} ]]; then
    verdict=none
    detail=""
    text="none"
    return
  fi
  scratch=$(mktemp -d)
  rsync -a --exclude target "$tree/" "$scratch/"
  mkdir -p "$scratch/tests"
  cp "${tests[@]}" "$scratch/tests/"
  local -a names=()
  local test
  for test in "${tests[@]}"; do
    test=$(basename "$test" .rs)
    names+=(--test "$test")
  done
  # The tree's target dir is shared, so the dependencies build once per run.
  output=$(cd "$scratch" && CARGO_TARGET_DIR="$tree/target" cargo test "${names[@]}" 2>&1) || status=$?
  printf '%s\n' "$output" >>"$log"
  rm -r "$scratch"
  classify_tests "$status" "$output"
}

diff_stats() {
  # diff_stats <tree>: lines added and removed against start/ over .rs and Cargo.toml files, the
  # dependencies the tree adds, and the source files it adds. Sets `added`, `removed`, `new_deps`,
  # and `new_files`.
  local tree=$1 start="$scenario_dir/start"
  local numstat
  # The path column reads `{start => tree}/src/lib.rs`, so the fields are split on tabs.
  # The scorer's own builds put a target/ in the tree, which is not the agent's diff.
  numstat=$(git diff --no-index --numstat "$start" "$tree" 2>/dev/null || true)
  added=$(awk -F'\t' '$3 ~ /\.(rs|toml)$/ && $3 !~ /\/target\// && $1 != "-" { s += $1 } END { print s + 0 }' <<<"$numstat")
  removed=$(awk -F'\t' '$3 ~ /\.(rs|toml)$/ && $3 !~ /\/target\// && $2 != "-" { s += $2 } END { print s + 0 }' <<<"$numstat")
  new_deps=$(comm -13 <(deps "$start/Cargo.toml") <(deps "$tree/Cargo.toml") | paste -sd, -)
  new_files=$(comm -13 <(sources "$start") <(sources "$tree") | paste -sd, -)
}

deps() {
  # deps <Cargo.toml>: the dependency names in every [dependencies]-like table, sorted
  awk '
    /^\[/ { table = ($0 ~ /dependencies\]$/) }
    table && /^[A-Za-z0-9_-]+ *=/ { sub(/ *=.*/, ""); print }
  ' "$1" 2>/dev/null | sort -u
}

sources() {
  # sources <crate>: the .rs files under src/ and tests/, relative to the crate, sorted
  (cd "$1" && find src tests -name '*.rs' 2>/dev/null | sort)
}

score_run() {
  local arm=$1 run=$2
  local dir="$here/results/$scenario/$arm/$run"
  local tree="$dir/tree"
  local label="$arm/$run"
  if [[ ! -d $tree/src ]]; then
    echo "| $label | no tree | | | | | | |"
    return
  fi
  local log="$dir/score.log"
  : >"$log"
  local text verdict detail
  local tests tests_verdict tests_detail output status=0
  output=$(cd "$tree" && cargo test 2>&1) || status=$?
  printf '%s\n' "$output" >>"$log"
  classify_tests "$status" "$output"
  tests=$text tests_verdict=$verdict tests_detail=$detail
  local ext ext_verdict ext_detail
  external "$tree" "$log"
  ext=$text ext_verdict=$verdict ext_detail=$detail
  local lib lib_verdict all all_verdict
  lints "$tree" "$log" -- "${flags[@]}" "${scenario_flags[@]}"
  lib=$text lib_verdict=$verdict
  lints "$tree" "$log" --all-targets -- "${flags[@]}" "${scenario_flags[@]}" \
    -A clippy::unwrap_used -A clippy::panic_in_result_fn
  all=$text all_verdict=$verdict
  local loc
  loc=$(cat "$tree"/src/*.rs 2>/dev/null | { rg -v '^\s*$' || true; } | wc -l | tr -d ' ')
  local added removed new_deps new_files
  diff_stats "$tree"
  echo "| $label | $tests | $ext | $lib | $all | $loc | +$added/-$removed | ${new_deps:-none} |"
  jq -n \
    --arg scenario "$scenario" --arg arm "$arm" --arg run "$run" \
    --arg tests "$tests_verdict" --arg tests_detail "$tests_detail" \
    --arg external "$ext_verdict" --arg external_detail "$ext_detail" \
    --arg lib "$lib_verdict" --arg lib_text "$lib" \
    --arg all "$all_verdict" --arg all_text "$all" \
    --argjson loc "$loc" --argjson added "$added" --argjson removed "$removed" \
    --arg new_deps "$new_deps" --arg new_files "$new_files" \
    '{
      scenario: $scenario, arm: $arm, run: $run,
      tests: { verdict: $tests, detail: $tests_detail },
      external: { verdict: $external, detail: $external_detail },
      check_lib: { verdict: $lib, text: $lib_text },
      check_all: { verdict: $all, text: $all_text },
      loc: $loc,
      diff: {
        added: $added, removed: $removed,
        new_deps: ($new_deps | split(",") | map(select(. != ""))),
        new_files: ($new_files | split(",") | map(select(. != "")))
      }
    }' >"$dir/score.json"
}

# Every arm and run under results/<scenario>/, in name order.
runs=()
while IFS= read -r dir; do
  runs+=("$dir")
done < <(find "$here/results/$scenario" -mindepth 2 -maxdepth 2 -type d -name 'r*' 2>/dev/null | sort)
if ((${#runs[@]} == 0)); then
  echo "score.sh: no runs under $here/results/$scenario; run.sh writes them" >&2
  exit 1
fi

echo "## $scenario"
echo
echo "| run | cargo test | external tests | check (lib) | check (all targets) | non-blank lines | diff vs start | new deps |"
echo "| --- | --- | --- | --- | --- | --- | --- | --- |"
for dir in "${runs[@]}"; do
  score_run "$(basename "$(dirname "$dir")")" "$(basename "$dir")"
done
echo
printf '| pattern |'
for dir in "${runs[@]}"; do
  printf ' %s/%s |' "$(basename "$(dirname "$dir")")" "$(basename "$dir")"
done
echo
printf '| --- |'
for _ in "${runs[@]}"; do
  printf ' --- |'
done
echo
patterns=(
  'unwrap\(\)'
  '\.expect\('
  '_ =>'
  'Instant::now\(\)|SystemTime::now\(\)'
  '&String'
  '&Vec<'
  'Box<dyn (std::error::)?Error'
  'fn get_'
  '#\[must_use'
  'fn test_'
  '#\[(tokio::)?test'
  '\.clone\(\)'
  'Arc<Mutex'
  'tokio::sync::Mutex'
  'broadcast'
  'CancellationToken'
  'biased;'
  'JoinSet|TaskTracker'
  'println!'
  'thiserror'
  'anyhow'
  'Outcome|Rejected'
  'let .* else'
  '&& let '
  'NonZero'
  'impl Iterator'
  'serde\(try_from'
  'PhantomData'
  'pub type Result'
)
for pattern in "${patterns[@]}"; do
  # shellcheck disable=SC2016 # the backticks are literal Markdown, not a command substitution
  printf '| `%s` |' "$pattern"
  for dir in "${runs[@]}"; do
    printf ' %s |' "$(count "$pattern" "$dir/tree/src")"
  done
  echo
done
