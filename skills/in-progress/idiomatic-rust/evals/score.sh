#!/usr/bin/env bash
# Mechanical scores for both arms of one scenario: tests, the LINTS.md check
# command, and pattern counts. Prints Markdown to stdout.
#
# usage: score.sh <scenario>
set -euo pipefail

scenario=$1
here=$(cd "$(dirname "$0")" && pwd)

flags=(
  -D warnings
  -W clippy::pedantic -A clippy::similar_names -A clippy::must_use_candidate -A clippy::inline_always
  -D clippy::unwrap_used -D clippy::panic_in_result_fn
  -D clippy::unimplemented -W clippy::todo
  -D clippy::dbg_macro -D clippy::print_stdout -D clippy::print_stderr -D clippy::exit
  -D clippy::undocumented_unsafe_blocks -D clippy::allow_attributes_without_reason
  -D clippy::await_holding_lock -D clippy::large_futures
  -W unreachable_pub -W missing_debug_implementations -W unsafe_op_in_unsafe_fn
)

count() {
  # count <pattern> <dir>: matches in .rs files, 0 when none
  { rg -c --no-messages "$1" "$2" --glob '*.rs' 2>/dev/null || true; } | awk -F: '{s+=$NF} END {print s+0}'
}

lints() {
  # lints <tree> [extra clippy args...]: "N total (lint a, lint b, ...)" from the JSON messages
  local tree=$1
  shift
  (cd "$tree" && cargo clippy --no-deps --all-features --message-format=json "$@" 2>/dev/null || true) |
    jq -r 'select(.reason=="compiler-message") | .message | select(.code != null) | .code.code' 2>/dev/null |
    sort | uniq -c | sort -rn |
    awk '{n+=$1; parts=parts (parts?", ":"") $2 " " $1} END {printf "%d (%s)", n, parts}'
}

score_arm() {
  local arm=$1
  local tree="$here/results/$scenario/$arm/tree"
  if [[ ! -d $tree/src ]]; then
    echo "| $arm | no tree | | | |"
    return
  fi
  local tests clippy_lib clippy_tests loc
  if (cd "$tree" && cargo test --quiet >/tmp/score-test.log 2>&1); then
    tests="pass ($({ rg -o 'test result: ok\. [0-9]+ passed' /tmp/score-test.log || true; } | awk '{s+=$4} END {print s+0}') tests)"
  else
    tests="FAIL"
  fi
  clippy_lib=$(lints "$tree" -- "${flags[@]}")
  clippy_tests=$(lints "$tree" --all-targets -- "${flags[@]}" -A clippy::unwrap_used -A clippy::panic_in_result_fn)
  loc=$(cat "$tree"/src/*.rs 2>/dev/null | { rg -v '^\s*$' || true; } | wc -l | tr -d ' ')
  echo "| $arm | $tests | $clippy_lib | $clippy_tests | $loc |"
}

echo "## $scenario"
echo
echo "| arm | cargo test | check findings (lib) | check findings (all targets) | non-blank lines |"
echo "| --- | --- | --- | --- | --- |"
score_arm bare
score_arm skill
echo
echo "| pattern | bare | skill |"
echo "| --- | --- | --- |"
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
)
for pattern in "${patterns[@]}"; do
  b=$(count "$pattern" "$here/results/$scenario/bare/tree/src")
  s=$(count "$pattern" "$here/results/$scenario/skill/tree/src")
  # shellcheck disable=SC2016 # the backticks are literal Markdown, not a command substitution
  printf '| `%s` | %s | %s |\n' "$pattern" "$b" "$s"
done
