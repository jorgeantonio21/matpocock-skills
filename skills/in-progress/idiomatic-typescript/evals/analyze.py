#!/usr/bin/env python3
"""Summarize idiomatic-typescript evaluation transcripts and scores."""

import argparse
import json
from collections import defaultdict
from pathlib import Path
from statistics import mean

HERE = Path(__file__).resolve().parent
SKILL_FILES = ("SKILL.md", "INVARIANTS.md", "RUNTIME.md", "TOOLING.md", "SOURCES.md")


def load(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def transcript(path: Path) -> tuple[dict, list[dict], str]:
    result: dict = {}
    calls: list[dict] = []
    last_text = ""
    if not path.is_file():
        return result, calls, last_text
    for line in path.read_text(encoding="utf-8").splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "result":
            result = event
        if event.get("type") != "assistant":
            continue
        for block in event.get("message", {}).get("content", []):
            if block.get("type") == "tool_use":
                calls.append(block)
            elif block.get("type") == "text" and block.get("text", "").strip():
                last_text = block["text"]
    return result, calls, last_text


def run_dirs(results: Path) -> list[Path]:
    return sorted(path for path in results.glob("*/*/r*") if path.is_dir())


def call_blobs(calls: list[dict]) -> list[str]:
    return [json.dumps(call.get("input", {})) for call in calls]


def summary(results: Path) -> None:
    print("scenario arm run exit wall turns input-tokens output-tokens cost skill-files checks")
    for run in run_dirs(results):
        result, calls, _ = transcript(run / "transcript.jsonl")
        blobs = call_blobs(calls)
        files = [name for name in SKILL_FILES if any(f"/{name}" in blob for blob in blobs)]
        checks = any("npm test" in blob or "typecheck" in blob for blob in blobs)
        cost = result.get("total_cost_usd") or 0
        usage = result.get("usage", {})
        input_tokens = sum(
            usage.get(key, 0)
            for key in ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens")
        )
        print(
            run.parts[-3], run.parts[-2], run.name,
            "ok" if result else "RUN",
            result.get("duration_ms", 0) // 1000,
            result.get("num_turns", ""), input_tokens,
            usage.get("output_tokens", 0), f"{cost:.2f}", files, checks,
        )


def ratio(scores: list[dict], key: str, allowed: tuple[str, ...]) -> str:
    if not scores:
        return "unscored"
    return f"{sum(score.get(key, {}).get('verdict') in allowed for score in scores)}/{len(scores)}"


def matrix(results: Path) -> None:
    groups: dict[tuple[str, str], list[Path]] = defaultdict(list)
    for run in run_dirs(results):
        groups[(run.parts[-3], run.parts[-2])].append(run)
    print("| scenario | arm | runs | typecheck | runtime | external | mean changed lines | mean wall s | mean input tokens | mean output tokens | mean cost $ | skill revision |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for (scenario, arm), runs in sorted(groups.items()):
        scores = [score for score in (load(run / "score.json") for run in runs) if score]
        metas = [meta for meta in (load(run / "meta.json") for run in runs) if meta]
        changed = [score["diff"]["added"] + score["diff"]["removed"] for score in scores]
        revisions = sorted({(run / "skill-revision.txt").read_text(encoding="utf-8").strip()[:12] for run in runs if (run / "skill-revision.txt").is_file()})
        print(
            f"| {scenario} | {arm} | {len(runs)} | {ratio(scores, 'typecheck', ('pass',))} "
            f"| {ratio(scores, 'runtime', ('pass',))} | {ratio(scores, 'external', ('pass', 'none'))} "
            f"| {mean(changed) if changed else 0:.0f} | {mean([m.get('wall_seconds', 0) for m in metas]) if metas else 0:.0f} "
            f"| {mean([m.get('input_tokens', 0) for m in metas]) if metas else 0:.0f} "
            f"| {mean([m.get('output_tokens', 0) for m in metas]) if metas else 0:.0f} "
            f"| {mean([(m.get('total_cost_usd') or 0) for m in metas]) if metas else 0:.2f} "
            f"| {', '.join(revisions) or 'none'} |"
        )


def write_meta(args: argparse.Namespace) -> None:
    result, calls, last_text = transcript(args.out / "transcript.jsonl")
    (args.out / "final-message.md").write_text(last_text, encoding="utf-8")
    usage = result.get("usage", {})
    meta = {
        "scenario": args.scenario,
        "arm": args.arm,
        "model": args.model,
        "skill_revision": args.skill_revision,
        "exit_status": args.status,
        "wall_seconds": args.finished - args.started,
        "num_turns": result.get("num_turns"),
        "input_tokens": sum(
            usage.get(key, 0)
            for key in ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens")
        ),
        "output_tokens": usage.get("output_tokens", 0),
        "thinking_tokens": usage.get("output_tokens_details", {}).get("thinking_tokens", 0),
        "total_cost_usd": result.get("total_cost_usd"),
        "model_usage": list(result.get("modelUsage", {})),
        "tool_calls": [f"{call.get('name', '')}: {json.dumps(call.get('input', {}))[:160]}" for call in calls],
    }
    (args.out / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in meta.items() if key != "tool_calls"}))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("summary", "matrix"):
        command = commands.add_parser(name)
        command.add_argument("results", nargs="?", type=Path, default=HERE / "results")
    meta = commands.add_parser("meta")
    meta.add_argument("out", type=Path)
    meta.add_argument("status", type=int)
    meta.add_argument("started", type=int)
    meta.add_argument("finished", type=int)
    meta.add_argument("arm")
    meta.add_argument("scenario")
    meta.add_argument("model")
    meta.add_argument("skill_revision")
    args = parser.parse_args()
    if args.command == "summary":
        summary(args.results)
    elif args.command == "matrix":
        matrix(args.results)
    else:
        write_meta(args)


if __name__ == "__main__":
    main()
