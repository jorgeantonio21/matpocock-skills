## What it does

`implement-by-commit` builds work that has already been decided, the same as [implement](https://aihero.dev/skills-implement) does, but it lands that work as a planned sequence of commits rather than one. It plans the commits up front, agrees the plan with you, then builds them one at a time from first to last, and the last one completes the work.

No commit lands until you have read it. At the end of every commit it stops, shows you what it built and the message it suggests, and waits for you to approve before it runs `git commit`. That is what separates it from `implement`, which builds the whole [ticket](https://www.aihero.dev/ai-coding-dictionary/ticket) and commits at the end: your review moves from the end of the work to the end of each commit, while a wrong turn is still one commit deep instead of ten.

## When to reach for it

You invoke this by typing `/implement-by-commit`; the [agent](https://www.aihero.dev/ai-coding-dictionary/agent) won't reach for it on its own. It ships with `disable-model-invocation: true`, so no other skill can call it either.

It takes exactly what `implement` takes: a ticket, a [spec](https://www.aihero.dev/ai-coding-dictionary/spec), or the plan you just agreed in the conversation. Choosing between the two is a question about review, not about the work:

| You want… | Reach for |
| --- | --- |
| To read and approve every commit before it lands | `/implement-by-commit` |
| A history a reviewer can walk one commit at a time | `/implement-by-commit` |
| The work built and committed in one go | [implement](https://aihero.dev/skills-implement) |
| One concrete behaviour, test-first, with no spec | [tdd](https://aihero.dev/skills-tdd) |
| Work that is already built, checked | [code-review](https://aihero.dev/skills-code-review) |

The cost is your attention: a five-commit plan is five stops. On work nobody but you will read, that is a tax with no return, and `implement` is the cheaper skill.

## Prerequisites

Install `pragmatic-programming` and the language baselines the target needs before starting. These baselines are in-progress skills installed separately; the promoted plugin does not include them.

It commits to the branch you are on. It does not create one and does not ask, so check you are on the right branch before you start.

It writes the agreed plan to `.scratch/<feature-slug>/commits.md` and ticks each commit off as it lands, in the same `.scratch/<feature-slug>/` directory [to-tickets](https://aihero.dev/skills-to-tickets) writes local tickets into.

## The gate

The idea the skill runs on is the **gate**: the stop before every commit that only you can open. It presents four things (what the commit does from the outside, the files it touched, anything it decided that the plan did not settle, and a suggested message) and then stops.

What you do at the gate decides what happens next:

| You reply with… | It does |
| --- | --- |
| Feedback | Folds it into the commit in front of you, then presents the gate again |
| Approval | Commits with the message, ticks the plan, starts the next commit |

Feedback belongs to the current commit, never to a follow-up commit. That is the point of gating before the commit rather than after: the fix is still an edit, not a second commit apologising for the first.

## The commit plan

The plan is agreed before any code is written, and every commit in it clears the same bar: **green** on its own, so the branch typechecks and its tests pass at that commit; one idea, so a reviewer can say what it does in a sentence; and no forward references, so it never depends on code that lands later. Tests ship in the commit whose behaviour they cover, which means there is no "add tests" commit at the end. Mechanical moves and renames are prefactored into their own commit ahead of the behaviour change that needs them, so the interesting diff stays small.

The plan lives in `commits.md` rather than only in the [context window](https://www.aihero.dev/ai-coding-dictionary/context-window), which is what makes a long build survivable: the ticked list is the record of where you got to, so a [session](https://www.aihero.dev/ai-coding-dictionary/session) that ends halfway can be resumed from the file plus `git log` rather than reconstructed.

When the work teaches it the plan was wrong (a commit that will not go green alone, an order that turns out backwards) it stops and re-plans with you rather than quietly deviating.

## Common questions

**Which coding baselines does it use?**

- `pragmatic-programming` supplies the design and building principles.
- `idiomatic-rust` applies when writing or refactoring Rust or changing `Cargo.toml`.
- `idiomatic-typescript` applies when writing or refactoring TypeScript, including `.tsx`, `.mts`, and `.cts`, or changing TypeScript module or build configuration.

The agent loads the relevant rules before writing code and runs the language baseline's checks before presenting each gate. The rules apply within the agreed scope, with repository standards and existing contracts taking precedence as each baseline specifies.

**Isn't one commit per ticket the right size?**

That is the rule `implement` follows, and for a small ticket it is right. This skill exists for the ticket that is one session of work but more than one idea, a size `to-tickets` produces routinely, because it sizes tickets to fit a fresh context window rather than to fit a reviewer's attention. Those two bars are not the same, and this skill is for the gap between them. If your ticket really is one idea, the plan will have one commit in it and you have just used `implement` with an extra confirmation step.

**Can it open a pull request instead of committing?**

No. Like `implement`, it commits to the current branch, and there is no PR mode or configuration flag. What it does address is the underlying complaint behind that request: that code lands before anyone has verified it. Here nothing lands unverified, because you approve each commit first. If you want a PR at the end, open it yourself once the last commit is in.

**Will `code-review` actually see the changes this time?**

Yes. `code-review` reviews `git diff <fixed-point>...HEAD`, which excludes staged and working-tree changes, which is the reason it can come up empty when run before a commit exists. This skill runs it at close-out, after every commit in the plan has landed, so the whole sequence is in `HEAD` and inside the diff.

**Why does it never sign the commit message?**

Because the message is yours. It ends at the body: no `Co-Authored-By`, no generated-with line, no trailer of any kind. Some harnesses append one by default, which is precisely why the skill states it rather than leaving it to chance.

**What happens if I stop halfway through the plan?**

The landed commits are real work on your branch, and `commits.md` shows which ones they were. Nothing is left in a dirty worktree pretending to be finished, because the only uncommitted state that ever exists is the commit currently sitting at its gate. Pick it up later by reading the file and continuing from the first unticked line.

## It's working if

- The agent loads the applicable coding baselines and reports their checks before presenting each gate.

- The first thing it produces is a numbered list of commits, and it waits for you to approve the list before writing any code.
- `.scratch/<feature-slug>/commits.md` exists, and gains a tick each time a commit lands.
- It genuinely stops between commits: it presents a summary and a suggested message, then says nothing until you reply.
- Your feedback shows up as changes to the commit under review, not as an extra "address review comments" commit afterwards.
- `git log` afterwards reads as a sequence of single-idea commits, and you can check any one of them out and have the tests pass.
- No commit message carries a trailer line.

## Where it fits

`implement-by-commit` is a chain step: the build step of the main flow, standing in the same slot as `implement`:

```txt
grill-with-docs → to-spec → to-tickets → implement-by-commit → code-review
```

Its neighbours are [implement](https://aihero.dev/skills-implement), the sibling it swaps out and the one to prefer when nobody is reviewing the commits; [to-tickets](https://aihero.dev/skills-to-tickets), which produces the tickets it consumes; [tdd](https://aihero.dev/skills-tdd), which it drives inside each commit at the agreed seams; and [code-review](https://aihero.dev/skills-code-review), which it runs once at the close over the whole sequence.

Like `implement`, it trusts what it was handed and does not reopen the plan: it decides how the work lands in history, not what the work is.

[ask-matt](https://aihero.dev/skills-ask-matt) is the router over the whole set when you are not sure which flow you are in.
