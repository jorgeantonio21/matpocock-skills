---
"mattpocock-skills": minor
---

Add **`idiomatic-rust`** (in-progress bucket, model-invoked): a reference skill for concise, idiomatic, readable Rust, written in Simplified Technical English. Six sections in the order an author decides things (Shape, Errors, Ownership, Flow, Surface, Words), each rule an instruction with its reason, with a snippet where the shape is not obvious from prose. The first rule makes the compiler the guardrail: one type per kind of value, so a swapped argument does not compile. Three disclosed references sit beside it: `LINTS.md` carries the clippy check command the agent runs on touched crates (pedantic and restriction lints as command-line flags, no repo change) and the rules it retires from prose; `RUNTIME.md` carries async and thread idiom; `CRATES.md` names the crates that retire hand-written impls, with verdicts and a skip list. The `implementer` and `craft-reviewer` agents point at it for Rust diffs.
