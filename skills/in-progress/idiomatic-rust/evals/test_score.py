#!/usr/bin/env python3
"""Run score.sh against a stub cargo, so every verdict is pinned without a toolchain.

The fixture is one scenario, s4-async, with one run under results/. The stub cargo answers
`cargo test` with one passing test, and answers `cargo clippy` with the stdout, stderr, and exit
status the case sets. Each case asserts the Markdown row score.sh prints, the score.json it
writes, and, where it matters, the arguments the stub saw. check.sh runs this file; run it
directly to see a failing case.
"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import NamedTuple

HERE = Path(__file__).resolve().parent
SCENARIO = "s4-async"

CARGO_STUB = """\
#!/usr/bin/env python3
import json
import os
import sys
from pathlib import Path

with Path(os.environ["CARGO_CALLS"]).open("a", encoding="utf-8") as calls:
    calls.write(json.dumps(sys.argv[1:]) + "\\n")
if sys.argv[1] == "test":
    print("test result: ok. 1 passed; 0 failed")
    sys.exit(0)
print(os.environ["CARGO_STDOUT"])
print(os.environ["CARGO_STDERR"], file=sys.stderr)
sys.exit(int(os.environ["CARGO_STATUS"]))
"""

CARGO_TOML = """\
[package]
name = "pool"
version = "0.1.0"
edition = "2024"
"""

FINISHED_OK = {"reason": "build-finished", "success": True}
FINISHED_FAILED = {"reason": "build-finished", "success": False}


class Scored(NamedTuple):
    """What one score.sh run left behind."""

    output: str
    calls: list[list[str]]
    log: str
    json: dict


def compiler_message(level: str, code: str | None, message: str) -> dict:
    """One compiler-message event as cargo prints it, with its rendered text."""
    return {
        "reason": "compiler-message",
        "message": {
            "level": level,
            "code": None if code is None else {"code": code},
            "message": message,
            "rendered": f"{level}: {message}\n",
        },
    }


def lines(*events: dict) -> str:
    """The events as the one-object-per-line stream cargo writes."""
    return "\n".join(json.dumps(event) for event in events)


def write_crate(crate: Path) -> None:
    """The smallest crate the scorer's diff stats can read."""
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text(CARGO_TOML, encoding="utf-8")
    (crate / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")


def score(*, status: int = 0, stdout: str = "", stderr: str = "") -> Scored:
    """Run score.sh on the fixture with a stub cargo that answers clippy as given."""
    with tempfile.TemporaryDirectory(prefix="score-test-") as temp:
        root = Path(temp)
        evals = root / "evals"
        scenario = evals / "scenarios" / SCENARIO
        run = evals / "results" / SCENARIO / "bare" / "r1"
        scenario.mkdir(parents=True)
        for name in ("score.sh", "flags.sh"):
            shutil.copyfile(HERE / name, evals / name)
        shutil.copyfile(HERE.parent / "LINTS.md", root / "LINTS.md")
        shutil.copyfile(
            HERE / "scenarios" / SCENARIO / "check-flags", scenario / "check-flags"
        )
        write_crate(scenario / "start")
        write_crate(run / "tree")
        stub = root / "cargo"
        stub.write_text(CARGO_STUB, encoding="utf-8")
        stub.chmod(0o755)
        env = {
            **os.environ,
            "PATH": f"{root}{os.pathsep}{os.environ['PATH']}",
            "CARGO_CALLS": str(root / "calls"),
            "CARGO_STDOUT": stdout,
            "CARGO_STDERR": stderr,
            "CARGO_STATUS": str(status),
        }
        result = subprocess.run(
            ["bash", str(evals / "score.sh"), SCENARIO],
            env=env,
            capture_output=True,
            text=True,
            check=True,
        )
        calls = [
            json.loads(line)
            for line in (root / "calls").read_text(encoding="utf-8").splitlines()
        ]
        return Scored(
            output=result.stdout,
            calls=calls,
            log=(run / "score.log").read_text(encoding="utf-8"),
            json=json.loads((run / "score.json").read_text(encoding="utf-8")),
        )


class ScoreTests(unittest.TestCase):
    def assert_incomplete(self, scored: Scored) -> None:
        self.assertIn("INCOMPLETE", scored.output)
        self.assertNotIn("0 ()", scored.output)
        self.assertEqual(scored.json["check_lib"]["verdict"], "incomplete")
        self.assertEqual(scored.json["check_all"]["verdict"], "incomplete")

    def test_startup_failure_is_incomplete(self) -> None:
        scored = score(status=1, stderr="compiler could not start")
        self.assert_incomplete(scored)
        self.assertIn("compiler could not start", scored.log)

    def test_empty_success_is_incomplete(self) -> None:
        self.assert_incomplete(score())

    def test_invalid_json_is_incomplete(self) -> None:
        self.assert_incomplete(score(status=1, stdout='{"reason":'))

    def test_non_object_json_is_incomplete(self) -> None:
        self.assert_incomplete(score(stdout="[]"))

    def test_lint_without_build_finished_is_incomplete(self) -> None:
        event = compiler_message("error", "clippy::unwrap_used", "unwrap used")
        self.assert_incomplete(score(status=101, stdout=lines(event)))

    def test_success_with_a_nonzero_exit_is_incomplete(self) -> None:
        self.assert_incomplete(score(status=1, stdout=lines(FINISHED_OK)))

    def test_warning_cannot_mask_a_failed_build(self) -> None:
        event = compiler_message("warning", "unused_imports", "unused import")
        self.assert_incomplete(score(status=1, stdout=lines(event, FINISHED_FAILED)))

    def test_rustc_errors_remain_build_failures(self) -> None:
        for code in ("E0308", None):
            with self.subTest(code=code):
                event = compiler_message("error", code, "invalid program")
                scored = score(status=101, stdout=lines(event))
                self.assertIn("BUILD FAILED (rustc errors: 1)", scored.output)
                self.assertEqual(scored.json["check_lib"]["verdict"], "build_failed")

    def test_completed_lint_failure_is_counted(self) -> None:
        event = compiler_message("error", "clippy::unwrap_used", "unwrap used")
        scored = score(status=101, stdout=lines(event, FINISHED_FAILED))
        self.assertIn("1 (clippy::unwrap_used 1)", scored.output)
        self.assertNotIn("INCOMPLETE", scored.output)
        self.assertEqual(scored.json["check_lib"]["verdict"], "findings")
        self.assertIn("error: unwrap used", scored.log)

    def test_clean_build_relaxes_print_stdout_in_both_passes(self) -> None:
        scored = score(stdout=lines(FINISHED_OK))
        self.assertIn("0 ()", scored.output)
        self.assertEqual(scored.json["check_lib"]["verdict"], "clean")
        self.assertEqual(scored.json["check_all"]["verdict"], "clean")
        clippy = [call for call in scored.calls if call[0] == "clippy"]
        self.assertEqual(len(clippy), 2)
        for call in clippy:
            at = len(call) - 1 - call[::-1].index("clippy::print_stdout")
            self.assertEqual(call[at - 1], "-A")


if __name__ == "__main__":
    unittest.main()
