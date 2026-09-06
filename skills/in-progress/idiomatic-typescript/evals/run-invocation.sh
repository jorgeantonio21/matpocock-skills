#!/usr/bin/env bash
# Test automatic skill discovery separately from forced-loading evaluations.
# usage: EVAL_PAID_RUNS=1 ./run-invocation.sh [model]
set -euo pipefail

if [[ ${EVAL_PAID_RUNS:-0} != 1 ]]; then
  echo "run-invocation.sh: paid runs are locked; agree the budget, then set EVAL_PAID_RUNS=1" >&2
  exit 2
fi

here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=materialize.sh
source "$here/materialize.sh"
skill_dir=$(cd "$here/.." && pwd)
model=${1:-}
out="$here/.invocation-results/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$out"
failures=0

while IFS=$'\t' read -r name expected prompt; do
  work=$(mktemp -d)
  mkdir -p "$work/.claude/skills/idiomatic-typescript"
  copy_skill "$skill_dir" "$work/.claude/skills/idiomatic-typescript"
  printf 'export const value: unknown = 1;\n' >"$work/sample.ts"
  printf 'export const value = 1;\n' >"$work/sample.js"
  printf '{"type":"module"}\n' >"$work/package.json"
  printf '{"compilerOptions":{"module":"NodeNext"}}\n' >"$work/tsconfig.json"
  printf 'A short note.\n' >"$work/note.md"
  args=(-p --permission-mode plan --setting-sources project --strict-mcp-config \
    --mcp-config '{}' --output-format stream-json --verbose)
  [[ -z $model ]] || args+=(--model "$model")
  status=0
  (cd "$work" && printf '%s' "$prompt" | claude "${args[@]}") \
    >"$out/$name.jsonl" 2>"$out/$name.stderr" || status=$?
  if ((status != 0)) || ! jq -e 'select(.type == "result" and .is_error == false)' \
    "$out/$name.jsonl" >/dev/null; then
    actual=incomplete
  elif jq -e '
    select(.type == "assistant")
    | .message.content[]?
    | select(.type == "tool_use")
    | select((.name | ascii_downcase) == "skill")
    | select((.input.skill // .input.name // "") == "idiomatic-typescript")
  ' "$out/$name.jsonl" >/dev/null; then
    actual=invoke
  else
    actual=skip
  fi
  printf '%s\t%s\t%s\n' "$name" "$expected" "$actual"
  [[ $actual == "$expected" ]] || failures=$((failures + 1))
  rm -rf "$work"
done <"$here/invocation/cases.tsv"

((failures == 0)) || { echo "run-invocation.sh: $failures invocation mismatch(es)" >&2; exit 1; }
