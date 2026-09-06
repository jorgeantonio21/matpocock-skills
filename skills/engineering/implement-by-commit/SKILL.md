---
name: implement-by-commit
description: Implement work as a planned sequence of human-reviewable commits, each gated on your approval before it lands.
disable-model-invocation: true
---

# Implement by Commit

Build the target (the spec, ticket, or PR the user names) as a planned sequence of **commits**, each one a diff a human reviews on its own. Where `/implement` builds the whole thing and commits at the end, this stops at a **gate** before every commit: you present the work, the user reviews it, and the commit lands only once they approve.

The user is the reviewer. Every commit you write is something they will read.

## Process

### 1. Read the target

Work from the spec, ticket, or issue passed as an argument: fetch it and read its full body and comments. With no argument, work from what is already in the conversation.

Explore the codebase before planning. Read `CONTEXT.md` if it exists so commit messages and interface names use the project's domain vocabulary, and respect ADRs in the area you're touching.

The baselines named below are in-progress prerequisites installed separately from the promoted plugin. Check that "pragmatic-programming" and the language baselines needed for this target are available before starting the build.

Before drafting the plan, call the Skill tool with "pragmatic-programming" and apply the sections relevant to planning and building.

### 2. Draft the commit plan

Break the target into an ordered sequence of commits.

<commit-rules>

- Each commit is **green** on its own: checked out at that commit, the branch typechecks and its tests pass.
- Each commit is **one idea**: the reviewer can say what it does in a single sentence, and the message writes itself from that sentence.
- Tests land in the commit whose behaviour they cover. There is no "add tests" commit.
- **Prefactor first.** A mechanical move, rename, or extraction gets its own commit ahead of the behaviour change that needs it, so the behaviour diff stays small. Make the change easy, then make the easy change.
- Order so nothing reaches forward: a commit never depends on code that lands later in the plan.
- The last commit completes the target. Nothing is left over.

</commit-rules>

Size each commit to a diff the reviewer reads in one sitting. When a commit's scope won't fit, split it.

### 3. Agree the plan

Present the plan as a numbered list, giving for each commit its title and the one sentence a reviewer would use to describe it. Ask the user whether the granularity is right, whether the order holds, and whether any commit should be split or merged.

Iterate until the user approves. Then write the approved plan to `.scratch/<feature-slug>/commits.md`:

<plan-template>

# Commit plan: <feature>

**Target:** the spec, ticket, or PR this sequence completes, as a path or link.

- [ ] 01 <title>: what a reviewer sees
- [ ] 02 <title>: what a reviewer sees

</plan-template>

### 4. Build one commit

Work the first unticked commit in the plan, and only that one.

Before writing or refactoring Rust or changing `Cargo.toml`, call the Skill tool with "idiomatic-rust". Before writing or refactoring TypeScript (including `.tsx`, `.mts`, and `.cts`) or changing its module or build configuration, call it with "idiomatic-typescript". Each skill is a separate call; load each once per context and follow its pointers as the work requires. Apply its rules within this commit's scope and preserve repository standards and existing contracts according to that baseline's precedence rules.

Use `/tdd` at pre-agreed seams. Typecheck and run the touched test files as you go, until both are clean.

Run the applicable language baseline's checks before presenting the gate.

### 5. Stop at the gate

Stop and present:

<gate-template>

**Commit NN: <title>**

**What this does:** the behaviour a reviewer sees, described from the outside.

**Files:** each file touched, one line each, saying what changed in it.

**Look closely at:** any decision you took that the plan didn't settle, or "nothing".

**Suggested message:**

```
<subject>

<body>
```

**Next:** NN+1, <title>

</gate-template>

The message is the user's own: a subject and a body, ending at the body. No trailer lines: no `Co-Authored-By`, no generated-with line, no signature of any kind.

Then wait. The user reads the working tree and replies:

- **Feedback** → fold it into this commit's work, then present the gate again. Feedback belongs to the commit in front of you, never to a follow-up commit.
- **Approval** → step 6.

`git commit` runs on approval and at no other point. Writing the next commit's code before this one lands is what turns a reviewable sequence back into one large diff.

### 6. Land it and take the next

Commit with the approved message. Tick the commit off in `commits.md`. Return to step 4 with the next unticked commit.

### 7. Close out

Once the last commit has landed, run the full test suite, then `/code-review` against the branch point to review the sequence as a whole.

## When the plan turns out wrong

The work teaches you what the plan couldn't know: a commit that won't go green alone, a dependency the order got backwards, a slice that is really two ideas. Stop there and take the revision to the user: say which commits change and why, agree the new shape, and update `commits.md` before building on.
