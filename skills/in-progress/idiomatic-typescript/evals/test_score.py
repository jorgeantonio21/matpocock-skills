#!/usr/bin/env python3
"""Pin score.sh pass, failure, and infrastructure verdicts with a stub npm."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCENARIO = "fixture"

NPM_STUB = """#!/usr/bin/env python3
import os
import sys
from pathlib import Path
with Path(os.environ["NPM_CALLS"]).open("a", encoding="utf-8") as calls:
    calls.write(" ".join(sys.argv[1:]) + "\\n")
command = " ".join(sys.argv[1:])
if command.startswith("ci"):
    key = "CI"
elif command == "run typecheck":
    key = "TYPE"
else:
    key = "TEST"
print(os.environ.get(f"{key}_OUTPUT", ""))
sys.exit(int(os.environ.get(f"{key}_STATUS", "0")))
"""

PACKAGE = {
    "name": "fixture",
    "version": "0.0.0",
    "private": True,
    "scripts": {"typecheck": "true", "test:runtime": "true", "test": "true"},
    "devDependencies": {},
}


def write_tree(path: Path) -> None:
    (path / "src").mkdir(parents=True)
    (path / "package.json").write_text(json.dumps(PACKAGE), encoding="utf-8")
    (path / "package-lock.json").write_text(json.dumps({"lockfileVersion": 3}), encoding="utf-8")
    node_version = subprocess.check_output(
        ["node", "-p", "process.versions.node"], text=True
    ).strip()
    (path / ".node-version").write_text(f"{node_version}\n", encoding="utf-8")
    (path / "src" / "index.ts").write_text("export const value = 1;\n", encoding="utf-8")


def scored(*, remove_package: bool = False, **environment: str) -> tuple[str, dict]:
    with tempfile.TemporaryDirectory(prefix="typescript-score-") as temp:
        root = Path(temp)
        evals = root / "evals"
        scenario = evals / "scenarios" / SCENARIO
        run = evals / "results" / SCENARIO / "bare" / "r1"
        base = evals / "fixtures" / "base"
        for path in (scenario / "start", run / "tree", base):
            write_tree(path)
        if remove_package:
            (run / "tree" / "package.json").unlink()
        for name in ("score.sh", "materialize.sh"):
            shutil.copyfile(HERE / name, evals / name)
        npm = root / "npm"
        npm.write_text(NPM_STUB, encoding="utf-8")
        npm.chmod(0o755)
        env = {
            **os.environ,
            "PATH": f"{root}{os.pathsep}{os.environ['PATH']}",
            "NPM_CALLS": str(root / "calls"),
            **environment,
        }
        result = subprocess.run(
            ["bash", str(evals / "score.sh"), SCENARIO],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise RuntimeError(f"score.sh failed:\n{result.stdout}\n{result.stderr}")
        return result.stdout, json.loads((run / "score.json").read_text(encoding="utf-8"))


class ScoreTests(unittest.TestCase):
    def test_clean_commands_pass(self) -> None:
        output, score = scored()
        self.assertNotIn("INCOMPLETE", output)
        self.assertEqual(score["typecheck"]["verdict"], "pass")
        self.assertEqual(score["runtime"]["verdict"], "pass")

    def test_type_diagnostic_is_a_failure(self) -> None:
        output, score = scored(TYPE_STATUS="2", TYPE_OUTPUT="src/index.ts(1,1): error TS2322: wrong")
        self.assertIn("FAIL (type diagnostic)", output)
        self.assertEqual(score["typecheck"]["verdict"], "fail")

    def test_runtime_assertion_is_a_failure(self) -> None:
        output, score = scored(TEST_STATUS="1", TEST_OUTPUT="not ok 1 - expected behavior")
        self.assertIn("FAIL (runtime test)", output)
        self.assertEqual(score["runtime"]["verdict"], "fail")

    def test_install_failure_is_incomplete(self) -> None:
        output, score = scored(CI_STATUS="1", CI_OUTPUT="registry unavailable")
        self.assertIn("INCOMPLETE (npm ci)", output)
        self.assertEqual(score["typecheck"]["verdict"], "incomplete")
        self.assertEqual(score["runtime"]["verdict"], "incomplete")

    def test_unexplained_failure_is_incomplete(self) -> None:
        output, score = scored(TYPE_STATUS="1", TYPE_OUTPUT="process stopped")
        self.assertIn("INCOMPLETE (exit 1", output)
        self.assertEqual(score["typecheck"]["verdict"], "incomplete")

    def test_missing_manifest_is_incomplete(self) -> None:
        output, score = scored(remove_package=True)
        self.assertIn("INCOMPLETE (no package.json)", output)
        self.assertEqual(score["typecheck"]["verdict"], "incomplete")
        self.assertEqual(score["external"]["verdict"], "incomplete")


if __name__ == "__main__":
    unittest.main()
