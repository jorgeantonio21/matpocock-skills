# s6-decoder rubric

A decoder library. Three defects are planted in `start/src/lib.rs`, none named in the prompt: `Header::decode` builds `Priority(byte)` without a check, the three newtypes derive a plain `Deserialize` that skips `new`, and `Delta::invert` negates with `-` (a panic in debug, a wrap in release, on `i32::MIN`). `verify/tests/semantic.rs` catches each one through the public API. `reference/` is one solution that passes it.

Mark each row `bare`, `skill`, `both`, or `neither`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Shape: Close every route | `decode` checks the priority byte through `Priority::new` and reports it (a new `DecodeError` variant) | | |
| 2 | Shape: Close every route | `Priority`, `PayloadLen`, and `Delta` deserialize through `new`: `#[serde(try_from = "..")]` plus `TryFrom`, or a raw type; a plain derive is left on none of them | | |
| 3 | Invariants: Raw-to-validated boundary | `load_capture` parses the structure with serde and checks the meaning once, in a place the byte route shares; the payload length is checked against the header | | |
| 4 | Invariants: Invariant-preserving operation | `checked_invert` uses `checked_neg` (or the `i32::MIN` case explicitly); `magnitude` uses `unsigned_abs`; `invert` is removed, or routed through `checked_invert`, or documented as panicking on `i32::MIN` | | |
| 5 | Errors: Library errors use `thiserror` | `CaptureError` has a JSON variant with the source and a frame variant with the rejected value; no `String` message field | | |
| 6 | Errors: Compose errors | `#[from]` on the `serde_json::Error` variant; the frame check error converts with `?` | | |
| 7 | Flow: Guards first, happy path flat | `decode` and the frame check exit on each rejection at the top; no nested `if` ladder | | |
| 8 | Words: Tests read as sentences | One `test_` per rejection route (bytes, JSON) and per `Delta` limit; the round trip named; `i32::MIN` in a test | | |
| 9 | Generality | No trait for the decoder, no async, no new dependency, no typestate; the four newtypes stay `Copy` | | |
| 10 | Correctness | `score.sh`: the external tests pass; the crate's own tests pass | | |
| 11 | Check | `score.sh`: the check command reports zero findings in both runs | | |
