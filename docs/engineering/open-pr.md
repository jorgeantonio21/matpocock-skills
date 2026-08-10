## What it does

`open-pr` ships the commits on your current branch as a pull request: it runs the repo's own checks, rescues commits stranded on `main` onto a properly named branch, drafts a title and body from the diff alone, and opens the PR with `gh`. It never picks the target itself — it asks, every run, which repo and base branch the PR lands on, and nothing outward-facing happens until you have approved the PR exactly as it will appear.

## When to reach for it

You invoke this by typing `/open-pr` — the [agent](https://www.aihero.dev/ai-coding-dictionary/agent) won't reach for it on its own.

Reach for it when the build is landed and the work ships as a pull request rather than a direct push. The build skills — [implement](https://aihero.dev/skills-implement), [implement-by-plan](https://aihero.dev/skills-implement-by-plan), [implement-by-commit](https://aihero.dev/skills-implement-by-commit) — all commit to the current branch and end there, with no PR mode; this is the step after any of them. To review a PR rather than open one, use [code-review](https://aihero.dev/skills-code-review) instead.

| Where you are standing | What it does |
| --- | --- |
| A feature branch with commits ahead of the base | Uses the branch as the PR's head |
| `main`, with commits that never should have landed there | Moves you onto a `<type>/ja/<slug>` branch at `HEAD`, and leaves `main` for you to reset |
| A branch with nothing ahead of the base | Stops — there is no PR to open |
| A dirty working tree | Stops — the PR ships commits, not the working directory |

## Prerequisites

The GitHub CLI (`gh`), authenticated against the host the repo lives on. Everything else it reads from the repo itself.

## The gate

The leading idea is the **gate**: one approval, with everything outward-facing behind it. Before the gate the skill only reads and drafts; at the gate you see the target repo and base, the head branch, the commit list, the title, and the body, and you can change any of them — including downgrading to a draft PR. After the gate it pushes and opens the PR in one motion. The run ends at the URL; the merge stays yours.

The body it drafts says what the diff does now — not the approaches discarded on the way, and not sold with filler adjectives. It ends at the last sentence: no trailer lines, no attribution footers.

## Common questions

**Why does it ask where the PR goes instead of just using `origin`?**

Because `gh` decides differently than you would: on a fork it defaults to the parent repo, so a PR you meant for your own `main` arrives on the upstream project's queue. The skill reads both candidates and makes the target an explicit answer, pinned with `--repo`, `--base`, and `--head` at create time.

**Can it merge the PR once it's open?**

No. The run ends at the URL. Merging is the reviewer's decision — even when the reviewer is you.

## It's working if

- It stops before pushing anything when the tree is dirty, the branch has nothing ahead of the base, or the repo's checks fail.
- Starting from `main` with stray commits, you end on a `<type>/ja/<slug>` branch and are told local `main` still needs resetting — and `main` itself is never pushed.
- You see the repo, base, title, and body before the PR exists anywhere outside your terminal.
- The PR body describes only what the diff does, and carries no trailer or attribution lines.
- The run ends with a URL, and nothing else has changed on GitHub.

## Where it fits

`open-pr` is the last chain step of the main flow — after a build skill has landed the commits and the review has been over them:

```txt
grill-with-docs → to-spec → to-tickets → implement → code-review → open-pr
```

Its neighbours are the three build skills, which produce the commits it ships and deliberately stop short of the PR, and [code-review](https://aihero.dev/skills-code-review), which reviews the sequence before it goes out. [ask-matt](https://aihero.dev/skills-ask-matt) is the router over the whole set when you are not sure which flow you are in.
