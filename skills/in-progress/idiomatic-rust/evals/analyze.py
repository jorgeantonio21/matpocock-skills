#!/usr/bin/env python3
"""Read the transcripts and scores under results/.

Usage:
  analyze.py summary [results-dir]
      One line per run: exit, wall time, turns, cost, model, the skill files it
      read, and whether it ran the check command and fmt.
  analyze.py matrix [results-dir]
      One Markdown row per scenario and arm over every run of it: how many runs, how
      many passed their own tests, the external tests, and both check passes, the mean
      wall time and cost, and the skill revision measured. Needs score.sh to have run.
  analyze.py meta OUT STATUS STARTED FINISHED ARM SCENARIO MODEL SKILL_REVISION
      Called by run.sh after one run. Writes OUT/final-message.md and
      OUT/meta.json from OUT/transcript.jsonl.
"""

import argparse
import json
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from statistics import mean

HERE = Path(__file__).resolve().parent
SKILL_FILES = ("SKILL.md", "INVARIANTS.md", "RUNTIME.md", "CRATES.md", "LINTS.md")
BRIEF_KEYS = ("command", "file_path", "pattern")
BRIEF_WIDTH = 120
CHECK_NEEDLES = ("clippy::pedantic", "flags=(")
FMT_NEEDLES = ("+nightly fmt",)
RUN_GLOB = "*/*/r*"


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
        return bool(self.result)

    def skill_files_read(self) -> list[str]:
        """The skill files named in any tool call's input, sorted."""
        read = {
            name
            for call in self.tool_calls
            for name in SKILL_FILES
            if f"idiomatic-rust/{name}" in call.blob or f"/{name}" in call.blob
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


def run_dirs(results: Path) -> list[Path]:
    """Every results/<scenario>/<arm>/r<N> directory, in name order."""
    return sorted(path for path in results.glob(RUN_GLOB) if path.is_dir())


def print_summary(results: Path) -> None:
    """One line per run under results/, in scenario, arm, then run order."""
    print(
        f"{'scenario':14} {'arm':20} {'run':4} {'exit':4} {'wall':>5} {'turns':>5} {'cost':>6}  "
        "model, skill files read, check, fmt"
    )
    for run in run_dirs(results):
        path = run / "transcript.jsonl"
        if not path.is_file():
            continue
        scenario, arm = run.parts[-3:-1]
        transcript = parse_transcript(path)
        result = transcript.result
        models = [
            model for model in result.get("modelUsage", {}) if "haiku" not in model
        ]
        exit_word = "ok" if transcript.finished() else "RUN"
        wall = result.get("duration_ms", 0) // 1000
        turns = result.get("num_turns", "")
        cost = result.get("total_cost_usd", 0) or 0
        print(
            f"{scenario:14} {arm:20} {run.name:4} {exit_word:4} {wall:>5} {turns!s:>5} {cost:6.2f}  "
            f"{models} read={transcript.skill_files_read()} "
            f"check={transcript.ran(CHECK_NEEDLES)} fmt={transcript.ran(FMT_NEEDLES)}"
        )


def load_json(path: Path) -> dict:
    """The parsed file, or an empty dict when it is missing."""
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def passed(scores: list[dict], key: str, verdicts: tuple[str, ...]) -> str:
    """How many scores have one of `verdicts` under `key`, as `hits/total`."""
    if not scores:
        return "unscored"
    hits = sum(score.get(key, {}).get("verdict") in verdicts for score in scores)
    return f"{hits}/{len(scores)}"


def print_matrix(results: Path) -> None:
    """One Markdown row per scenario and arm, aggregated over its runs."""
    groups: dict[tuple[str, str], list[Path]] = defaultdict(list)
    for run in run_dirs(results):
        scenario, arm = run.parts[-3:-1]
        groups[(scenario, arm)].append(run)
    print(
        "| scenario | arm | runs | own tests | external | check lib | check all "
        "| check ran | mean wall s | mean cost $ | skill revision |"
    )
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for (scenario, arm), runs in sorted(groups.items()):
        metas = [
            meta for meta in (load_json(run / "meta.json") for run in runs) if meta
        ]
        scores = [
            score for score in (load_json(run / "score.json") for run in runs) if score
        ]
        if not metas:
            print(f"| {scenario} | {arm} | {len(runs)} | no meta.json | | | | | | | |")
            continue
        walls = [meta.get("wall_seconds", 0) for meta in metas]
        costs = [meta.get("total_cost_usd") or 0 for meta in metas]
        check_ran = sum(
            parse_transcript(run / "transcript.jsonl").ran(CHECK_NEEDLES)
            for run in runs
            if (run / "transcript.jsonl").is_file()
        )
        revisions = sorted(
            {
                (run / "skill-revision.txt").read_text(encoding="utf-8").strip()[:12]
                for run in runs
                if (run / "skill-revision.txt").is_file()
            }
        )
        print(
            f"| {scenario} | {arm} | {len(runs)} "
            f"| {passed(scores, 'tests', ('pass',))} "
            f"| {passed(scores, 'external', ('pass', 'none'))} "
            f"| {passed(scores, 'check_lib', ('clean',))} "
            f"| {passed(scores, 'check_all', ('clean',))} "
            f"| {check_ran}/{len(runs)} | {mean(walls):.0f} | {mean(costs):.2f} "
            f"| {', '.join(revisions) or 'none'} |"
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
        "skill_revision": args.skill_revision,
        "exit_status": args.status,
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

    for name, help_text in (
        ("summary", "one line per run under results/"),
        ("matrix", "one Markdown row per scenario and arm over its runs"),
    ):
        command = commands.add_parser(name, help=help_text)
        command.add_argument("results", nargs="?", type=Path, default=HERE / "results")

    meta = commands.add_parser(
        "meta", help="write final-message.md and meta.json for one run"
    )
    meta.add_argument(
        "out", type=Path, help="the run's results/<scenario>/<arm>/r<N> directory"
    )
    meta.add_argument("status", type=int, help="the claude process exit status")
    meta.add_argument("started", type=int, help="epoch seconds when the run started")
    meta.add_argument("finished", type=int, help="epoch seconds when the run finished")
    meta.add_argument("arm", help="bare, skill, or skill@<git-ref>")
    meta.add_argument("scenario", help="the scenario name")
    meta.add_argument("model", help="the model passed to run.sh, or 'default'")
    meta.add_argument(
        "skill_revision", help="the commit the skill arm loaded, or 'none' for bare"
    )

    args = parser.parse_args()
    if args.command == "summary":
        print_summary(args.results)
    elif args.command == "matrix":
        print_matrix(args.results)
    else:
        write_meta(args)


if __name__ == "__main__":
    main()
