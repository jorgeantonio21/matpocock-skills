#!/usr/bin/env bash
# Run one isolated arm. Repeated comparisons use compare.sh.
set -euo pipefail
usage() { echo 'usage: run.sh <scenario> <bare|merged|skill> [model]' >&2; exit 2; }
(($# == 2 || $# == 3)) || usage
scenario=$1
arm=$2
model=${3:-}
here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
case $arm in bare | merged | skill) ;; *) usage ;; esac
[[ $scenario =~ ^[a-z0-9-]+$ && -d $here/scenarios/$scenario/start ]] || usage

results_root=${EVAL_RESULTS_ROOT:-$here/results}
mkdir -p "$results_root"
results_root=$(cd "$results_root" && pwd)
out="$results_root/$scenario/$arm"
if [[ -e $out/transcript.jsonl ]]; then
  echo "run.sh: refusing to overwrite $out; choose a fresh EVAL_RESULTS_ROOT" >&2
  exit 2
fi
work_root=${EVAL_WORK_ROOT:-/tmp/idiomatic-rust-eval}
mkdir -p "$work_root" "$out"
work=$(mktemp -d "$work_root/$scenario-$arm-XXXXXX")
rsync -aL --exclude target "$here/scenarios/$scenario/start/" "$work/"
args=(-p --safe-mode --dangerously-skip-permissions --output-format stream-json --verbose)
[[ -z $model ]] || args+=(--model "$model")
[[ -z ${EVAL_MAX_BUDGET_USD:-} ]] || args+=(--max-budget-usd "$EVAL_MAX_BUDGET_USD")
prompt=$(<"$here/scenarios/$scenario/prompt.md")
if [[ $arm != bare ]]; then
  snapshot="$out/idiomatic-rust"
  mkdir -p "$snapshot"
  if [[ $arm == merged ]]; then
    baseline=9fe7b179dd3a8652d1c0fb4d9935011f39d953af
    for file in SKILL.md RUNTIME.md CRATES.md LINTS.md; do
      git -C "$skill_dir" show "$baseline:skills/in-progress/idiomatic-rust/$file" >"$snapshot/$file"
    done
  else
    revision_dir=${EVAL_SKILL_SOURCE:-$skill_dir}
    cp "$revision_dir/"*.md "$snapshot/"
  fi
  system=$(printf 'A skill is loaded for this session.\n\nBase directory for this skill: %s\n\n%s' "$snapshot" "$(<"$snapshot/SKILL.md")")
  args+=(--append-system-prompt "$system" --add-dir "$snapshot")
  prompt="$prompt

Follow the idiomatic-rust skill loaded in your instructions, including its Check step."
fi
printf '%s\n' "$prompt" >"$out/prompt.txt"
(cd "$work" && rustc -Vv && cargo -V && claude --version) >"$out/toolchain.txt"
python3 "$here/provenance.py" "$out" "$work" "$here/scenarios/$scenario"
started=$(date +%s)
status=0
cd "$work"
printf '%s' "$prompt" | perl -e 'alarm shift; exec @ARGV' "${EVAL_LIMIT_SECONDS:-2400}" \
  claude "${args[@]}" >"$out/transcript.jsonl" 2>"$out/stderr.log" || status=$?
finished=$(date +%s)
rsync -a --delete --exclude target --exclude .claude "$work/" "$out/tree/"
python3 "$here/analyze.py" meta "$out" "$status" "$started" "$finished" "$arm" "$scenario" "${model:-default}"
# An API error or budget/turn limit may still leave a result event and exit zero.
python3 - "$out/meta.json" <<'PY'
import json, sys
meta = json.load(open(sys.argv[1]))
if not meta['complete']:
    raise SystemExit(1)
PY
