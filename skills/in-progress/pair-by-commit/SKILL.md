---
name: pair-by-commit
description: Implement work as a planned sequence of commits, each built by an implementer agent and reviewed on four axes before it reaches your gate.
disable-model-invocation: true
---

# Pair by Commit

Build the target (the spec, ticket, or PR the user names) as a planned sequence of **commits**, the way `/implement-by-commit` does, with two agents on the job instead of one. An **implementer** builds each commit. A **reviewer** checks it on four axes before you see it. You still open every gate; what changes is that the work reaching the gate has already been reviewed, and the review sits in front of you beside the code.

The implementer is the `implementer` agent, spawned once and continued by message. The reviewer is the `full-review` skill: Standards, Spec, Bugs proven by running them, and Craft. You orchestrate: hold the plan, dispatch the work, present the gates. You write no code yourself, and nothing outside `.scratch/`.

## Process

### 1. Read the target

Work from the spec, ticket, or issue passed as an argument: fetch it and read its full body and comments. With no argument, work from what is already in the conversation.

Explore the codebase before planning. Read `CONTEXT.md` if it exists so commit messages and interface names use the project's domain vocabulary, and respect ADRs in the area you're touching.

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

Spawn the `implementer` agent for the first unticked commit, or continue it by message for every commit after. Send **context pointers** only: the plan file, the commit number, the target. It reads the rest itself.

It is the only writer in the working tree, so wait for it and touch nothing while it works. It replies with a gate report and the short sha of a **provisional commit**: the commit is on the branch, at the top, amendable until the gate opens, and nothing below it is touched from here on.

If it reports **drift** (a commit that won't go green alone, a slice that is really two ideas), go to _When the plan turns out wrong_.

### 5. Review it

Call the Skill tool with "full-review": fixed point `HEAD~1`; output path `.scratch/<feature-slug>/reviews/NN.md`; scope the plan file and commit NN; spec the target; previous reviews, the files already in that directory.

The verdict comes back **blocked** or **clean**.

### 6. Fix loop

Blocked: message the implementer with the review file's path. It fixes each blocking finding, amends the provisional commit, and replies with what it did. Then call the Skill tool with "full-review" again for a **targeted re-check**: the same fixed point, only the axes that had blocking findings, the previous verdict passed in.

Two rounds at most. A blocker still standing after the second re-check goes to the gate as unresolved, in the user's sight, and the decision is theirs.

Advisory findings never start a round. They wait for the gate.

### 7. Stop at the gate

Present, together:

<gate-template>

**Commit NN: <title>** (provisional, <short-sha>)

**Review:** clean, or the blocking findings unresolved after two rounds, listed with their axis.

**What this does:** from the implementer's report.

**Files:** from the implementer's report.

**Look closely at:** the implementer's open decisions, then the advisory findings worth the user's eye, each with its axis and the rewrite where Craft gave one.

**Message:** subject and body as committed.

**Next:** NN+1, <title>

</gate-template>

Then wait. The user reads `git show HEAD` and the review file, and replies:

- **Feedback**, including "apply advisory 2 and 3" → message the implementer with it. It amends. Present the gate again.
- **Approval** → step 8.

The provisional commit stays amendable until the user says it is done, and no next commit starts before then. Feedback belongs to the commit in front of you, never to a follow-up commit.

### 8. Land it and take the next

If the user changed the message, have the implementer amend it. Tick the commit off in `commits.md`. Return to step 4 with the next unticked commit.

### 9. Close out

Once the last commit has landed, run the full test suite, then call the Skill tool with "full-review" against the branch point, over the whole sequence, output path `.scratch/<feature-slug>/reviews/final.md`. Report what it finds, per axis, together with the advisory findings that accumulated across the run.

## When the plan turns out wrong

The work teaches you what the plan couldn't know: a commit that won't go green alone, a dependency the order got backwards, a slice that is really two ideas. Stop there and take the revision to the user: say which commits change and why, agree the new shape, and update `commits.md` before building on. A provisional commit at the top is reshaped by amending, never discarded.

## One writer per tree

The implementer writes in the working tree. The review sub-agents read it. The bug hunter writes only inside its own throwaway worktree. You write under `.scratch/` and nowhere else.
