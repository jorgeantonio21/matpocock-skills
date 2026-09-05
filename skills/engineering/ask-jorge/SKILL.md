---
name: ask-jorge
description: Ask which skill or flow fits your situation. A router over the skills in this fork.
disable-model-invocation: true
---

# Ask Jorge

You don't remember every skill, so ask.

A **flow** is a path through the skills. Most paths run along one **main flow**, and two **on-ramps** merge onto it. Everything else is standalone, or a reference layer (vocabulary and baselines) that runs underneath.

## The main flow: idea → ship

The route most work travels. You have an idea and want it built.

1. **`/grill-with-docs`**: sharpen the idea by interview. Start here whenever you are **working in a working directory**: it's stateful, retaining what it learns in `CONTEXT.md` and ADRs. (No working directory? Use `/grill-me`; see Standalone. Both run the same `/grilling` primitive; `grill-with-docs` is the one that leaves a paper trail, which makes it the better of the two whenever a repo is there to leave it in.)
2. **Branch: can you settle every question in conversation?** If a question needs a runnable answer (state, business logic, a UI you have to see), detour through a prototype, bridged by **`/handoff`** in both directions (a prototype lives in its own directory, which is exactly what `/handoff` is for; see Phase boundaries):
   - **`/handoff`** out, then open a fresh session against that file,
   - **`/prototype`** to answer the question with throwaway code,
   - **`/handoff`** back what you learned, and reference it from the original idea thread.
3. **Branch: is this a multi-session build?**
   - **Yes** → **`/to-spec`** (turn the thread into a spec), then **`/to-tickets`** to split it into tracer-bullet tickets, each declaring its **blocking edges**. On a local tracker that's one file per ticket under `.scratch/<feature>/issues/`, worked blockers-first by hand; on a real tracker the edges become native blocking links, so any ticket whose blockers are done can be grabbed. Kick off **`/implement`**, **`/implement-by-plan`**, or **`/implement-by-commit`** per ticket, **`/clear`ing context between each one**. Each ticket is self-contained, so the last one's context is disposable.
   - **No** → **`/implement`** right here, in the same context window, or **`/implement-by-plan`** to agree the commit sequence up front, or **`/implement-by-commit`** if a human will review each commit.

   Either way, the build skill drives **`/tdd`** internally, one red-green slice at a time, then closes out by running **`/code-review`**, a two-axis review (Standards + Spec) of the diff, before committing. **`/implement`** commits at the end of each ticket; **`/implement-by-plan`** agrees the commit sequence up front, then lands it unattended; **`/implement-by-commit`** pauses for approval before every commit. Reach for **`/tdd`** on its own when you just want to build a concrete behaviour test-first without a full spec, and **`/code-review`** on its own whenever you want to review a branch or PR against a fixed point.

   **`/full-review`** is the four-axis version of that review: Standards and Spec by running `/code-review` unchanged, plus **Bugs** proven by running them in a throwaway worktree and **Craft** with the idiomatic rewrite attached to each finding. It writes one verdict file, **blocked** or **clean**, with the blocking findings (a proven bug, a test that cannot fail, a spec miss, a breach of a documented standard) apart from the advisory ones, and it never ranks one axis against another. Reach for it on its own when you want a branch or PR reviewed for defects and idiom as well as standards and spec, and accept that it costs more agent time than `/code-review`. The pair skills below run it at every commit.

   **Branch: how much do you want to review?** `/implement` lands a ticket as one diff and commits at the end. **`/implement-by-commit`** plans the ticket as a sequence of commits instead, each green and reviewable on its own, and stops at a **gate** before every one: it presents the diff and a suggested message, you approve, it commits, then it takes the next. **`/implement-by-plan`** builds the same commit sequence with the gates removed: you review the plan once, before any code exists, then the run goes unattended to the last commit, stopping only if the work proves the plan wrong. Same `/tdd` inside and the same `/code-review` at the close, whichever you pick. Reach for `/implement-by-commit` when catching a wrong turn at commit two beats catching it at the end; for `/implement-by-plan` when you want a history a reviewer can walk commit by commit but you won't sit at the gates to get it; for `/implement` when the resulting history matters to nobody.

   **Branch: one agent or two?** All three of those build and review inside the window you are in. **`/pair-by-commit`** and **`/pair-by-plan`** run the same two commit loops with two agents on the job. An `implementer` agent builds each commit of the agreed plan, test-first, and lands it as a **provisional commit**: on the branch, at the top, amendable until its gate opens. **`/full-review`** then checks it on four axes, and every blocking finding goes back to the implementer for a fix and an amend, two rounds at most, before the commit reaches you. You orchestrate and write no code. `/pair-by-commit` keeps the gates of `/implement-by-commit`: you open every one, with the review beside the code and `git show HEAD` as the thing to read. `/pair-by-plan` makes a clean verdict the gate, as `/implement-by-plan` does: you agree the plan once, with the branch and every open decision in it, because nothing else gets a human eye until close-out, then the run goes unattended and stops only on **drift** or on a blocker that survives two rounds. Both close with a whole-sequence review, and neither applies an advisory finding without you. Reach for a pair skill when you want the bugs proven and the idiom judged before you look, and for the single-agent skill when the diff is small enough that your own read at the gate is the review.

   None of the five opens a pull request; each ends at commits on the branch you are on. When the work ships as a PR, **`/open-pr`** is the step after the build: it rescues commits stranded on `main` onto a properly named branch, asks which repo and base the PR targets, drafts the body from the diff alone, and stops at one **gate** before anything outward-facing happens.

