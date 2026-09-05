# s8-cli rubric

A small CLI whose stdout is its product, so `check-flags` relaxes `print_stdout` and `print_stderr` in both check passes. The prompt is a persistent representation change: version 2 is stricter, version 1 files must keep their meaning, and a corrupt file must never be migrated. `verify/tests/semantic.rs` drives the binary. `reference/` is one solution that passes it.

Mark each row `bare`, `skill`, `both`, or `neither`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Invariants: Persistent representation change | A version 1 zero loads as auto (an enum such as `Workers::Auto`), not as `1` and not as an error | | |
| 2 | Invariants: Persistent representation change | Version 2 rejects `0`, `65`, and an unknown word; the reason names the file | | |
| 3 | Invariants: Raw-to-validated boundary | The stored form (a version number and a number-or-word field) is parsed first, then the meaning is checked once for both versions; not two unrelated `Config` structs with duplicated checks | | |
| 4 | Shape: Name the absence | `workers` is an enum (`Auto` or a count), not `Option<u32>` or a `u32` with `0` still meaning auto inside the core | | |
| 5 | Shape: The compiler is the guardrail | The count is bounded once (`1..=64`) in a constructor or a `TryFrom`, not re-checked in `validate` and again in `migrate` | | |
| 6 | Errors: Library errors use `thiserror` | New `LoadError` variants carry the path and the rejected value; the message is what stderr prints | | |
| 7 | Errors: Propagate every error | `migrate` validates before it opens the output; a write error is reported, not swallowed | | |
| 8 | Flow: Match exhaustively | The command dispatch and the version dispatch name every case | | |
| 9 | Words: Tests read as sentences | A test per fixture, a test for the zero migration, a test for the corrupt input; the tests go through `load`, not only through the binary | | |
| 10 | Generality | No config crate, no CLI parsing crate, no async, no trait; `main` stays a dispatch over `env::args` | | |
| 11 | Correctness | `score.sh`: the external tests pass; the crate's own tests pass; `fixtures/` still loads | | |
| 12 | Check | `score.sh`: the check command reports zero findings in both runs, with the CLI relaxation | | |
