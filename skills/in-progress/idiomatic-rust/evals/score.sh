#!/usr/bin/env bash
# Mechanical scores for both arms of one scenario: tests, the LINTS.md check
# command, and pattern counts. Prints Markdown to stdout. cargo's own output
# goes to results/<scenario>/<arm>/score.log.
#
# usage: score.sh <scenario>
set -euo pipefail

if (($# != 1)); then
  echo "usage: score.sh <scenario>" >&2
  exit 2
fi
scenario=$1
here=$(cd "$(dirname "$0")" && pwd)
lints_md="$here/../LINTS.md"

# The lint set is written once, in LINTS.md. Take the `flags=( ... )` array
# from the check-command block there, so this script cannot drift from it.
flags_block=$(awk '
  /^## The check command/ { section = 1 }
  section && /^flags=\(/ { block = 1 }
  block { print }
  block && /^\)$/ { exit }
' "$lints_md")
if [[ -z $flags_block ]]; then
  echo "score.sh: no 'flags=(' block under '## The check command' in $lints_md" >&2
  exit 1
fi
flags=()
eval "$flags_block"
if ((${#flags[@]} == 0)); then
  echo "score.sh: the flags block in $lints_md is empty" >&2
  exit 1
fi

# s4's summary is CLI output; its rubric permits stdout in both passes.
scenario_flags=()
case $scenario in
s4-async) scenario_flags=(-A clippy::print_stdout) ;;
esac

sum_field() {
  # sum_field <n>: the sum of awk field n over stdin, 0 when stdin is empty
  awk -v n="$1" '{ s += $n } END { print s + 0 }'
}

count() {
  # count <pattern> <dir>: matches in .rs files under dir, 0 when none
  { rg -c --no-messages "$1" "$2" --glob '*.rs' || true; } | awk -F: '{ print $NF }' | sum_field 1
}

tests_passed() {
  # tests_passed <log>: passed tests summed over every `test result:` line
  { rg -o 'test result: ok\. [0-9]+ passed' "$1" || true; } | sum_field 4
}

lints() {
  # lints <tree> <log> [clippy args...]: "N (lint a, lint b, ...)" from the JSON
  # messages, or "BUILD FAILED (n errors)" when rustc reports an error that is
  # not a lint. A tree that does not compile then never scores as clean.
  local tree=$1 log=$2
  shift 2
  local messages errors status=0 finished denied_lints
  # Under -D warnings the exit status is non-zero for a lint finding too, so
  # inspect both completion and diagnostics before calling the run clean.
  messages=$(cd "$tree" && cargo clippy --no-deps --all-features --message-format=json "$@" \
    2>>"$log") || status=$?
  printf '%s\n' "$messages" >>"$log"
  if ! jq -es 'all(.[]; type == "object")' <<<"$messages" >/dev/null 2>&1; then
    printf 'INCOMPLETE (invalid compiler JSON; exit: %d)' "$status"
    return
  fi
  errors=$(jq -rs '
    [ .[] | select(.reason == "compiler-message") | .message
      | select(.level == "error")
      | select(
          (.code == null and (.message | startswith("aborting due to") | not))
          or (.code != null and (.code.code | test("^E[0-9]{4}$")))
        )
    ] | length' <<<"$messages")
  if ((errors > 0)); then
    printf 'BUILD FAILED (rustc errors: %d)' "$errors"
    return
  fi
  finished=$(jq -r 'select(.reason == "build-finished") | .success' <<<"$messages" | tail -n 1)
  denied_lints=$(jq -s '[.[] | select(.reason == "compiler-message") | .message
    | select(.level == "error" and .code != null)] | length' <<<"$messages")
  if { ((status == 0)) && [[ $finished != true ]]; } ||
    { ((status != 0)) && { [[ $finished != false ]] || ((denied_lints == 0)); }; }; then
    printf 'INCOMPLETE (cargo exit: %d; see score.log)' "$status"
    return
  fi
  jq -r 'select(.reason == "compiler-message") | .message | select(.code != null) | .code.code' \
    <<<"$messages" |
    sort | uniq -c | sort -rn |
    awk '
      { n += $1; parts = parts (parts ? ", " : "") $2 " " $1 }
      END { printf "%d (%s)", n, parts }
    '
}

score_arm() {
  local arm=$1
  local run="$here/results/$scenario/$arm"
  local tree="$run/tree"
  if [[ ! -d $tree/src ]]; then
    echo "| $arm | no tree | | | |"
    return
  fi
  local log="$run/score.log"
  : >"$log"
  local tests clippy_lib clippy_tests loc
  if (cd "$tree" && cargo test --quiet >>"$log" 2>&1); then
    tests="pass ($(tests_passed "$log") tests)"
  else
    tests="FAIL (see score.log)"
  fi
  clippy_lib=$(lints "$tree" "$log" -- "${flags[@]}" "${scenario_flags[@]}")
  clippy_tests=$(lints "$tree" "$log" --all-targets -- "${flags[@]}" "${scenario_flags[@]}" \
    -A clippy::unwrap_used -A clippy::panic_in_result_fn)
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
  bare_count=$(count "$pattern" "$here/results/$scenario/bare/tree/src")
  skill_count=$(count "$pattern" "$here/results/$scenario/skill/tree/src")
  # shellcheck disable=SC2016 # the backticks are literal Markdown, not a command substitution
  printf '| `%s` | %s | %s |\n' "$pattern" "$bare_count" "$skill_count"
done
