---
name: pragmatic-programming
description: "The Pragmatic Programmer's tips as a baseline for building and reviewing code. Use when designing, writing, or reviewing code and you want its principles applied (DRY, orthogonality, crash early, design by contract, easier to change, prove it), or when another skill needs the tip list."
---

# Pragmatic Programming

The tips from _The Pragmatic Programmer_ (Thomas and Hunt, 20th Anniversary Edition), cut down to the ones a diff can be judged against. Each entry names the official tip, then reads *what it looks like* → *the move*. Building, read the section for the work in hand before writing. Reviewing, match every entry against the diff and cite the tip on each finding; every entry is a labelled judgement call, never a hard violation.

Where an entry overlaps a Fowler smell from the `code-review` Standards baseline, the smell is bracketed. A review already running that baseline skips the bracketed entries here and reports the smell once, under Standards. A documented repo standard overrides any tip.

Tip numbers are the book's. Source: https://pragprog.com/tips/.

## Bugs

Where a diff goes wrong. Each is a place to **prove** something, by running it, rather than to suspect it.

- **Tip 34, Don't Assume It, Prove It.** Correctness resting on a claim ("the API never returns null", "this list arrives sorted"). → Find the evidence: a test with real and boundary data, or a runtime check. If neither exists, write the test that would fail.
- **Tip 37, Design with Contracts.** A new or changed function whose preconditions, postconditions, and invariants live only in the author's head. → State them in types, docs, or checks, and confirm every caller in the diff honours them.
- **Tip 38, Crash Early.** A failure caught and limped past: a swallowed exception, a default after a failed parse, an error logged and ignored. → Fail at the point of detection, with a message that names what went wrong and on what input.
- **Tip 39, Use Assertions to Prevent the Impossible.** An invariant assumed ("can't happen", non-empty, in range) rather than checked. → Assert it, side-effect free, where it is assumed.
- **Tip 40, Finish What You Start.** A resource (file, lock, connection, task, transaction) acquired on one path and not released on every exit, early returns and error branches included. → Release in the scope that acquired it, with a guard the exits cannot skip.
- **Tip 41, Act Locally.** A mutable or an open handle whose scope stretches well past its use, where a later edit could reorder it. → Narrow the scope to the lines that need it.
- **Tip 57, Shared State Is Incorrect State.** New mutable state reachable from more than one thread, task, or handler with no ownership story; a read-modify-write that is not atomic. → Give it one owner, or make the update atomic.
- **Tip 58, Random Failures Are Often Concurrency Issues.** A diff touching async, threads, timers, or retries that assumes a completion order, or a test that passes by timing. → Hunt the race; make the test control the order instead of waiting on it.
- **Tip 62, Don't Program by Coincidence.** Code that works because of undocumented ordering, defaults, or observed-but-unguaranteed library behaviour. → Name what it relies on, then make the reliance explicit or remove it.
- **Tip 70, Test Your Software, or Your Users Will.** Behaviour or an error path added without a test; a bug fix with no regression test. → Land the test in the same change.

## Craft

How the code reads and whether it is the code this codebase would write.

- **Tip 5, Don't Live with Broken Windows.** A hack added beside an existing hack, a TODO-guarded workaround, a known-bad pattern copied because it was already there. → Fix or fence it now; leave the area better than it was found.
- **Tip 15, DRY, Don't Repeat Yourself.** The same *knowledge* encoded twice: copied logic, a restated constant, a schema mirrored in validation and docs. → One representation, referenced from both places. [Duplicated Code; Repeated Switches when the copy is a match or switch.]
- **Tip 17, Eliminate Effects Between Unrelated Things.** A change that makes one module depend on another's internals, or a function now doing two unrelated jobs. → Split by reason to change; depend on the interface. [Divergent Change, Shotgun Surgery.]
- **Tip 22, Program Close to the Problem Domain.** Raw strings, ints, and tuples standing in for domain concepts; names taken from the mechanism rather than the domain. → Name the concept and give it a type. [Primitive Obsession, Data Clumps.]
- **Tip 45, Tell, Don't Ask.** A caller that pulls fields from an object, computes, and pushes results back. → Move the computation onto the object that owns the data. [Feature Envy.]
- **Tip 46, Don't Chain Method Calls.** A traversal through several objects' internals, coupling the caller to every intermediate type. → One method on the first object hides the walk. [Message Chains.]
- **Tip 47, Avoid Global Data.** A new global, singleton, or module-level mutable. → Pass it in; if it must be shared, put it behind an API (Tip 48).
- **Tip 51, Don't Pay Inheritance Tax.** Inheritance used only to share code. → Interfaces or traits for polymorphism (Tip 52), containment for reuse (Tip 53). [Refused Bequest when the subclass ignores what it inherits.]
- **Tip 69, Design to Test.** Code untestable without a live network, clock, or global; a hard-wired dependency; hidden I/O. → Accept the dependency at the interface and return results instead of producing effects.
- **Tip 74, Name Well; Rename When Needed.** A name describing mechanism instead of intent, or one that kept its identifier when the diff changed its behaviour. → Rename to what it means now. [Mysterious Name.]

## Design

Whether the shape will survive the next change.

- **Tip 14, Good Design Is Easier to Change Than Bad Design.** A structural choice where one shifted requirement would touch many places. → Prefer the shape that keeps that count small. *Easier to change* is the tie-breaker for every design call.
- **Tip 43, Avoid Fortune-Telling.** An abstraction, flag, or extension point with no current caller. → Delete it; inline until a second use appears. [Speculative Generality.]
- **Tip 44, Decoupled Code Is Easier to Change.** A new dependency between modules that did not know each other; one concept spread across many files. → Depend on an interface at a seam; gather the concept in one place. [Shotgun Surgery.]
- **Tip 49, Programming Is About Code, But Programs Are About Data.** Work modelled as objects mutated in place; state hoarded where it is not needed. → Model it as transformations over clearly shaped data, and pass state rather than hoarding it (Tip 50).
- **Tip 55, Parameterize Your App Using External Configuration.** An environment- or customer-specific value, or a policy, hardcoded. → Values to configuration, policy to data (Tip 79).
- **Tip 72, Keep It Simple and Minimize Attack Surfaces.** More complexity, broader input handling, new privileges, or a wider public surface than the feature needs. → Narrow to what the feature needs.

## Building

The three tips that shape how work lands, for whoever is writing rather than reviewing.

- **Tip 20, Use Tracer Bullets to Find the Target.** Each slice cuts end to end, thin, and runs; the next slice responds to what the last one taught.
- **Tip 42, Take Small Steps, Always.** One step, one check, then the next. A step too big to check is two steps.
- **Tip 8, Make Quality a Requirements Issue.** Good enough is a decision the user makes, not a default the code drifts to. Build to the agreed bar and stop there.
