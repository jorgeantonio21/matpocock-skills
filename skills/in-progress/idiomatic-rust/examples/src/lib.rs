//! The complete form of every Rust block in the skill's guidance files.
//!
//! 1. `admission`, `aggregate`, `interpretation`, `intrinsic`, `migration`, `operation`,
//!    `raw_to_validated`, and `transition` are the eight patterns in `INVARIANTS.md`.
//! 2. `skill`, `runtime`, and `crates` hold the blocks in `SKILL.md`, `RUNTIME.md`, and
//!    `CRATES.md`, each inside the stub items it needs to compile.
//! 3. Each fenced block in those files is a verbatim excerpt of one module here, up to one
//!    uniform indentation. `evals/check.sh` builds this crate, runs its tests and the `LINTS.md`
//!    check, and fails when a block is not found in a module.
//! 4. To change an example, edit the module, then paste the excerpt back into its file.

pub mod admission;
pub mod aggregate;
pub mod crates;
pub mod interpretation;
pub mod intrinsic;
pub mod migration;
pub mod operation;
pub mod raw_to_validated;
pub mod runtime;
pub mod skill;
pub mod transition;
