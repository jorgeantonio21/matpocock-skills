---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

The baselines named below are in-progress prerequisites installed separately from the promoted plugin. Check that "pragmatic-programming" and the language baselines needed for this target are available before starting the build.

Before implementation, call the Skill tool with "pragmatic-programming" and apply the sections relevant to the work.

Before writing or refactoring Rust or changing `Cargo.toml`, call the Skill tool with "idiomatic-rust". Before writing or refactoring TypeScript (including `.tsx`, `.mts`, and `.cts`) or changing its module or build configuration, call it with "idiomatic-typescript". Each skill is a separate call; load each once per context and follow its pointers as the work requires.

Use /tdd where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Run the applicable language baseline's checks before committing. Apply its rules within the requested scope and preserve repository standards and existing contracts according to that baseline's precedence rules.

Once done, use /code-review to review the work.

Commit your work to the current branch.
