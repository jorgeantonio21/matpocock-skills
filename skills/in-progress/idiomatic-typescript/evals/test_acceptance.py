#!/usr/bin/env python3
"""Exercise acceptance scoring with real candidate trees and the pinned compiler."""

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class AcceptanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="typescript-acceptance-")
        self.addCleanup(self.temp.cleanup)
        self.evals = Path(self.temp.name) / "evals"
        self.evals.mkdir()
        for name in ("score.sh", "materialize.sh"):
            shutil.copyfile(HERE / name, self.evals / name)
        shutil.copytree(HERE / "fixtures", self.evals / "fixtures")

    def candidate(self, scenario: str, variant: str) -> Path:
        self.scenario = scenario
        shutil.copytree(HERE / "scenarios" / scenario, self.evals / "scenarios" / scenario)
        self.run = self.evals / "results" / scenario / "bare" / "r1"
        tree = self.run / "tree"
        shutil.copytree(HERE / "fixtures" / "base", tree)
        shutil.copytree(HERE / "scenarios" / scenario / variant, tree, dirs_exist_ok=True)
        return tree

    def score(self) -> dict:
        result = subprocess.run(
            ["bash", str(self.evals / "score.sh"), self.scenario],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return json.loads((self.run / "score.json").read_text(encoding="utf-8"))

    def test_noop_package_scripts_cannot_pass_broken_code(self) -> None:
        tree = self.candidate("s1-untrusted-data", "start")
        package_path = tree / "package.json"
        package = json.loads(package_path.read_text(encoding="utf-8"))
        package["scripts"] = {name: "true" for name in package["scripts"]}
        package_path.write_text(json.dumps(package), encoding="utf-8")

        self.assertEqual(self.score()["external"]["verdict"], "fail")

    def test_candidate_config_cannot_skip_acceptance_type_checks(self) -> None:
        tree = self.candidate("s3-generic-api", "start")
        config_path = tree / "tsconfig.json"
        config = json.loads(config_path.read_text(encoding="utf-8"))
        config["compilerOptions"]["noCheck"] = True
        config["include"] = ["src/**/*.ts"]
        config["exclude"] = ["type-tests"]
        config_path.write_text(json.dumps(config), encoding="utf-8")

        self.assertEqual(self.score()["external"]["verdict"], "fail")

    def test_broken_runtime_package_export_fails_acceptance(self) -> None:
        tree = self.candidate("s5-package-consumers", "reference")
        package_path = tree / "package.json"
        package = json.loads(package_path.read_text(encoding="utf-8"))
        package["exports"]["."]["import"] = "./dist/missing.js"
        package_path.write_text(json.dumps(package), encoding="utf-8")

        self.assertEqual(self.score()["external"]["verdict"], "fail")

    def test_incompatible_exported_declarations_fail_acceptance(self) -> None:
        tree = self.candidate("s5-package-consumers", "reference")
        (tree / "src" / "incompatible.ts").write_text(
            "export function greet(name: string): number { return name.length; }\n",
            encoding="utf-8",
        )
        package_path = tree / "package.json"
        package = json.loads(package_path.read_text(encoding="utf-8"))
        package["exports"]["."]["types"] = "./dist/incompatible.d.ts"
        package_path.write_text(json.dumps(package), encoding="utf-8")

        self.assertEqual(self.score()["external"]["verdict"], "fail")

    def test_correct_package_passes_without_rewriting_the_candidate(self) -> None:
        tree = self.candidate("s5-package-consumers", "reference")
        package_path = tree / "package.json"
        package = json.loads(package_path.read_text(encoding="utf-8"))
        package["scripts"] = {name: "true" for name in package["scripts"]}
        original = json.dumps(package)
        package_path.write_text(original, encoding="utf-8")

        self.assertEqual(self.score()["external"]["verdict"], "pass")
        self.assertEqual(package_path.read_text(encoding="utf-8"), original)


if __name__ == "__main__":
    unittest.main()
