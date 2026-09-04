#!/usr/bin/env python3
"""Summarize every run under results/: model, cost, which skill files it read, whether it ran the check.

Usage: analyze.py [results-dir]
"""

import glob
import json
import os
import sys

results = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "results")
SKILL_FILES = ("SKILL.md", "RUNTIME.md", "CRATES.md", "LINTS.md")

print(f"{'scenario':14} {'arm':6} {'exit':4} {'wall':>5} {'turns':>5} {'cost':>6}  model, skill files read, check, fmt")
for path in sorted(glob.glob(f"{results}/*/*/transcript.jsonl")):
    scenario, arm = path.split("/")[-3:-1]
    result = {}
    text = []
    read = set()
    check = fmt = False
    for line in open(path, encoding="utf-8"):
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "result":
            result = event
        if event.get("type") != "assistant":
            continue
        for block in event.get("message", {}).get("content", []):
            if block.get("type") != "tool_use":
                continue
            blob = json.dumps(block.get("input", {}))
            for name in SKILL_FILES:
                if f"idiomatic-rust/{name}" in blob:
                    read.add(name)
            if "clippy::pedantic" in blob or "flags=(" in blob:
                check = True
            if "+nightly fmt" in blob:
                fmt = True
    models = [m for m in result.get("modelUsage", {}) if "haiku" not in m]
    finished = bool(result)
    print(
        f"{scenario:14} {arm:6} {'ok' if finished else 'RUN':4} {result.get('duration_ms', 0) // 1000:>5} "
        f"{str(result.get('num_turns', '')):>5} {result.get('total_cost_usd', 0) or 0:6.2f}  "
        f"{models} read={sorted(read)} check={check} fmt={fmt}"
    )
