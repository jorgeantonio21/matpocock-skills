"""Regression tests for real scorer process and diagnostic failure modes."""

import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("score", Path(__file__).with_name("score.py"))
score = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(score)


def messages(code=None, level="error", finish=False):
    return "\n".join(json.dumps(event) for event in [
        {"reason": "compiler-message", "message": {"level": level, "message": "diagnostic", "code": {"code": code} if code else None}},
        {"reason": "build-finished", "success": finish},
    ])


class ScoringTests(unittest.TestCase):
    def test_startup_failure_with_only_stderr(self):
        with tempfile.TemporaryDirectory() as temp:
            cargo = Path(temp) / "cargo"
            cargo.write_text("#!/bin/sh\necho 'could not start compiler' >&2\nexit 1\n")
            cargo.chmod(0o755)
            log = io.StringIO()
            with patch.dict(os.environ, {"PATH": temp}):
                status, stdout = score.run(["cargo", "clippy"], temp, log)
            self.assertEqual(score.classify_lints(status, stdout)["status"], "INCOMPLETE")
            self.assertIn("could not start compiler", log.getvalue())

    def test_missing_executable(self):
        with patch.dict(os.environ, {"PATH": ""}):
            status, stdout = score.run(["cargo"], "/tmp", io.StringIO())
        self.assertEqual(score.classify_lints(status, stdout)["status"], "INCOMPLETE")

    def test_empty_success_is_not_clean(self):
        self.assertEqual(score.classify_lints(0, "")["status"], "INCOMPLETE")

    def test_rustc_error(self):
        for code in ("E0308", None):
            self.assertEqual(score.classify_lints(101, messages(code))["status"], "BUILD FAILED")

    def test_lint_denial_is_counted(self):
        result = score.classify_lints(101, messages("clippy::unwrap_used"))
        self.assertEqual((result["status"], result["count"]), ("FINDINGS", 1))

    def test_truncated_json_does_not_look_clean(self):
        self.assertEqual(score.classify_lints(1, '{"reason":')["status"], "INCOMPLETE")

    def test_warning_cannot_mask_infrastructure_failure(self):
        self.assertEqual(score.classify_lints(1, messages("unused_imports", "warning"))["status"], "INCOMPLETE")

    def test_success_requires_finished_build(self):
        self.assertEqual(score.classify_lints(0, '{"reason":"build-finished","success":true}')["status"], "CLEAN")

    def test_cli_exception_and_test_relaxations_reach_both_passes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            scenario = root / "cli"
            scenario.mkdir()
            (scenario / "lints.json").write_text('["-A", "clippy::print_stdout"]')
            tree = root / "results" / "cli" / "skill" / "tree"
            (tree / "src").mkdir(parents=True)
            (tree / "Cargo.toml").write_text("")
            calls = []

            def cargo(command, tree, log):
                calls.append(command)
                return (0, "test result: ok. 1 passed" if command[1] == "test" else '{"reason":"build-finished","success":true}')

            with patch.object(score, "run", side_effect=cargo):
                score.score_arm(scenario, "skill", root / "results")
            clippy = [call for call in calls if call[1] == "clippy"]
            self.assertEqual(len(clippy), 2)
            for command in clippy:
                index = len(command) - 1 - command[::-1].index("clippy::print_stdout")
                self.assertEqual(command[index - 1], "-A")
            for lint in ("unwrap_used", "expect_used", "panic_in_result_fn"):
                index = len(clippy[1]) - 1 - clippy[1][::-1].index(f"clippy::{lint}")
                self.assertEqual(clippy[1][index - 1], "-A")


if __name__ == "__main__":
    unittest.main()
