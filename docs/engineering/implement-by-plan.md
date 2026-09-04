## What it does

`implement-by-plan` builds work that has already been decided, the same as [implement](https://aihero.dev/skills-implement) does, but it lands that work as a planned sequence of commits rather than one, and it agrees the sequence with you before writing a line of code.

The plan is the only thing you review. Once you approve it the build runs to the last commit without stopping, so everything you would otherwise catch commit by commit has to be caught in the plan: the slicing, the order, the open decisions, the branch it all lands on. That is what separates it from [implement-by-commit](https://aihero.dev/skills-implement-by-commit), which stops for your approval before every commit: same plan, same commits, one review point instead of ten.

## When to reach for it

You invoke this by typing `/implement-by-plan`; the [agent](https://www.aihero.dev/ai-coding-dictionary/agent) won't reach for it on its own. It ships with `disable-model-invocation: true`, so no other skill can call it either.

It takes what its siblings take: a [ticket](https://www.aihero.dev/ai-coding-dictionary/ticket), a [spec](https://www.aihero.dev/ai-coding-dictionary/spec), or the plan you just agreed in the conversation. All three build the same work, and picking between them is a question about your attention, not about the code:

| You want to review… | Reach for |
| --- | --- |
| The plan, then nothing until it's done | `/implement-by-plan` |
| Every commit before it lands | [implement-by-commit](https://aihero.dev/skills-implement-by-commit) |
| Nothing, just build it and commit | [implement](https://aihero.dev/skills-implement) |
| One concrete behaviour, test-first, with no spec | [tdd](https://aihero.dev/skills-tdd) |
| Work that is already built, checked | [code-review](https://aihero.dev/skills-code-review) |

Reach for it when you want a history a reviewer can walk one commit at a time, but you are not going to sit at ten gates to get it: you are stepping away while it builds, or the work is routine enough that the plan is the interesting part.

## Prerequisites

It commits to the branch you are on. It reads that branch back to you in the plan and asks you to confirm it, but it does not create one, so a sequence of commits arrives wherever you were standing when you started.

It writes the agreed plan to `.scratch/<feature-slug>/commits.md` and ticks each commit off as it lands, in the same `.scratch/<feature-slug>/` directory [to-tickets](https://aihero.dev/skills-to-tickets) writes local tickets into.

## The contract

The plan is a **contract**: agreed before the work starts, binding while it runs, and renegotiated openly rather than quietly. It is the whole of your involvement, so it carries more than a list of titles: it names the files each commit expects to touch, the decisions the ticket left open and which way the agent intends to go on them, and the branch the sequence lands on.

Every commit in it clears the same bar: **green** on its own, so the branch typechecks and its tests pass at that commit; one idea, so a reviewer can say what it does in a sentence; and no forward references, so it never depends on code landing later. Tests ship in the commit whose behaviour they cover, which means there is no "add tests" commit at the end. Mechanical moves and renames are prefactored into their own commit ahead of the behaviour change that needs them, so the interesting diff stays small.

Green-on-its-own does more work here than it does with gates in place. When nobody reads a diff before it lands, a clean checkout and `git bisect` are the entire safety net, and a commit that only goes green once the next one arrives takes that net away.

## Drift

**Drift** is the one thing that stops the run: the work proving the contract wrong. A commit that will not go green alone, an order that turns out backwards, a slice that is really two ideas, or a decision big enough that you would have wanted it at the plan.

When it hits drift it stops, says which commits change and why, and re-agrees the plan with you before building on. That is the only interrupt in the run, which is exactly why it matters: an unattended build that absorbs a surprise quietly is one that hands you a sequence you never approved.

## Common questions

**What stops it dropping the close-out at the end of a long run?**

This is the most-reported failure on `implement`: the [agent](https://www.aihero.dev/ai-coding-dictionary/agent) implements the code, then truncates before the review and commit steps, or asks "are we done?" instead of finishing. The cause is context burial: by the end of a long build the skill's closing steps are far back in the [context window](https://www.aihero.dev/ai-coding-dictionary/context-window) with nothing re-surfacing them.

`commits.md` is the structural answer rather than a sterner instruction. The remaining work lives in a file on disk, re-read at each commit, so "what happens next" is a lookup rather than a memory. It reduces the failure; it does not abolish it. The longer the plan, the further the close-out drifts from view, which is the honest argument for a plan sized to one [session](https://www.aihero.dev/ai-coding-dictionary/session) rather than a heroic twelve-commit run.

**How is this different from just using `/implement`?**

Both run start to finish without stopping. What you get here is the commit plan: a history someone can review one commit at a time afterwards, and a point before any code exists where you can move the slicing around. `/implement` gives you neither; it lands the ticket as one diff. If the resulting history does not matter to anyone, `/implement` is the cheaper skill and this one is ceremony.

**Can it work through several tickets in one run?**

No. It takes one target and builds it. Sequencing a whole feature directory (spawning a [subagent](https://www.aihero.dev/ai-coding-dictionary/subagent) per ticket, walking the blocking edges) is a different job that nothing here does yet. Work tickets one at a time, in dependency order, clearing context between them, exactly as you would with `/implement`.

**Can it open a pull request instead of committing?**

No. Like both its siblings, it commits to the current branch, and there is no PR mode. The branch line in the plan exists because that matters more here than it does with gates: nobody is watching while eight commits arrive. Confirm the branch when it asks, and open the PR yourself once the last commit is in.

## It's working if

- The first thing it produces is a numbered list of commits, and it writes no code until you approve the list.
- The plan tells you the branch it will commit to and the decisions it had to make, and you can overrule either while the code still doesn't exist.
- `.scratch/<feature-slug>/commits.md` exists, and gains a tick each time a commit lands.
- After you approve, it stays quiet. The only thing that brings it back to you is the plan no longer matching the work.
- `git log` afterwards reads as a sequence of single-idea commits, and you can check any one of them out and have the tests pass.
- It runs the full suite and `code-review` at the end without being reminded.
- No commit message carries a trailer line.

## Where it fits

`implement-by-plan` is a chain step: the build step of the main flow, standing in the same slot as `implement`:

```txt
grill-with-docs → to-spec → to-tickets → implement-by-plan → code-review
```

Its neighbours are [implement-by-commit](https://aihero.dev/skills-implement-by-commit), the same plan with a stop before every commit, and the one to prefer when the work lands in front of a reviewer; [implement](https://aihero.dev/skills-implement), which skips the plan entirely; [to-tickets](https://aihero.dev/skills-to-tickets), which produces the tickets it consumes; [tdd](https://aihero.dev/skills-tdd), which it drives inside each commit at the agreed seams; and [code-review](https://aihero.dev/skills-code-review), which it runs once at the close over the whole sequence.

Like both siblings, it trusts what it was handed and does not reopen the plan: it decides how the work lands in history, not what the work is.

[ask-matt](https://aihero.dev/skills-ask-matt) is the router over the whole set when you are not sure which flow you are in.
