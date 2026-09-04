#!/usr/bin/env bash
# Run one evaluation scenario in one arm with headless Claude Code.
#
# usage: run.sh <scenario> <bare|skill> [model]
#
# The scenario's start/ crate is copied to a throwaway directory under
# /tmp, the agent runs there with every permission granted, and the final
# tree (minus target/) plus the transcript land under results/.
set -euo pipefail

scenario=$1
arm=$2
model=${3:-}

here=$(cd "$(dirname "$0")" && pwd)
skill_dir=$(cd "$here/.." && pwd)
work_root=${EVAL_WORK_ROOT:-/tmp/idiomatic-rust-eval}
# A fresh work dir per run; the old ones are throwaway and stay in /tmp.
work="$work_root/$scenario/$arm-$(date +%Y%m%d-%H%M%S)"
out="$here/results/$scenario/$arm"
limit_seconds=${EVAL_LIMIT_SECONDS:-2400}

mkdir -p "$work" "$out"
rsync -aL --exclude target "$here/scenarios/$scenario/start/" "$work/"

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

python3 - "$out" "$status" "$started" "$finished" "$arm" "$scenario" "${model:-default}" <<'PY'
import json, sys
out, status, started, finished, arm, scenario, model = sys.argv[1:]
result = {}
tool_calls = []
last_text = ""
for line in open(f"{out}/transcript.jsonl", encoding="utf-8"):
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        continue
    if event.get("type") == "result":
        result = event
    if event.get("type") == "assistant":
        for block in event.get("message", {}).get("content", []):
            if block.get("type") == "tool_use":
                name = block.get("name")
                inp = block.get("input", {})
                brief = inp.get("command") or inp.get("file_path") or inp.get("pattern") or ""
                tool_calls.append(f"{name}: {str(brief)[:120]}")
            if block.get("type") == "text" and block.get("text", "").strip():
                last_text = block["text"]
open(f"{out}/final-message.md", "w").write(last_text)
meta = {
    "scenario": scenario,
    "arm": arm,
    "model": model,
    "exit_status": int(status),
    "wall_seconds": int(finished) - int(started),
    "num_turns": result.get("num_turns"),
    "total_cost_usd": result.get("total_cost_usd"),
    "model_usage": list(result.get("modelUsage", {}).keys()),
    "tool_calls": tool_calls,
}
json.dump(meta, open(f"{out}/meta.json", "w"), indent=2)
print(json.dumps({k: v for k, v in meta.items() if k != "tool_calls"}))
print(f"tool calls: {len(tool_calls)}")
PY
