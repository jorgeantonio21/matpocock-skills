---
name: pair-by-plan
description: Agree a commit plan up front, then let an implementer agent build it and a four-axis reviewer gate every commit, unattended to the last one.
disable-model-invocation: true
---

# Pair by Plan

Build the target (the spec, ticket, or PR the user names) as a planned sequence of **commits**, agreed before any code is written and then landed without stopping, the way `/implement-by-plan` does, with two agents on the job instead of one. An **implementer** builds each commit. A **reviewer** checks it on four axes, and its verdict is the gate the user isn't there to open.

The plan is the **contract**. The user reviews it, approves it, and from that point the run goes unattended to the last commit, stopping only when a blocker survives the fix loop or the work proves the plan wrong.

The implementer is the `implementer` agent, spawned once and continued by message. The reviewer is the `full-review` skill: Standards, Spec, Bugs proven by running them, and Craft. You orchestrate: hold the plan, dispatch the work, apply the verdicts. You write no code yourself, and nothing outside `.scratch/`.

## Process

### 1. Read the target

Work from the spec, ticket, or issue passed as an argument: fetch it and read its full body and comments. With no argument, work from what is already in the conversation.

Explore the codebase before planning. Read `CONTEXT.md` if it exists so commit messages and interface names use the project's domain vocabulary, and respect ADRs in the area you're touching.

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

The user reviews this and nothing else until close-out, so put in front of them everything they would otherwise catch at a commit.

Present:

- The plan as a numbered list, giving for each commit its title, the one sentence a reviewer would use to describe it, and the files you expect it to touch.
- Every **open decision** the target left unsettled, and which way you intend to go. These land unreviewed by a human unless they are said here.
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

### 4. Build one commit

Spawn the `implementer` agent for the first unticked commit, or continue it by message for every commit after. Send **context pointers** only: the plan file, the commit number, the target. It reads the rest itself.

It is the only writer in the working tree, so wait for it and touch nothing while it works. It replies with a gate report and the short sha of a **provisional commit**: the commit is on the branch, at the top, amendable until its verdict is clean, and nothing below it is touched from here on.

If it reports **drift**, go to step 7.

### 5. Review it

Call the Skill tool with "full-review": fixed point `HEAD~1`; output path `.scratch/<feature-slug>/reviews/NN.md`; scope the plan file and commit NN; spec the target; previous reviews, the files already in that directory.

The verdict comes back **blocked** or **clean**.

### 6. The verdict is the gate

- **Clean** → tick the commit off in `commits.md` and return to step 4 with the next unticked commit. The ticked file, the review files, and `git log` are what make an interrupted run resumable.
- **Blocked** → message the implementer with the review file's path. It fixes each blocking finding, amends the provisional commit, and replies with what it did. Call the Skill tool with "full-review" again for a **targeted re-check**: the same fixed point, only the axes that had blocking findings, the previous verdict passed in. Two rounds at most. Clean after a round → tick and continue. Still blocked after the second → step 7.

Advisory findings never start a round and are never applied during the run. They stay in the review files for close-out.

### 7. Stop on drift

The contract holds until the work proves it wrong. **Drift** is anything that makes the agreed plan no longer the plan being built:

- A commit that won't go green on its own.
- An order that turns out backwards, with a commit needing code that lands later.
- A slice that is really two ideas, or two that are really one.
- A decision big enough that the user would have wanted it at step 3.
- A blocking finding that survives two fix rounds.

Stop at the point you find it. This is the only interrupt in the run, so use it rather than absorbing the surprise quietly: name the commit, point at the review file where there is one, say which commits change and why, agree the new shape, and update `commits.md` before building on. A provisional commit at the top is reshaped by amending, never discarded.

### 8. Close out

Once the last commit has landed, run the full test suite, then call the Skill tool with "full-review" against the branch point, over the whole sequence, output path `.scratch/<feature-slug>/reviews/final.md`. This is the user's first sight of the code, so report what it finds rather than only what was fixed: per axis, with every advisory finding that accumulated across the run, so the user can pick the ones worth a follow-up.

## One writer per tree

The implementer writes in the working tree. The review sub-agents read it. The bug hunter writes only inside its own throwaway worktree. You write under `.scratch/` and nowhere else.
