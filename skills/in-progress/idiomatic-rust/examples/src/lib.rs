//! The complete form of every example in `INVARIANTS.md`, one module per pattern.
//!
//! 1. Each fenced Rust block in `INVARIANTS.md` is a verbatim excerpt of one module here.
//! 2. `evals/check.sh` builds this crate, runs its tests and the `LINTS.md` check, and fails when
//!    a block in `INVARIANTS.md` is not found in a module.
//! 3. To change an example, edit the module, then paste the excerpt back into `INVARIANTS.md`.

pub mod admission;
pub mod aggregate;
pub mod interpretation;
pub mod intrinsic;
pub mod migration;
pub mod operation;
pub mod raw_to_validated;
pub mod transition;
