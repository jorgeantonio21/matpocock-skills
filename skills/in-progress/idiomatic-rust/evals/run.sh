#!/usr/bin/env bash
# Run one evaluation scenario in one arm with headless Claude Code.
#
# usage: run.sh <scenario> <bare|skill> [model]
#
# The scenario's start/ crate is copied to a throwaway directory under
# EVAL_WORK_ROOT (default /tmp/idiomatic-rust-eval), the agent runs there with
# every permission granted, and the final tree (minus target/) plus the
# transcript land under results/. EVAL_LIMIT_SECONDS (default 2400) kills a
# run that overstays.
set -euo pipefail

usage() {
  echo "usage: run.sh <scenario> <bare|skill> [model]" >&2
  exit 2
}

(($# == 2 || $# == 3)) || usage
scenario=$1
arm=$2
model=${3:-}

here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
start="$here/scenarios/$scenario/start"

case $arm in
bare | skill) ;;
*)
  echo "run.sh: the arm is 'bare' or 'skill', not '$arm'" >&2
  usage
  ;;
esac
if [[ ! -d $start ]]; then
  echo "run.sh: no start/ crate for scenario '$scenario' under $here/scenarios" >&2
  usage
fi

work_root=${EVAL_WORK_ROOT:-/tmp/idiomatic-rust-eval}
# A fresh work dir per run; the old ones are throwaway and stay where they are.
work="$work_root/$scenario/$arm-$(date +%Y%m%d-%H%M%S)"
out="$here/results/$scenario/$arm"
limit_seconds=${EVAL_LIMIT_SECONDS:-2400}

mkdir -p "$work" "$out"
rsync -aL --exclude target "$start/" "$work/"

prompt=$(<"$here/scenarios/$scenario/prompt.md")
args=(-p --safe-mode --dangerously-skip-permissions --output-format stream-json --verbose)
if [[ -n $model ]]; then
  args+=(--model "$model")
fi

if [[ $arm == skill ]]; then
  system=$(printf 'A skill is loaded for this session.\n\nBase directory for this skill: %s\n\n%s' \
    "$skill_dir" "$(<"$skill_dir/SKILL.md")")
  args+=(--append-system-prompt "$system" --add-dir "$skill_dir")
  prompt="$prompt

Follow the idiomatic-rust skill loaded in your instructions, including its Check step."
fi

printf '%s\n' "$prompt" >"$out/prompt.txt"
started=$(date +%s)
cd "$work"
status=0
# The prompt goes through stdin: --add-dir is variadic and would swallow a
# trailing positional prompt.
printf '%s' "$prompt" | perl -e 'alarm shift; exec @ARGV' "$limit_seconds" \
  claude "${args[@]}" >"$out/transcript.jsonl" 2>"$out/stderr.log" || status=$?
finished=$(date +%s)

rsync -a --delete --exclude target --exclude .claude "$work/" "$out/tree/"

python3 "$here/analyze.py" meta "$out" "$status" "$started" "$finished" \
  "$arm" "$scenario" "${model:-default}"
