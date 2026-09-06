#!/usr/bin/env python3
"""Verify the example consumer gate rejects an incompatible emitted declaration."""

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

EXAMPLES = Path(__file__).resolve().parent.parent / "examples"


class ConsumerTests(unittest.TestCase):
    def test_bundler_consumer_checks_emitted_declarations(self) -> None:
        with tempfile.TemporaryDirectory(prefix="typescript-consumer-") as temp:
            tree = Path(temp)
            shutil.copytree(
                EXAMPLES, tree, dirs_exist_ok=True,
                ignore=shutil.ignore_patterns("node_modules", "dist"),
            )
            for command in (["npm", "ci", "--ignore-scripts"], ["npm", "run", "build"]):
                result = subprocess.run(command, cwd=tree, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            command = ["node_modules/.bin/tsc", "-p", "consumers/bundler/tsconfig.json"]
            healthy = subprocess.run(command, cwd=tree, capture_output=True, text=True)
            self.assertEqual(healthy.returncode, 0, healthy.stdout + healthy.stderr)

            (tree / "dist" / "generics.d.ts").write_text(
                "export declare function indexBy(): void;\n", encoding="utf-8"
            )
            broken = subprocess.run(command, cwd=tree, capture_output=True, text=True)
            self.assertNotEqual(broken.returncode, 0, "consumer ignored the broken declaration")
            self.assertIn("TS2554", broken.stdout + broken.stderr)


if __name__ == "__main__":
    unittest.main()
