#!/usr/bin/env bash
# Run one evaluation scenario in one arm with headless Claude Code.
#
# usage: run.sh <scenario> <bare|skill|skill@<git-ref>> [model]
#
# bare            the model, the prompt, the starting crate, and the Rust toolchain.
# skill           the same, plus the working tree's SKILL.md in the system prompt.
# skill@<git-ref> the same, plus the skill as committed at <git-ref>, so two revisions of the
#                 skill are compared on the same model. The export is cached under EVAL_WORK_ROOT.
#
# The scenario's start/ crate is copied to a throwaway directory under EVAL_WORK_ROOT (default
# /tmp/idiomatic-rust-eval), the agent runs there with every permission granted, and the final
# tree (minus target/) plus the transcript land under results/<scenario>/<arm>/r<N>/, where N is
# one more than the runs of that arm so far. Repeated runs accumulate. EVAL_LIMIT_SECONDS (default
# 2400) kills a run that overstays.
set -euo pipefail

usage() {
  echo "usage: run.sh <scenario> <bare|skill|skill@<git-ref>> [model]" >&2
  exit 2
}

(($# == 2 || $# == 3)) || usage
scenario=$1
arm=$2
model=${3:-}

here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
start="$here/scenarios/$scenario/start"
work_root=${EVAL_WORK_ROOT:-/tmp/idiomatic-rust-eval}
limit_seconds=${EVAL_LIMIT_SECONDS:-2400}

case $arm in
bare | skill | skill@?*) ;;
*)
  echo "run.sh: the arm is 'bare', 'skill', or 'skill@<git-ref>', not '$arm'" >&2
  usage
  ;;
esac
if [[ ! -d $start ]]; then
  echo "run.sh: no start/ crate for scenario '$scenario' under $here/scenarios" >&2
  usage
fi

# The skill files the skill arms load: the working tree, or an export of the named revision.
# skill-revision.txt in the results records which, so a matrix can name the revision it measured.
skill_revision=none
if [[ $arm == skill ]]; then
  skill_revision=$(git -C "$skill_dir" rev-parse HEAD)
  if [[ -n $(git -C "$skill_dir" status --porcelain -- .) ]]; then
    skill_revision="$skill_revision (with uncommitted changes)"
  fi
elif [[ $arm == skill@* ]]; then
  ref=${arm#skill@}
  skill_revision=$(git -C "$skill_dir" rev-parse --verify "$ref^{commit}")
  export_dir="$work_root/skill-exports/$skill_revision"
  if [[ ! -f $export_dir/SKILL.md ]]; then
    mkdir -p "$export_dir"
    # Run from the skill directory, git archive writes paths relative to it.
    git -C "$skill_dir" archive "$skill_revision" | tar -x -C "$export_dir"
  fi
  skill_dir=$export_dir
fi

out_arm="$here/results/$scenario/$arm"
mkdir -p "$out_arm"
run=$(($(find "$out_arm" -mindepth 1 -maxdepth 1 -type d -name 'r*' | wc -l) + 1))
out="$out_arm/r$run"
# A fresh work dir per run; the old ones are throwaway and stay where they are.
work="$work_root/$scenario/$arm-r$run-$(date +%Y%m%d-%H%M%S)"

mkdir -p "$work" "$out"
rsync -aL --exclude target "$start/" "$work/"

prompt=$(<"$here/scenarios/$scenario/prompt.md")
args=(-p --safe-mode --dangerously-skip-permissions --output-format stream-json --verbose)
if [[ -n $model ]]; then
  args+=(--model "$model")
fi

if [[ $arm != bare ]]; then
  system=$(printf 'A skill is loaded for this session.\n\nBase directory for this skill: %s\n\n%s' \
    "$skill_dir" "$(<"$skill_dir/SKILL.md")")
  args+=(--append-system-prompt "$system" --add-dir "$skill_dir")
  prompt="$prompt

Follow the idiomatic-rust skill loaded in your instructions, including its Check step."
fi

printf '%s\n' "$prompt" >"$out/prompt.txt"
printf '%s\n' "$skill_revision" >"$out/skill-revision.txt"
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
  "$arm" "$scenario" "${model:-default}" "$skill_revision"
