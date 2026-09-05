#!/usr/bin/env bash
# Score every run of one scenario without exposing acceptance tests to the model.
# usage: ./score.sh <scenario>
set -euo pipefail

(($# == 1)) || { echo "usage: score.sh <scenario>" >&2; exit 2; }
scenario=$1
here=$(cd "$(dirname "$0")" && pwd)
scenario_dir="$here/scenarios/$scenario"
[[ -d $scenario_dir ]] || { echo "score.sh: unknown scenario '$scenario'" >&2; exit 2; }
# shellcheck source=materialize.sh
source "$here/materialize.sh"

run_command() {
  local kind=$1 tree=$2 log=$3
  shift 3
  local output status=0
  output=$(cd "$tree" && "$@" 2>&1) || status=$?
  printf '%s\n' "$output" >>"$log"
  if ((status == 0)); then
    verdict=pass
    text=pass
  elif [[ $output =~ error\ TS[0-9]+ ]]; then
    verdict=fail
    text="FAIL (type diagnostic)"
  elif [[ $kind == test && $output =~ (not\ ok|fail[^a-zA-Z]) ]]; then
    verdict=fail
    text="FAIL (runtime test)"
  else
    verdict=incomplete
    text="INCOMPLETE (exit $status; see score.log)"
  fi
}

install_tree() {
  local tree=$1 log=$2 output status=0
  if [[ ! -f $tree/.node-version ]] || [[ $(<"$tree/.node-version") != "$(node -p 'process.versions.node')" ]]; then
    printf 'Node version does not match .node-version\n' >>"$log"
    return 65
  fi
  output=$(cd "$tree" && npm ci --ignore-scripts 2>&1) || status=$?
  printf '%s\n' "$output" >>"$log"
  if ((status == 0)); then
    return 0
  fi
  return "$status"
}

external() {
  local tree=$1 log=$2 scratch
  if [[ ! -d $scenario_dir/verify ]]; then
    external_verdict=none
    external_text=none
    return
  fi
  scratch=$(mktemp -d)
  rsync -a --exclude node_modules --exclude dist "$tree/" "$scratch/"
  [[ ! -d $scenario_dir/verify/tests ]] || rsync -a "$scenario_dir/verify/tests/" "$scratch/tests/"
  [[ ! -d $scenario_dir/verify/type-tests ]] || {
    mkdir -p "$scratch/type-tests"
    rsync -a "$scenario_dir/verify/type-tests/" "$scratch/type-tests/"
  }
  if ! install_tree "$scratch" "$log"; then
    external_verdict=incomplete
    external_text="INCOMPLETE (npm ci; see score.log)"
  else
    run_command test "$scratch" "$log" npm test
    external_verdict=$verdict
    external_text=$text
  fi
  rm -rf "$scratch"
}

materialized_start() {
  local destination=$1
  materialize "$scenario_dir/start" "$destination"
}

dependency_names() {
  local package=$1
  jq -r '((.dependencies // {}) + (.devDependencies // {})) | keys[]' "$package" 2>/dev/null | sort || true
}

new_dependencies() {
  local start=$1 tree=$2
  comm -13 \
    <(dependency_names "$start/package.json") \
    <(dependency_names "$tree/package.json") \
    | paste -sd, -
}

diff_stats() {
  local start=$1 tree=$2 numstat
  numstat=$(git diff --no-index --numstat "$start" "$tree" 2>/dev/null || true)
  added=$(awk -F'\t' '$3 ~ /\.(ts|tsx|mts|cts|mjs|json)$/ && $3 !~ /(node_modules|dist|package-lock)/ && $1 != "-" { s += $1 } END { print s + 0 }' <<<"$numstat")
  removed=$(awk -F'\t' '$3 ~ /\.(ts|tsx|mts|cts|mjs|json)$/ && $3 !~ /(node_modules|dist|package-lock)/ && $2 != "-" { s += $2 } END { print s + 0 }' <<<"$numstat")
}

score_run() {
  local arm=$1 run=$2
  local dir="$here/results/$scenario/$arm/$run"
  local tree="$dir/tree" label="$arm/$run" log="$dir/score.log"
  if [[ ! -f $tree/package.json ]]; then
    echo "| $label | INCOMPLETE (no package.json) | INCOMPLETE (no package.json) | INCOMPLETE (no package.json) | 0 | +0/-0 | none |"
    jq -n --arg scenario "$scenario" --arg arm "$arm" --arg run "$run" \
      '{scenario: $scenario, arm: $arm, run: $run,
        typecheck: {verdict: "incomplete"}, runtime: {verdict: "incomplete"},
        external: {verdict: "incomplete"}, loc: 0,
        diff: {added: 0, removed: 0, new_deps: []}}' >"$dir/score.json"
    return
  fi
  : >"$log"
  local install_verdict=pass
  if ! install_tree "$tree" "$log"; then
    install_verdict=incomplete
  fi

  local typecheck typecheck_verdict runtime runtime_verdict verdict text
  if [[ $install_verdict == pass ]]; then
    run_command type "$tree" "$log" npm run typecheck
    typecheck=$text typecheck_verdict=$verdict
    run_command test "$tree" "$log" npm run test:runtime
    runtime=$text runtime_verdict=$verdict
  else
    typecheck="INCOMPLETE (npm ci)" typecheck_verdict=incomplete
    runtime="INCOMPLETE (npm ci)" runtime_verdict=incomplete
  fi

  local external_text external_verdict
  external "$tree" "$log"

  local start added removed deps loc
  start=$(mktemp -d)
  materialized_start "$start"
  diff_stats "$start" "$tree"
  deps=$(new_dependencies "$start" "$tree")
  rm -rf "$start"
  loc=$(find "$tree/src" -type f \( -name '*.ts' -o -name '*.tsx' -o -name '*.mts' -o -name '*.cts' \) -print0 \
    | xargs -0 cat 2>/dev/null | { rg -v '^\s*$' || true; } | wc -l | tr -d ' ')

  echo "| $label | $typecheck | $runtime | $external_text | $loc | +$added/-$removed | ${deps:-none} |"
  jq -n \
    --arg scenario "$scenario" --arg arm "$arm" --arg run "$run" \
    --arg typecheck "$typecheck_verdict" --arg runtime "$runtime_verdict" \
    --arg external "$external_verdict" --argjson loc "$loc" \
    --argjson added "$added" --argjson removed "$removed" --arg deps "$deps" \
    '{scenario: $scenario, arm: $arm, run: $run,
      typecheck: {verdict: $typecheck}, runtime: {verdict: $runtime}, external: {verdict: $external},
      loc: $loc, diff: {added: $added, removed: $removed,
        new_deps: ($deps | split(",") | map(select(. != "")))}}' >"$dir/score.json"
}

runs=()
while IFS= read -r path; do runs+=("$path"); done \
  < <(find "$here/results/$scenario" -mindepth 2 -maxdepth 2 -type d -name 'r*' 2>/dev/null | sort)
((${#runs[@]} > 0)) || { echo "score.sh: no runs for '$scenario'" >&2; exit 1; }

echo "## $scenario"
echo
echo "| run | typecheck | runtime tests | external acceptance | source lines | diff vs start | new deps |"
echo "| --- | --- | --- | --- | --- | --- | --- |"
for path in "${runs[@]}"; do
  score_run "$(basename "$(dirname "$path")")" "$(basename "$path")"
done
