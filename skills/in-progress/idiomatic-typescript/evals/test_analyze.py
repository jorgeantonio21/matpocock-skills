#!/usr/bin/env python3
"""Check the published matrix through the analysis CLI."""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class MatrixTests(unittest.TestCase):
    def external_cell(self, verdicts: list[str | None]) -> str:
        with tempfile.TemporaryDirectory(prefix="typescript-matrix-") as temp:
            results = Path(temp)
            for number, verdict in enumerate(verdicts, start=1):
                run = results / "scenario" / "bare" / f"r{number}"
                run.mkdir(parents=True)
                if verdict is not None:
                    score = {
                        "typecheck": {"verdict": "pass"},
                        "runtime": {"verdict": "pass"},
                        "external": {"verdict": verdict},
                        "diff": {"added": 0, "removed": 0},
                    }
                    (run / "score.json").write_text(json.dumps(score), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(HERE / "analyze.py"), "matrix", str(results)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            header, _, row = result.stdout.splitlines()
            cells = dict(zip(
                (cell.strip() for cell in header.split("|")),
                (cell.strip() for cell in row.split("|")),
            ))
            return cells["external"]

    def test_manual_review_has_no_automatic_acceptance_ratio(self) -> None:
        self.assertEqual(self.external_cell(["none"]), "n/a")

    def test_only_actual_passes_contribute_to_the_ratio(self) -> None:
        self.assertEqual(self.external_cell(["pass", "none", "fail", "incomplete"]), "1/4")

    def test_passing_automatic_suites_keep_their_ratio(self) -> None:
        self.assertEqual(self.external_cell(["pass", "pass"]), "2/2")

    def test_missing_scores_remain_unscored(self) -> None:
        self.assertEqual(self.external_cell([None]), "unscored")


if __name__ == "__main__":
    unittest.main()
