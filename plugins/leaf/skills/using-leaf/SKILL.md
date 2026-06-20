---
name: using-leaf
description: Use when starting LEAF work or deciding which leaf skill applies — establishes the LEAF loop, routes to the right gate skill, and points conduct at soul. Injected at session start.
---

# Using LEAF

**Leaf before tree.** Don't let an agent grow the whole artifact up front — that
produces confident-looking slop and loses the way before you can tell the
direction is wrong. Validate one cheap, inspectable instance, then expand.

LEAF closes four kinds of uncertainty in order:

| Phase | What it makes you able to do |
|---|---|
| **Learn** | Judge what the work needs — learned, not guessed (① Intent · ② Unknowns & Context) |
| **Example** | Prove one cheap instance right before scaling (③ Criteria · ④ Wireframe) |
| **Architect** | Generalize that instance into a shippable generator (⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact) |
| **Feedback** | Confirm it still holds, then settle what was established (⑨ Review · ⑩ Retrospect) |

## Which skill to use

Invoke the relevant LEAF skill **before** doing substantial work — even a 1%
chance it applies means invoke it.

| Skill | Use it for |
|---|---|
| `soul` | **First, always.** Shared conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn` | Capturing/triaging an idea and running Learn (① Intent, ② Unknowns & Context) on a sprout |
| `work` | Carrying a sprout after Learn from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩ |
| `clean` | Cleaning a LEAF document into a simple current report before review or gate close |
| `done` | Deciding whether a finished leaf should stay, be pressed (citable digest), or fall |
| `profile` | Reading/updating the machine-global and repo-local LEAF profiles |

Process skills first (decide *how* to approach), then domain skills.

## The CLI is the body

The skills give the workflow its method; the `leaf` CLI gives it a repo-local
body — the `.leaf/` workspace (`leaf init`, `leaf new <slug>`, `leaf doctor`).
If `leaf` is not installed, install it before relying on the workspace commands
(`brew install hoetaek/tap/leaf`, or `cargo install --git https://github.com/hoetaek/leaf`).

For vague, early, or idea-stage work, start with `learn`. To build a sprout
that already passed Learn, use `work`.
