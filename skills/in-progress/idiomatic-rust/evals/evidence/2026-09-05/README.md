# Decoder trial evidence

This directory publishes the completed bare trial, not a repeated model comparison. `bare-decoder-meta.json` records model, time, turns, reported cost, and hashes of local raw artifacts. The input hashes describe the actual pre-lockfile start tree. The resolved lockfile is included so local semantic checks are reproducible.

To reconstruct the output, copy `../../scenarios/s6-decoder/start/` to a disposable directory, apply `bare-decoder.patch` there with `git apply`, and replace its `Cargo.lock` with `bare-decoder.Cargo.lock`. `bare-decoder-meta.json` contains the expected SHA-256 of the resulting `src/lib.rs` and lockfile. Run `cargo +1.97.1 test --locked`; then copy `../../scenarios/s6-decoder/oracle.rs` into the reconstructed tree at `tests/eval_contract.rs` and run `cargo +1.97.1 test --locked --test eval_contract`.

`bare-decoder-score.json` records 14 generated tests and 3 independent tests passing. The lint findings are separate from correctness. `bare-decoder-toolchain.txt` identifies the compiler and CLI. `interrupted-runs.json` identifies three incomplete starts and leaves their costs unknown. Full transcripts and stderr stay at the local paths named in metadata; the source patch and semantic checks can be audited without them.
