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

## Invoke before you work

This is a forcing function, not a suggestion.

<EXTREMELY-IMPORTANT>
If there is even a 1% chance a leaf skill applies to what you are about to do,
you MUST invoke it BEFORE you respond or act. This is not negotiable. This is
not optional. You cannot rationalize your way out of it. Knowing the LEAF loop
is not running it — only the skill carries the gates.
</EXTREMELY-IMPORTANT>

**You are looking at LEAF work when you see:**

- a document, essay, memo, report, spec, study note, proposal, or presentation
  to produce or substantially revise
- an idea to capture, triage, or park ("save this", "maybe later", a brainstorm
  fragment, a half-formed curiosity)
- a task to understand, research, benchmark, or map a topic
- a build, prototype, or code task large enough to need design before scaling
- any substantial knowledge or build work carried in this repo

If it fits, route through the table below **before** you answer or act. It is
NOT LEAF work only when the reply is one or two sentences, a trivial mechanical
edit, or a direct lookup — then just answer.

**Red flags — these thoughts mean STOP, you are rationalizing past the skill:**

| Thought | Reality |
|---|---|
| "I'll just write the doc directly" | Producing the artifact IS what `work` governs. Invoke it first. |
| "I know the LEAF loop, I don't need the skill" | Knowing the loop ≠ running its gates. Invoke it. |
| "The CLI / `.leaf` isn't set up, so leaf doesn't apply" | The skills carry the method regardless — install per "The CLI is the body", don't skip the workflow. |
| "This is too small for the whole loop" | The loop scales down. Invoke it and let the gate be cheap; skipping is how slop starts. |
| "Let me gather context first, then decide" | `learn` IS how you gather context. Invoke it before exploring. |
| "It's a code task, not a leaf task" | `work` covers code and mixed deliverables, not just prose. |
| "The user asked a question, not for LEAF work" | A substantial knowledge task is leaf work. Check the boundary above before answering. |

## Which skill to use

| Skill | Use it for |
|---|---|
| `soul` | **First, always.** Shared conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn` | Capturing/triaging an idea and running Learn (① Intent, ② Unknowns & Context) on a sprout |
| `work` | Carrying a sprout after Learn from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩ |
| `polish` | Polishing a LEAF document into a simple current report before review or gate close |
| `press` | Pressing a reference-worthy leaf into a citable digest once press is the chosen close-out |
| `profile` | Reading/updating the machine-global and repo-local LEAF profiles |

Process skills first (decide *how* to approach), then domain skills.

## Ending a leaf

After ⑩ Retrospect, a leaf does not just stop — decide what kind of end it
deserves. Recommend one option, name the reason in one sentence, then let the
user confirm when the choice is not already explicit. Run `leaf doctor` before
and after close-out.

- **Keep** — still useful but not ready for citation. Update only the minimal
  `00-status.md` note that makes the keep decision visible. Do not create
  `pressed.md`.
- **Press** — reference-worthy: it established reusable knowledge, a durable
  decision, a pattern, a citable artifact, or a lesson future leaves reuse.
  Invoke the `press` skill to write the citable digest.
- **Fall** — the work should stop being carried, or is completed but not worth
  future citation. Move it to fallen with an explicit reason
  (`abandoned`, `superseded`, `parked`, `split`, `invalidated`, `archived`, or
  `completed-not-reference-worthy`):

  ```bash
  leaf fall <slug> --reason "<fallen reason>"
  ```

  Before falling, verify the user wants it removed from the carried set (or the
  leaf itself records the close-out decision), it is not being kept as
  reference-worthy, the reason is named, and a concise closure note exists or
  can be written. Then enrich the fallen status with a closure summary, reusable
  lessons, unresolved limits, and a successor only when source context supports
  it.

Do not keep or press a leaf just because effort was spent; carry it forward only
because future work should be able to cite it.

## The CLI is the body

The skills give the workflow its method; the `leaf` CLI gives it a repo-local
body — the `.leaf/` workspace (`leaf init`, `leaf new <slug>`, `leaf doctor`).
This plugin requires **`leaf` CLI ≥ 0.8.0**, and every workspace command WILL
FAIL until it is installed. If `leaf` is not on PATH, you MUST tell the user to
run the host's install entry before any LEAF work — do not silently skip it.
In Codex, use `$leaf:install` (enabled skills also appear in the slash command
list). In Claude-style slash command hosts, use `/leaf:install`. Where neither
entry is available, fall back to the platform installer: Homebrew on macOS, the
shell installer on Linux, or the PowerShell installer on Windows.

For vague, early, or idea-stage work, start with `learn`. To build a sprout
that already passed Learn, use `work`.
