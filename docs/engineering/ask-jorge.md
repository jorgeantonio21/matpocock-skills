## What it does

`ask-jorge` is the router over the skills in this fork. You describe the situation you are in (an idea you cannot start, a pile of incoming bug reports, a [session](https://www.aihero.dev/ai-coding-dictionary/session) that has run long) and it names the skill or the sequence of skills that fits, plus where the human decisions in that sequence sit.

It recommends and stops. It does not grill, write a [spec](https://www.aihero.dev/ai-coding-dictionary/spec), open a file or fire the skill it just named; what you get back is the next thing to type, and you type it.

It exists beside [ask-matt](https://aihero.dev/skills-ask-matt): the same hand-written map, extended with the skills this fork adds on top of upstream. In the promoted set those are `implement-by-commit`, [implement-by-plan](https://aihero.dev/skills-implement-by-plan), and `open-pr`. In the in-progress bucket they are the two-agent `pair-by-commit` and `pair-by-plan` loops, the four-axis `full-review` they gate on, and the `pragmatic-programming` and `idiomatic-rust` baselines the agents behind them apply. The Rust reference also covers validation, admission, and compatibility guarantees while retaining its writing conventions. The fork keeps `ask-matt` itself byte-identical to upstream so merges never conflict on it, which makes `/ask-jorge` the one to type here: `ask-matt`'s map does not know the fork's skills exist.

## When to reach for it

You invoke this by typing `/ask-jorge`; the agent won't reach for it on its own.

| Your situation | What the router gives back |
| --- | --- |
| An idea, and no idea where to start | The head of the main flow, whether the build is small enough to skip the spec, how much of the build you want to review (nothing, the commit plan once, or every commit), and whether one agent builds it or an implementer and a reviewer pair up |
| A branch you want reviewed for bugs, not only standards | `full-review`, the four-axis review that proves each bug by running it, and which of its findings can block |
| Bugs and requests arriving from other people | The [triage](https://aihero.dev/skills-triage) on-ramp, and why [tickets](https://www.aihero.dev/ai-coding-dictionary/ticket) you generated yourself don't belong on it |
| Two skills that look interchangeable | The line between them, and it is usually one concrete test rather than a matter of taste |
| A long session and a decision about the [context](https://www.aihero.dev/ai-coding-dictionary/context) | The ordered tree over the five options at a phase boundary |
| A skill you have already picked | Nothing useful. Invoke that skill directly. |

## Prerequisites

The router names skills; it does not install them. Everything it points at has to be installed for the recommendation to be actionable. It knows the promoted skills in this fork plus the fork's own in-progress ones (the pair loops, `full-review`, and the two baselines), which the plugin does not ship: install those directly, and link the three agents behind the pair flow (`implementer`, `bug-hunter`, `craft-reviewer`) into the harness before the first run.

The tracker-dependent routes (triage, `to-spec`, `to-tickets`, `implement`) assume [setup-matt-pocock-skills](https://aihero.dev/skills-setup-matt-pocock-skills) has already configured an issue tracker in the repo. The router will happily recommend them before that has happened.

## Flows, not skills

The word the skill gives you to think with is **flow**: a path *through* the skills, not a single one. Naming your situation places you on a flow at a step, which is a different answer from "here is the skill that matches your keywords". Four kinds of route exist, and the skill itself carries them in full:

- **The main flow**, idea to ship. Grill, spec, tickets, implement (or implement-by-plan to agree the commit sequence up front, or implement-by-commit when a human reviews each commit, or the pair-by-plan and pair-by-commit versions of those two, where an implementer agent builds and full-review gates), review, then open-pr when the work ships as a pull request. Two branches sit inside it: a prototype detour when a question needs runnable code to settle, and the spec-and-tickets split, which only earns its cost when the build spans more than one session.
- **On-ramps**, for a situation that generates work and then merges onto the main flow: incoming bug reports, something broken, or an effort too foggy and too large to hold in one session.
- **Standalones**, off every flow, reached for on their own terms: the prototype, the questionnaire, the merge conflict you are already sitting in.
- **A reference layer underneath**: the two vocabulary references the other skills pull in when the words rather than the process are the problem, and the two rule baselines (pragmatic-programming, idiomatic-rust) the build and review agents judge a diff against.

## Common questions

**Why not just use `/ask-matt`?**

In this fork, `ask-matt` is deliberately frozen to upstream's version so that upstream merges never conflict on it, which means its map ends at upstream's skill set. Ask it about reviewing a build commit by commit, pairing an implementer agent with a reviewer, a review that proves its bugs by running them, or shipping a branch as a pull request, and it routes you to `/implement` or `/code-review` because it has never heard of the alternatives. `ask-jorge` is the same map with the fork's skills drawn in.

**It described a skill's behaviour, and the skill doesn't do that.**

A known failure inherited from `ask-matt`: the router answers from its own one-line summary of each skill rather than from the skill, and verifies only when pushed. When it asserts something load-bearing about another skill, ask it to open that `SKILL.md` first.

## It's working if

- It ends by naming what to type and stops there, instead of starting the work itself.
- The route it gives back mentions where to clear or compact context and where you are expected to review, not just a list of skill names.
- Asked about commit-by-commit review, a two-agent build, a review that proves its bugs, or opening a pull request, it names the fork's skills rather than routing around them.
- You recognise your own situation in what it hands back, rather than the nearest generic scenario.

## Where it fits

`ask-jorge` is a **standalone router** that sits over the whole set: upstream's skills plus the fork's additions. It is never a step in a chain; it points into every chain. From here you most often land on [grill-with-docs](https://aihero.dev/skills-grill-with-docs), the head of the main flow, or [triage](https://aihero.dev/skills-triage), the on-ramp for work that arrived rather than work you started.

It is a [secondary source](https://www.aihero.dev/ai-coding-dictionary/secondary-source) over the skills it describes. Where the router and a `SKILL.md` disagree, the `SKILL.md` is right. [ask-matt](https://aihero.dev/skills-ask-matt) remains in the fork as upstream's router, frozen; this page, not that one, describes the map you actually get here.
