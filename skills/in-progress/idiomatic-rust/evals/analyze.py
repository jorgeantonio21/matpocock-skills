#!/usr/bin/env python3
"""Read the stream-json transcripts under results/.

Usage:
  analyze.py summary [results-dir]
      One line per run: exit, wall time, turns, cost, model, the skill files it
      read, and whether it ran the check command and fmt.
  analyze.py meta OUT STATUS STARTED FINISHED ARM SCENARIO MODEL
      Called by run.sh after one run. Writes OUT/final-message.md and
      OUT/meta.json from OUT/transcript.jsonl.
"""

import argparse
import json
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL_FILES = ("SKILL.md", "RUNTIME.md", "CRATES.md", "LINTS.md", "INVARIANTS.md")
BRIEF_KEYS = ("command", "file_path", "pattern")
BRIEF_WIDTH = 120
CHECK_NEEDLES = ("clippy::pedantic", "flags=(")
FMT_NEEDLES = ("+nightly fmt", "cargo fmt", "rustfmt")


@dataclass
class ToolCall:
    """One tool_use block: the tool's name and its input, serialised once for searching."""

    name: str
    input: dict
    blob: str = field(init=False)

    def __post_init__(self) -> None:
        self.blob = json.dumps(self.input)

    def brief(self) -> str:
        """The tool name and the first argument that identifies the call, truncated."""
        arg = next((self.input[key] for key in BRIEF_KEYS if self.input.get(key)), "")
        arg = str(arg)[:BRIEF_WIDTH]
        return f"{self.name}: {arg}"


@dataclass
class Transcript:
    """What one transcript.jsonl says: the result event, the tool calls, the last text."""

    result: dict = field(default_factory=dict)
    tool_calls: list[ToolCall] = field(default_factory=list)
    last_text: str = ""

    def finished(self) -> bool:
        """True when the run reached its result event."""
        return bool(self.result) and not self.result.get("is_error", False) and self.result.get("subtype", "success") == "success"

    def skill_files_read(self) -> list[str]:
        """The skill files named in any tool call's input, sorted."""
        read = {
            name
            for call in self.tool_calls
            for name in SKILL_FILES
            if f"idiomatic-rust/{name}" in call.blob
        }
        return sorted(read)

    def ran(self, needles: tuple[str, ...]) -> bool:
        """True when any tool call's input contains one of the needles."""
        return any(
            needle in call.blob for call in self.tool_calls for needle in needles
        )


def parse_transcript(path: Path) -> Transcript:
    """Walk a stream-json transcript once and keep what the reports need."""
    transcript = Transcript()
    with path.open(encoding="utf-8") as lines:
        for line in lines:
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("type") == "result":
                transcript.result = event
            if event.get("type") != "assistant":
                continue
            for block in event.get("message", {}).get("content", []):
                if block.get("type") == "tool_use":
                    transcript.tool_calls.append(
                        ToolCall(block.get("name", ""), block.get("input", {}))
                    )
                elif block.get("type") == "text" and block.get("text", "").strip():
                    transcript.last_text = block["text"]
    return transcript


def print_summary(results: Path) -> None:
    """One line per run under results/, in scenario then arm order."""
    print(
        f"{'scenario':14} {'arm':6} {'exit':4} {'wall':>5} {'turns':>5} {'cost':>6}  "
        "model, skill files read, check, fmt, run"
    )
    for path in sorted(results.rglob("transcript.jsonl")):
        scenario, arm = path.parts[-3:-1]
        transcript = parse_transcript(path)
        result = transcript.result
        models = [
            model for model in result.get("modelUsage", {}) if "haiku" not in model
        ]
        exit_word = "ok" if transcript.finished() else "incomplete"
        duration = result.get("duration_ms")
        wall = duration // 1000 if duration is not None else "unknown"
        turns = result.get("num_turns", "")
        reported_cost = result.get("total_cost_usd")
        cost = f"{reported_cost:.2f}" if reported_cost is not None else "unknown"
        print(
            f"{scenario:14} {arm:6} {exit_word:4} {wall:>5} {turns!s:>5} {cost:>7}  "
            f"{models} read={transcript.skill_files_read()} "
            f"check={transcript.ran(CHECK_NEEDLES)} fmt={transcript.ran(FMT_NEEDLES)} "
            f"run={path.parent.relative_to(results)}"
        )


def write_meta(args: argparse.Namespace) -> None:
    """Write final-message.md and meta.json for one finished run."""
    out: Path = args.out
    transcript = parse_transcript(out / "transcript.jsonl")
    (out / "final-message.md").write_text(transcript.last_text, encoding="utf-8")
    meta = {
        "scenario": args.scenario,
        "arm": args.arm,
        "model": args.model,
        "exit_status": args.status,
        "complete": args.status == 0 and transcript.finished(),
        "result_subtype": transcript.result.get("subtype"),
        "wall_seconds": args.finished - args.started,
        "num_turns": transcript.result.get("num_turns"),
        "total_cost_usd": transcript.result.get("total_cost_usd"),
        "model_usage": list(transcript.result.get("modelUsage", {}).keys()),
        "tool_calls": [call.brief() for call in transcript.tool_calls],
    }
    (out / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps({key: value for key, value in meta.items() if key != "tool_calls"})
    )
    print(f"tool calls: {len(transcript.tool_calls)}")


def main() -> None:
    """Dispatch on the subcommand."""
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="command", required=True)

    summary = commands.add_parser("summary", help="one line per run under results/")
    summary.add_argument("results", nargs="?", type=Path, default=HERE / "results")

    meta = commands.add_parser(
        "meta", help="write final-message.md and meta.json for one run"
    )
    meta.add_argument(
        "out", type=Path, help="the run's results/<scenario>/<arm> directory"
    )
    meta.add_argument("status", type=int, help="the claude process exit status")
    meta.add_argument("started", type=int, help="epoch seconds when the run started")
    meta.add_argument("finished", type=int, help="epoch seconds when the run finished")
    meta.add_argument("arm", help="bare, merged, or skill")
    meta.add_argument("scenario", help="the scenario name")
    meta.add_argument("model", help="the model passed to run.sh, or 'default'")

    args = parser.parse_args()
    if args.command == "summary":
        print_summary(args.results)
    else:
        write_meta(args)


if __name__ == "__main__":
    main()
