---
name: leaf-idea
description: Use when capturing, parking, comparing, enriching, or lightly triaging a rough idea before committing to full leaf-work; trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, or requests to decide whether an idea should die, defer, enrich, or become structured LEAF work.
---

# LEAF Idea

Use this skill for an idea-only pass before full `leaf-work`. The goal is to
preserve enough context that the idea can be resumed later without pretending it
is ready for criteria, wireframe, design, tasks, or execution.

An idea is exploration. It is allowed to die.

## Boundary

- Use `.leaf/seeds/<slug>/`; do not write loose planning files elsewhere.
- Create or resume a seed with the `leaf` CLI. Run `leaf init` first when
  `.leaf/` is absent, then `leaf new <slug>` unless the seed already exists.
- Work only in `00-status.md`, `01-Learn/01-intent.md`, and
  `01-Learn/02-unknowns.md` unless the user explicitly promotes to
  `leaf-work`.
- Do not fill Gate 3 Criteria, Gate 4 Wireframe, Gate 5 Design, or tasks from
  this skill. Mention the next gate only as a recommendation.
- If the user wants a real artifact, plan, draft, task graph, benchmark, or
  execution path now, switch to `leaf-work`.
- `leaf promote <slug>` is the boundary from seed to active leaf. Do not run it
  merely because an idea was captured; run it only when the user explicitly
  commits the work and the next LEAF move is after Learn.

## First Read

Inspect local truth before asking:

```bash
git status --short --branch
find .leaf/seeds .leaf/leaves -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If a likely matching seed already exists, resume it instead of creating a
duplicate. Use lowercase ASCII kebab-case slugs.

## Capture

Record a compact idea snapshot:

- raw user wording, preserving phrasing that may matter later
- current hunch: what this might become, stated as tentative
- why it surfaced: problem, obligation, curiosity, discomfort, or unknown
- possible output forms, if visible
- related seeds, leaves, docs, files, or prior decisions checked
- assumptions, risks, non-goals, and open questions

Write raw wording and current hunch in `01-Learn/01-intent.md`. Write unknowns,
known context, evidence checked, and open questions in `01-Learn/02-unknowns.md`.
Keep entries short; this is not a full Learn pass.

Use these status labels in `00-status.md`:

- `captured`: raw idea saved with minimal context
- `enriched`: meaningful context, references, or alternatives were added
- `ready-for-leaf-work`: enough context exists to start full gate work without
  rediscovering the basics
- `deferred`: intentionally parked with a resume condition
- `killed`: intentionally not worth pursuing now

Do not mark seeds as `fallen`. `fallen` is only for committed
`.leaf/leaves/<slug>/` work that is closed later.

## Triage

End every pass with one recommendation:

| Recommendation | Use when |
|---|---|
| `kill` | no problem, obligation, curiosity, or discomfort survives inspection |
| `defer` | the idea is real but not worth attention until a named condition changes |
| `enrich` | one or two cheap facts/examples would decide whether it has weight |
| `split` | several independent ideas are bundled together |
| `promote to leaf-work` | the user wants structured work and Gate 1 can start |

When recommending promotion, name the first missing `leaf-work` gate and the
one question or artifact that should start it. If Learn is already complete and
the next phase is Example, run `leaf promote <slug>` after explicit user
approval and continue from `.leaf/leaves/<slug>/`.

## Response Shape

Report briefly:

- seed path
- status label
- evidence checked
- what was captured or changed
- recommendation and why
- next action, if the user resumes later
