#!/usr/bin/env python3
"""Score completed trees; keep compiler failures distinct from lint findings."""

import argparse
from collections import Counter
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ARMS = ("bare", "merged", "skill")


def lint_flags(path):
    text = path.read_text().split("## The check command", 1)[1]
    block = re.search(r"^flags=\(\n(.*?)^\)$", text, re.M | re.S)
    if not block:
        raise ValueError(f"no flags block in {path}")
    flags = shlex.split(block[1], comments=True)
    if not flags:
        raise ValueError(f"empty flags block in {path}")
    return flags


def run(command, tree, log):
    try:
        result = subprocess.run(command, cwd=tree, capture_output=True, text=True)
    except OSError as error:
        log.write(f"{shlex.join(command)}\n{error}\n")
        return 127, ""
    log.write(f"{shlex.join(command)}\n{result.stdout}\n{result.stderr}\n")
    return result.returncode, result.stdout


def classify_lints(status, stdout):
    events = []
    for line in stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(event, dict):
            events.append(event)
    messages = [e["message"] for e in events if e.get("reason") == "compiler-message"]
    errors = [
        m for m in messages
        if m.get("level") == "error" and (
            re.fullmatch(r"E\d{4}", (m.get("code") or {}).get("code", ""))
            or (m.get("code") is None and not m.get("message", "").startswith("aborting due to"))
        )
    ]
    if errors:
        return {"status": "BUILD FAILED", "rustc_errors": len(errors), "exit_status": status}
    finished = [e for e in events if e.get("reason") == "build-finished"]
    findings = Counter(
        m["code"]["code"] for m in messages
        if m.get("code") and m.get("level") in ("warning", "error")
    )
    # Lints denied by -D warnings fail a completed build. Missing completion,
    # a signal, or a nonzero exit without error-level lint evidence is incomplete.
    denied_lints = any(m.get("level") == "error" and m.get("code") for m in messages)
    complete = bool(finished) and (
        (status == 0 and finished[-1].get("success") is True)
        or (status > 0 and finished[-1].get("success") is False and denied_lints)
    )
    if not complete:
        return {"status": "INCOMPLETE", "exit_status": status, "observed_lints": dict(findings)}
    return {"status": "FINDINGS" if findings else "CLEAN", "count": sum(findings.values()),
            "lints": dict(findings), "exit_status": status}


def tests(tree, log, target=None):
    command = ["cargo", "test", "--quiet"]
    if target is not None:
        command += ["--test", target]
    status, stdout = run(command, tree, log)
    passed = re.findall(r"test result: ok\. (\d+) passed", stdout)
    return {"status": "PASS" if status == 0 and passed else "FAIL" if status else "INCOMPLETE",
            "passed": sum(map(int, passed)), "exit_status": status}


def score_arm(scenario, arm, results):
    out = results / scenario.name / arm
    tree = out / "tree"
    if not (tree / "Cargo.toml").exists():
        return {"arm": arm, "status": "NO TREE"}
    extra = json.loads((scenario / "lints.json").read_text()) if (scenario / "lints.json").exists() else []
    flags = lint_flags(Path(os.environ.get("EVAL_LINTS_PATH", HERE.parent / "LINTS.md"))) + extra
    report = {"arm": arm, "scenario": scenario.name}
    with (out / "score.log").open("w") as log:
        report["tests"] = tests(tree, log)
        for name, target_flags, relax in (
            ("library", [], []),
            ("all_targets", ["--all-targets"], ["-A", "clippy::unwrap_used", "-A", "clippy::expect_used", "-A", "clippy::panic_in_result_fn"]),
        ):
            command = ["cargo", "clippy", "--no-deps", "--all-features", "--message-format=json"] + target_flags + ["--"] + flags + relax
            status, stdout = run(command, tree, log)
            report[name] = classify_lints(status, stdout)
        oracle = scenario / "oracle.rs"
        if oracle.exists():
            with tempfile.TemporaryDirectory(prefix="rust-oracle-") as temp:
                isolated = Path(temp) / "tree"
                shutil.copytree(tree, isolated, ignore=shutil.ignore_patterns("target", ".git"))
                (isolated / "tests").mkdir(exist_ok=True)
                target = isolated / "tests" / "eval_contract.rs"
                if target.exists():
                    raise ValueError("evaluation test name collides with generated tree")
                shutil.copyfile(oracle, target)
                report["independent_correctness"] = tests(isolated, log, "eval_contract")
        else:
            report["independent_correctness"] = {"status": "NOT MEASURED"}
    report["nonblank_rust_lines"] = sum(
        bool(line.strip()) for path in (tree / "src").rglob("*.rs") for line in path.read_text().splitlines()
    )
    (out / "score.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scenario")
    parser.add_argument("--results", type=Path, default=Path(os.environ.get("EVAL_RESULTS_ROOT", HERE / "results")))
    args = parser.parse_args()
    scenario = HERE / "scenarios" / args.scenario
    if not re.fullmatch(r"[a-z0-9-]+", args.scenario) or not scenario.is_dir():
        parser.error("unknown scenario")
    print("| arm | generated tests | independent correctness | library lints | all-target lints |")
    print("| --- | --- | --- | --- | --- |")
    for arm in ARMS:
        report = score_arm(scenario, arm, args.results)
        cells = [arm]
        for key in ("tests", "independent_correctness", "library", "all_targets"):
            result = report.get(key, {"status": report.get("status")})
            cells.append(result["status"] + (f" ({result['count']})" if "count" in result else ""))
        print("| " + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()
