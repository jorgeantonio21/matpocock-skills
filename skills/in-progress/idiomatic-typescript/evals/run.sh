#!/usr/bin/env bash
# Run one paid model evaluation.
# usage: EVAL_PAID_RUNS=1 ./run.sh <scenario> <bare|skill|skill@<git-ref>> [model]
set -euo pipefail

usage() {
  echo "usage: EVAL_PAID_RUNS=1 run.sh <scenario> <bare|skill|skill@<git-ref>> [model]" >&2
  exit 2
}

(($# == 2 || $# == 3)) || usage
if [[ ${EVAL_PAID_RUNS:-0} != 1 ]]; then
  echo "run.sh: paid runs are locked; agree the budget, then set EVAL_PAID_RUNS=1" >&2
  exit 2
fi

scenario=$1
arm=$2
model=${3:-}
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=materialize.sh
source "$here/materialize.sh"
original_skill_dir=$(cd "$here/.." && pwd)
skill_dir=$original_skill_dir
repo=$(git -C "$skill_dir" rev-parse --show-toplevel)
skill_rel=${skill_dir#"$repo"/}
source="$here/scenarios/$scenario/start"
work_root=${EVAL_WORK_ROOT:-/tmp/idiomatic-typescript-eval}
limit_seconds=${EVAL_LIMIT_SECONDS:-2400}
expected_node=$(<"$here/fixtures/base/.node-version")
actual_node=$(node -p 'process.versions.node')
[[ $actual_node == "$expected_node" ]] || {
  echo "run.sh: Node $expected_node is required, found $actual_node" >&2
  exit 1
}

case $arm in
  bare | skill | skill@?*) ;;
  *) echo "run.sh: invalid arm '$arm'" >&2; usage ;;
esac
[[ -d $source ]] || { echo "run.sh: unknown scenario '$scenario'" >&2; usage; }

skill_revision=none
skill_load_tmp=""
trap '[[ -z ${skill_load_tmp:-} ]] || rm -rf "$skill_load_tmp"' EXIT
if [[ $arm == skill ]]; then
  skill_revision=$(git -C "$repo" rev-parse HEAD)
  if [[ -n $(git -C "$repo" status --porcelain -- "$skill_rel") ]]; then
    skill_revision="$skill_revision (with uncommitted changes)"
  fi
  mkdir -p "$work_root"
  skill_load_tmp=$(mktemp -d "$work_root/working-skill.XXXXXX")
  copy_skill "$skill_dir" "$skill_load_tmp"
  skill_dir=$skill_load_tmp
elif [[ $arm == skill@* ]]; then
  ref=${arm#skill@}
  revision=$(git -C "$repo" rev-parse --verify "$ref^{commit}")
  skill_revision=$revision
  export_root="$work_root/skill-guidance-exports/$revision"
  skill_dir="$export_root/$skill_rel"
  if [[ ! -f $skill_dir/SKILL.md ]]; then
    rm -rf "$export_root"
    mkdir -p "$export_root"
    git -C "$repo" archive "$revision" \
      "$skill_rel/SKILL.md" "$skill_rel/INVARIANTS.md" "$skill_rel/RUNTIME.md" \
      "$skill_rel/TOOLING.md" "$skill_rel/SOURCES.md" "$skill_rel/agents" \
      | tar -x -C "$export_root"
  fi
fi

out_arm="$here/results/$scenario/$arm"
mkdir -p "$out_arm"
run=$(($(find "$out_arm" -mindepth 1 -maxdepth 1 -type d -name 'r*' | wc -l) + 1))
out="$out_arm/r$run"
work="$work_root/$scenario/$arm-r$run-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$out"

materialize "$source" "$work"
if ! (cd "$work" && npm ci --ignore-scripts); then
  echo "run.sh: fixture dependency installation failed before the paid run" >&2
  exit 1
fi

prompt=$(<"$here/scenarios/$scenario/prompt.md")
# Separate task directories avoid accidental exposure, not reads elsewhere on the host.
args=(-p --safe-mode --dangerously-skip-permissions --output-format stream-json --verbose)
[[ -z $model ]] || args+=(--model "$model")
if [[ $arm != bare ]]; then
  system=$(printf 'A skill is loaded for this session.\n\nBase directory for this skill: %s\n\n%s' \
    "$skill_dir" "$(<"$skill_dir/SKILL.md")")
  args+=(--append-system-prompt "$system" --add-dir "$skill_dir")
  prompt="$prompt

Follow the idiomatic-typescript skill loaded in your instructions, including its verification gate."
fi

printf '%s\n' "$prompt" >"$out/prompt.txt"
printf '%s\n' "$skill_revision" >"$out/skill-revision.txt"
started=$(date +%s)
status=0
(
  cd "$work"
  printf '%s' "$prompt" | perl -e 'alarm shift; exec @ARGV' "$limit_seconds" \
    claude "${args[@]}"
) >"$out/transcript.jsonl" 2>"$out/stderr.log" || status=$?
finished=$(date +%s)

rsync -a --delete --exclude node_modules --exclude dist --exclude .claude "$work/" "$out/tree/"
python3 "$here/analyze.py" meta "$out" "$status" "$started" "$finished" \
  "$arm" "$scenario" "${model:-default}" "$skill_revision"
