//! Independent tests for s8-cli. They drive the binary and check meaning: a version 1 file keeps
//! its meaning through validation and migration, version 2 rejects what it must, and a file that
//! does not validate is never migrated. The scorer copies this file into `tests/` of each result tree.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("cfgtool-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the scratch directory is created");
        Self { dir }
    }

    fn file(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, contents).expect("the input file is written");
        path
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn cfgtool(args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cfgtool"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn validate(path: &Path) -> Output {
    cfgtool(&[Path::new("validate"), path])
}

fn migrate(from: &Path, to: &Path) -> Output {
    cfgtool(&[Path::new("migrate"), from, to])
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn json(path: &Path) -> serde_json::Value {
    let text = fs::read_to_string(path).expect("the output file is readable");
    serde_json::from_str(&text).expect("the output file is JSON")
}

const V1_AUTO: &str = r#"{"version": 1, "workers": 0, "listen": "127.0.0.1:8080"}"#;
const V1_FIXED: &str = r#"{"version": 1, "workers": 4, "listen": "127.0.0.1:8080"}"#;

#[test]
fn test_validate_reports_a_version_1_zero_as_auto() {
    let scratch = Scratch::new("v1-auto");
    let output = validate(&scratch.file("v1.json", V1_AUTO));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("version 1"), "stdout names the version: {out}");
    assert!(
        out.contains("workers auto"),
        "stdout reports auto, not 0: {out}"
    );
}

#[test]
fn test_validate_reports_a_fixed_count() {
    let scratch = Scratch::new("v1-fixed");
    let output = validate(&scratch.file("v1.json", V1_FIXED));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("workers 4"),
        "stdout: {}",
        stdout(&output)
    );
}

#[test]
fn test_validate_accepts_version_2_auto_and_the_bounds() {
    let scratch = Scratch::new("v2-ok");
    for (name, contents, expected) in [
        (
            "auto.json",
            r#"{"version": 2, "workers": "auto", "listen": "127.0.0.1:1"}"#,
            "workers auto",
        ),
        (
            "one.json",
            r#"{"version": 2, "workers": 1, "listen": "127.0.0.1:1"}"#,
            "workers 1",
        ),
        (
            "max.json",
            r#"{"version": 2, "workers": 64, "listen": "127.0.0.1:1"}"#,
            "workers 64",
        ),
    ] {
        let output = validate(&scratch.file(name, contents));
        assert!(
            output.status.success(),
            "{name}: stderr: {}",
            stderr(&output)
        );
        let out = stdout(&output);
        assert!(
            out.contains("version 2") && out.contains(expected),
            "{name}: stdout: {out}"
        );
    }
}

#[test]
fn test_validate_rejects_what_version_2_forbids_and_names_the_file() {
    let scratch = Scratch::new("v2-bad");
    for (name, contents) in [
        (
            "zero.json",
            r#"{"version": 2, "workers": 0, "listen": "127.0.0.1:1"}"#,
        ),
        (
            "over.json",
            r#"{"version": 2, "workers": 65, "listen": "127.0.0.1:1"}"#,
        ),
        (
            "word.json",
            r#"{"version": 2, "workers": "many", "listen": "127.0.0.1:1"}"#,
        ),
        (
            "v3.json",
            r#"{"version": 3, "workers": "auto", "listen": "127.0.0.1:1"}"#,
        ),
        ("corrupt.json", r#"{"version": 2, "workers": "#),
    ] {
        let output = validate(&scratch.file(name, contents));
        assert_eq!(
            output.status.code(),
            Some(2),
            "{name}: exit code; stdout: {}",
            stdout(&output)
        );
        assert!(
            stderr(&output).contains(name),
            "{name}: stderr names the file: {}",
            stderr(&output)
        );
    }
}

#[test]
fn test_validate_of_a_missing_file_exits_2_and_names_it() {
    let scratch = Scratch::new("missing");
    let output = validate(&scratch.path("missing.json"));
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("missing.json"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn test_migrate_keeps_the_meaning_of_a_version_1_zero() {
    let scratch = Scratch::new("migrate-auto");
    let to = scratch.path("v2.json");
    let output = migrate(&scratch.file("v1.json", V1_AUTO), &to);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let migrated = json(&to);
    assert_eq!(migrated["version"], 2);
    assert_eq!(
        migrated["workers"], "auto",
        "zero meant auto, and auto it stays"
    );
    assert_eq!(
        migrated["listen"], "127.0.0.1:8080",
        "listen is carried over unchanged"
    );
    // the migrated file validates as version 2
    assert!(validate(&to).status.success());
}

#[test]
fn test_migrate_keeps_a_fixed_count() {
    let scratch = Scratch::new("migrate-fixed");
    let to = scratch.path("v2.json");
    assert!(
        migrate(&scratch.file("v1.json", V1_FIXED), &to)
            .status
            .success()
    );
    assert_eq!(json(&to)["workers"], 4);
}

#[test]
fn test_migrate_writes_nothing_for_a_file_that_does_not_validate() {
    let scratch = Scratch::new("migrate-bad");
    for (name, contents) in [
        ("corrupt.json", r#"{"version": 1, "workers": "#),
        (
            "zero-v2.json",
            r#"{"version": 2, "workers": 0, "listen": "127.0.0.1:1"}"#,
        ),
    ] {
        let to = scratch.path(&format!("{name}.out"));
        let output = migrate(&scratch.file(name, contents), &to);
        assert_eq!(output.status.code(), Some(2), "{name}: exit code");
        assert!(!to.exists(), "{name}: no output file is written");
    }
}

#[test]
fn test_migrate_of_a_version_2_file_keeps_it_as_it_is() {
    let scratch = Scratch::new("migrate-v2");
    let to = scratch.path("copy.json");
    let from = scratch.file(
        "v2.json",
        r#"{"version": 2, "workers": 8, "listen": "127.0.0.1:9"}"#,
    );
    assert!(migrate(&from, &to).status.success());
    let copy = json(&to);
    assert_eq!(copy["version"], 2);
    assert_eq!(copy["workers"], 8);
    assert_eq!(copy["listen"], "127.0.0.1:9");
}
