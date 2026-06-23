---
name: using-leaf
description: Use when starting LEAF work or deciding which leaf skill applies — establishes the LEAF loop, routes to the right gate skill, and points conduct at soul. Injected at session start.
---

# Using LEAF

**Leaf before tree.** Validate one cheap, inspectable instance before scaling —
growing the whole artifact up front produces confident-looking slop.

LEAF closes four kinds of uncertainty in order:

| Phase | What it makes you able to do |
|---|---|
| **Learn** | Judge what the work needs (① Intent · ② Unknowns & Context) |
| **Example** | Prove one cheap instance before scaling (③ Criteria · ④ Wireframe) |
| **Architect** | Generalize it into a shippable generator (⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact) |
| **Feedback** | Confirm it holds, then settle what was established (⑨ Review · ⑩ Retrospect) |

## Invoke before you work

<EXTREMELY-IMPORTANT>
If there is even a 1% chance a leaf skill applies, invoke it BEFORE you respond
or act. Knowing the loop is not running it — only the skill carries the gates.
</EXTREMELY-IMPORTANT>

It is LEAF work to produce or substantially revise a document; capture, triage,
or park an idea; research, benchmark, or map a topic; build or prototype
something large enough to need design; decide whether one work item should split
into several (→ `split`); or carry any substantial knowledge/build work in this
repo. It is **not** LEAF work when the reply is a sentence or two, a trivial
edit, or a direct lookup — then just answer.

## Which skill to use

| Skill | Use it for |
|---|---|
| `soul` | **First, always.** Conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn` | Capture/triage an idea and run Learn (① Intent, ② Unknowns & Context) |
| `split` | Decide whether/how to split one work item into separate leaves |
| `autopilot` | Run the gates automatically after the human-reviewed why/what/wireframe triple |
| `work` | Carry a sprout from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩ |
| `polish` | Make the cumulative document read as one connected report at each phase boundary |
| `press` | Press a reference-worthy leaf into a citable digest |
| `tend` | Sweep the pressed knowledge graph and reconcile drift with current code (banner/supersede) |
| `profile` | Read/update the machine-global and repo-local LEAF profiles |

Process skills first (decide *how*), then domain skills.

## Ending a leaf

After ⑩, `polish` the cumulative whole, then decide the end and let the user
confirm:

- **keep** — useful but not citable; note it in `00-status.md`.
- **press** — reference-worthy (reusable decision, pattern, lesson); invoke `press`.
- **fall** — stop carrying it: `leaf fall <slug> --reason "<abandoned|superseded|parked|split|invalidated|archived|completed-not-reference-worthy>"`.

Don't keep or press just because effort was spent.

## The CLI is the body

The `leaf` CLI gives the workflow a repo-local `.leaf/` body (`leaf init`,
`leaf new`, `leaf next`, `leaf doctor`); requires `leaf` ≥ 0.12.0. `leaf next`
crosses a phase boundary, pausing (멈칫) if the phase you are leaving still
carries its `<!-- leaf:polish-pending -->` marker — polish removes it. If `leaf`
is not on PATH,
tell the user to run the install entry (`$leaf:install` in Codex, `/leaf:install`
in Claude) before any LEAF work — don't skip it.

For early/idea work start with `learn`; to build a sprout that passed Learn use `work`.
