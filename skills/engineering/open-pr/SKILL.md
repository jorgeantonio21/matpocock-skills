---
name: open-pr
description: Ship the commits on the current branch as a pull request, with one gate before anything outward-facing.
disable-model-invocation: true
---

# Open PR

Ship the commits on the current branch as a **pull request**. The build skills end at the last commit; this is the step after: checked before anything leaves the machine, described by what the diff does now, and gated once.

## Process

### 1. Pre-flight

Run `git fetch origin`, then read the state: the current branch, the commits ahead of the base, the working tree.

- A dirty tree stops the run: the PR ships commits, not the working directory. Say what is uncommitted and stop.
- Nothing ahead of the base stops it too: there is no PR to open.

Then run what the repo itself would run before code leaves the machine: `prek run` where a hook config exists, and the format/lint/test commands the repo's docs name. A PR that arrives red spends the reviewer it was opened for.

### 2. Get off main

Commits stranded on the default branch are the rescue case, not an error. If the current branch is `main`/`master` with commits ahead:

- Create `<type>/ja/<short-description>` at `HEAD` and switch to it, with `type` read from the commits themselves (`feat`, `fix`, `refactor`, `chore`, `docs`).
- Leave local `main` where it stands and tell the user it needs resetting to origin's; that reset is theirs to run.

Commits already pushed to a shared branch are history: build on them, never amend or rebase them.

### 3. Ask where it lands

Read the candidates (`origin`, and the parent repo where this is a fork, via `gh repo view --json parent`) and ask the user which repo and which base branch the PR targets. On a fork `gh` defaults to the parent, and that is a choice the user makes, not one to inherit. The answer is pinned at create time with explicit `--repo`, `--base`, and `--head`.

### 4. Draft what the diff says

- **Title**: imperative mood, ≤72 characters, in the repo's commit-subject convention.
- **Body**: what the code does now, and only what is in the diff. Plain, factual language: a bug fix is a bug fix. (Never: critical, crucial, essential, significant, comprehensive, robust, elegant.)
- The body is the whole message. It ends at the last sentence: no trailers, no attribution lines, no generated-with footers.

### 5. The gate

Present the PR exactly as it will appear: target repo and base, head branch, the commit list, title, body. Ready-for-review is the default; the user can make it a draft here. Iterate until approved; this is the only approval in the run, and nothing outward-facing happens before it.

### 6. Ship

Push the branch with `-u`, then `gh pr create` with `--repo`, `--base`, and `--head` explicit. If a PR already exists for this branch, report its URL rather than failing. End by reporting the PR's URL; the merge is the reviewer's, not yours.
