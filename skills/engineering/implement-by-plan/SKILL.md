---
name: implement-by-plan
description: Agree a commit plan up front, then build and land the whole sequence in one unattended run.
disable-model-invocation: true
---

# Implement by Plan

Build the target (the spec, ticket, or PR the user names) as a planned sequence of **commits**, agreed before any code is written and then landed without stopping.

The plan is the **contract**. The user reviews it, approves it, and from that point the build runs unattended to the last commit. Where `/implement-by-commit` stops for approval before every commit, this spends the user's whole review budget in one place, up front, so the plan has to carry it.

## Process

### 1. Read the target

Work from the spec, ticket, or issue passed as an argument: fetch it and read its full body and comments. With no argument, work from what is already in the conversation.

Explore the codebase before planning. Read `CONTEXT.md` if it exists so commit messages and interface names use the project's domain vocabulary, and respect ADRs in the area you're touching.

The baselines named below are in-progress prerequisites installed separately from the promoted plugin. Check that "pragmatic-programming" and the language baselines needed for this target are available before starting the build.

Before drafting the plan, call the Skill tool with "pragmatic-programming" and apply the sections relevant to planning and building.

### 2. Draft the commit plan

Break the target into an ordered sequence of commits.

<commit-rules>

- Each commit is **green** on its own: checked out at that commit, the branch typechecks and its tests pass. Nobody reads these diffs before they land, so a clean checkout and `git bisect` are the whole safety net, and a commit that only goes green once the next one arrives takes that net away.
- Each commit is **one idea**: the reviewer can say what it does in a single sentence, and the message writes itself from that sentence.
- Tests land in the commit whose behaviour they cover. There is no "add tests" commit.
- **Prefactor first.** A mechanical move, rename, or extraction gets its own commit ahead of the behaviour change that needs it, so the behaviour diff stays small. Make the change easy, then make the easy change.
- Order so nothing reaches forward: a commit never depends on code that lands later in the plan.
- The last commit completes the target. Nothing is left over.

</commit-rules>

Size each commit to a diff the reviewer reads in one sitting. When a commit's scope won't fit, split it.

### 3. Agree the contract

The user reviews this and nothing else, so put in front of them everything they would otherwise catch at a commit.

Present:

- The plan as a numbered list, giving for each commit its title, the one sentence a reviewer would use to describe it, and the files you expect it to touch.
- Every **open decision** the target left unsettled, and which way you intend to go. These land unreviewed unless they are said here.
- The branch you will land on, read from `git branch --show-current`. A sequence of commits arrives on it unattended, so the user confirms it before the run starts.

Ask whether the granularity is right, whether the order holds, and whether any commit should be split or merged. Iterate until the user approves.

Then write the approved plan to `.scratch/<feature-slug>/commits.md`:

<plan-template>

# Commit plan: <feature>

**Target:** the spec, ticket, or PR this sequence completes, as a path or link.
**Branch:** the branch the sequence lands on.

- [ ] 01 <title>: what a reviewer sees
- [ ] 02 <title>: what a reviewer sees

</plan-template>

### 4. Build and land each commit

Work the commits in plan order, one at a time, using `/tdd` at the agreed seams. Finish the one in front of you before starting the next; writing the next commit's code early is what collapses a reviewable sequence back into one diff.

Before writing or refactoring Rust or changing `Cargo.toml`, call the Skill tool with "idiomatic-rust". Before writing or refactoring TypeScript (including `.tsx`, `.mts`, and `.cts`) or changing its module or build configuration, call it with "idiomatic-typescript". Each skill is a separate call; load each once per context and follow its pointers as the work requires. Apply its rules within this commit's scope and preserve repository standards and existing contracts according to that baseline's precedence rules.

Before each commit lands, run the check that would have been the user's:

- Typecheck clean.
- Run the applicable language baseline's checks and handle findings as that baseline specifies.
- The tests covering this commit's behaviour pass, and the full suite too, where the commit touches shared code.
- The diff is this commit's one idea and nothing else. Work belonging to a later commit stays for that commit.

Then commit, and tick the commit off in `commits.md` before starting the next. The ticked file plus `git log` is what makes an interrupted run resumable.

The message is the user's own: a subject and a body, ending at the body. No trailer lines: no `Co-Authored-By`, no generated-with line, no signature of any kind.

### 5. Stop on drift

The contract holds until the work proves it wrong. **Drift** is anything that makes the agreed plan no longer the plan you are building:

- A commit that won't go green on its own.
- An order that turns out backwards, with a commit needing code that lands later.
- A slice that is really two ideas, or two that are really one.
- A decision big enough that the user would have wanted it at step 3.

Stop at the point you find it. This is the only interrupt in the run, so use it rather than absorbing the surprise quietly: say which commits change and why, agree the new shape, and update `commits.md` before building on. A plan silently deviated from is a contract the user never agreed to.

### 6. Close out

Once the last commit has landed, run the full test suite, then `/code-review` against the branch point to review the sequence as a whole. This is the user's first sight of the code, so report what it finds rather than only what you fixed.