### Context hygiene

Keep steps 1–3 in **one unbroken context window** (don't compact or clear until after `/to-tickets`) so the grilling, spec, and tickets all build on the same thinking. Each `/implement` then starts fresh, working from the ticket.

The limit on this is the **[smart zone](https://www.aihero.dev/ai-coding-dictionary/smart-zone)**: the window (~150k tokens on state-of-the-art models) within which the model still reasons sharply. If a session approaches it before `/to-tickets`, don't push on degraded; `/compact` at the nearest phase boundary and carry on (see Phase boundaries).

## On-ramps

A starting situation that generates work, then merges onto the main flow.

- **Bugs and requests piling up** → **`/triage`**. It moves issues through triage roles and produces agent-ready issues, which **`/implement`** later picks up.

  Triage is only for issues **you didn't create**: bug reports, incoming feature requests, anything that arrives raw. Tickets that `/to-tickets` produced are already agent-ready, so **don't triage them**.

- **Something's broken** → **`/diagnosing-bugs`**. For the hard ones: the bug that resists a first glance, the intermittent flake, the regression that crept in between two known-good states. It refuses to theorise until it has a **tight feedback loop** (one command that already goes red on *this* bug), then fixes with a regression test. Its post-mortem hands off to **`/improve-codebase-architecture`** when the real finding is that there's no good seam to lock the bug down.

- **A huge, foggy effort, a greenfield project or a huge feature build, too big for one session** → **`/wayfinder`**, the most cognitively demanding flow here. When the way from here to the destination isn't visible yet, it charts a **shared map** of **decision tickets** on the issue tracker and resolves them one at a time, producing **decisions, not deliverables**, until the fog is pushed back and the way is clear. Where **`/grill-with-docs`** sharpens an idea you can hold in one session, wayfinder is for the idea you can't, and it's slower and denser, so save it for exactly that, never a well-scoped feature.

  When the map clears, **it hands off, it doesn't build**: merge onto the main flow at **`/to-spec`**, which collapses the map's linked decisions into a buildable plan, then `/to-tickets` and `/implement` as usual. Looping the map straight into `/implement` skips that collapse and throws the linked detail away; go straight to `/implement` only when the effort turned out genuinely small.

## Codebase health

Not feature work: upkeep.

- **`/improve-codebase-architecture`**: run whenever you have a spare moment to keep the codebase good for agents to operate in. It surfaces **deepening opportunities**; picking one _generates an idea_ you can take into the main flow at `/grill-with-docs`. It's the survey that finds the candidates; **`/codebase-design`** (below) is the bench you design the chosen one on.

## Vocabulary underneath

Two model-invoked references that run *beneath* the other skills, each the single source of truth for its vocabulary. Reach for them directly when the **words**, not the process, are the problem; or let the skills above pull them in.

- **`/domain-modeling`**: sharpen the project's *domain* language: challenge a fuzzy term, resolve an overloaded word ("account" doing three jobs), record a hard-to-reverse decision as an ADR. It's the active discipline `/grill-with-docs` drives to keep `CONTEXT.md` a clean glossary.
- **`/codebase-design`**: the deep-module vocabulary (module, interface, depth, seam, adapter, leverage, locality) for designing a module's *shape*: a lot of behaviour behind a small interface at a clean seam. `/tdd` and `/improve-codebase-architecture` both speak it.

## Baselines underneath

Two more model-invoked references, each a list of rules a diff is judged against rather than a vocabulary. The `implementer`, `bug-hunter`, and `craft-reviewer` agents carry them, so `/full-review` and the pair skills apply them without being asked. Reach for one directly when you want its rules applied to the code in front of you, with no flow around it.

- **`/pragmatic-programming`**: the tips from _The Pragmatic Programmer_ cut down to the ones a diff can be judged against, in four sections: **Bugs** (prove it, crash early, contracts, resources, shared state), **Craft** (DRY, tell don't ask, design to test, naming), **Design** (easier to change, no fortune-telling, decoupling), and **Building** (tracer bullets, small steps). Every entry is a labelled judgement call, never a hard violation. Where a tip overlaps a `/code-review` smell, the smell is reported once, under Standards, so a review running both never says the same thing twice.
- **`/idiomatic-rust`**: the Rust rules a linter cannot enforce, each an instruction with its reason, in the order an author decides things: Shape, Errors, Ownership, Flow, Surface, Words. Beside it sit the invariant patterns (what a type can guarantee, what each pattern prevents, and what is enough instead), the clippy check command it runs on every crate a diff touches, the async and hot-path idiom, and a short optional set of crates that remove hand-written impls. The `implementer` and `craft-reviewer` agents read it on any diff that touches `.rs` or `Cargo.toml`. Reach for it yourself when you write, refactor, or review Rust with no pair flow around it, or when you decide what a type should guarantee. A documented standard in the repo overrides it.

## Phase boundaries

A **phase** is a chunk of work inside a session: the grilling, the implementation, the QA. At the **boundary** between two of them you have five options, and picking between them is the fuzziest decision in this whole map:

- **Continue**: stay put. Costs nothing, loses nothing.
- **`/clear`**: empty the window, when nothing here matters to what's next.
- **`/handoff`**: write a portable markdown file. Narrow: only for a **new harness**, a **new directory**, a **colleague**, or forking a side task **mid-phase**. What it buys is portability.
- **Subagent**: send a tightly-scoped task to its own window and get a report back.
- **`/compact`**: compress this context and seed a fresh session with it. The **default**, at the bottom of the tree rather than the first reach.

Read [PHASE-BOUNDARIES.md](PHASE-BOUNDARIES.md) for the ordered tree: the five questions, the reasoning behind each branch, and why the primary-source cost makes **Continue** the one to rule out first. Make the decision **at** a boundary; mid-phase, continue or split the rest into subagents.

## Standalone

Off the main flow entirely.

- **`/grill-me`**: the same relentless interview as `/grill-with-docs`, but **stateless**: it saves nothing locally and builds no `CONTEXT.md`. Reach for it when you are **not working in a working directory**: sharpening a plan, a design, a piece of writing, anything with no repo under it. If you are in a working directory, use `/grill-with-docs` instead: it runs the same interview and leaves a paper trail, so it is strictly the better one.
- **`/grilling`**: the interview primitive itself: rounds, the frontier, facts are the agent's job and decisions are yours. `/grill-me` and `/grill-with-docs` are the two named ways in, and `/triage`, `/wayfinder` and `/improve-codebase-architecture` all run it internally. Reach for it directly only when you want the interview with no wrapper around it.
- **`/resolving-merge-conflicts`**: work an in-progress merge or rebase conflict hunk by hunk, resolving by **intent** traced to each side's primary source rather than by picking lines, then finish the operation. It never runs `--abort`. Standalone and off every flow: reach for it when you are already mid-conflict.
- **`/prototype`**: a small, throwaway program that answers one design question: does this state model feel right, or what should this UI look like. Throwaway is a constraint on how the code is written, not a promise to destroy it: the answer folds into the real code, and the prototype itself is kept as a **primary source** on a `prototype/<name>` branch out of main, pointed at from the implementation issue. It's the detour in step 2 of the main flow, but reach for it any time a design question is hard to settle on paper.
- **`/research`**: delegate reading legwork to a **background agent**: it investigates a question against **primary sources**, then leaves a cited Markdown file in the repo. Keep working while it reads. The file it produces is something to take *into* the main flow at `/grill-with-docs`: research feeds the thinking, it doesn't replace it.
- **`/to-questionnaire`**: when the thing blocking you isn't in your head or the codebase but in **someone else's**, this writes them a questionnaire to fill in. It's the inverse of `/grill-me`: instead of interviewing you about the subject, it interviews you about the **send** (who it's going to, what you need back) and aims the questions at the gap. What comes back is material for `/grill-with-docs` or `/to-spec`.
- **`/wizard`**: for the steps only a **human** can take: provisioning infrastructure, setting up credentials or CI secrets, clicking through an unfamiliar third-party dashboard, running a one-off migration or cutover. It generates an interactive bash script that opens each URL, captures each value, and writes it into `.env` and GitHub secrets, so the procedure stops being something you re-explain to an agent every time. Model-invoked, so the agent reaches for it the moment it hits a wall only you can pass. If the agent could just do it itself, it should; this is for where a human is genuinely in the loop.
- **`/wait-what`**: the corrective for a message that didn't land. Use it mid-conversation, inside any other skill, and the agent re-pitches what it just said with the context you were missing, in plain English, using the `CONTEXT.md` vocabulary. It works after the fact; `/grill-with-docs` is the upfront cure, because a shared language agreed early is what stops the jargon arriving at all.
- **`/teach`**: learn a concept over multiple sessions, using the current directory as a stateful workspace.
- **`/writing-for-agents`**: reference for writing documents agents consume: skills, AGENTS.md, pointed-at docs.

## Precondition

**`/setup-matt-pocock-skills`**: run before your first engineering flow to configure the issue tracker, triage labels, and doc layout the other skills assume. Custom issue trackers also work.

**The agents behind the pair flow**: `/full-review`, `/pair-by-commit`, and `/pair-by-plan` spawn the `implementer`, `bug-hunter`, and `craft-reviewer` agents, which live beside the skills rather than inside them. Install them into the harness before the first run (in this fork, `scripts/link-agents.sh` symlinks them). Those three skills and the two baselines are **in progress**: public on purpose, installed directly rather than through the plugin, and free to change.
