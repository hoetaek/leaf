---
name: leaf-idea
description: Use when capturing, parking, comparing, enriching, splitting, or lightly triaging a rough idea before committing to full leaf-work; trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, bundled ideas that may need separate seeds, or requests to decide whether an idea should die, defer, enrich, split, or become structured LEAF work.
---

# LEAF Idea

Use this skill for an idea-only pass before full `leaf-work`. The goal is to
preserve enough context that the idea can be resumed later without pretending it
is ready for criteria, wireframe, design, tasks, or execution.

An idea is exploration. It is allowed to die.

A seed is one possible future leaf, not an inbox. Before capturing or promoting,
decide whether the rough input is one work item, several independent future
leaves, or one unstable frame that needs a question before it can be split.

## Boundary

- Use `.leaf/01-seeds/<slug>/`; do not write loose planning files elsewhere.
- One seed should represent one possible `leaf-work` thread. Do not make one
  seed carry multiple independent outcomes that could become sibling leaves.
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
find .leaf/01-seeds .leaf/02-leaves -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If a likely matching seed already exists, resume it instead of creating a
duplicate. Use lowercase ASCII kebab-case slugs.

## Split Check

Run this check before creating a seed, during enrichment when the idea starts to
branch, and before recommending promotion.

Use `split now` when bundled parts have independent core nouns, deliverables,
success checks, reviewers/arbiters, lifecycles, likely-change axes, or
review/promote paths. If the user asked to capture the work and the split is
clear, create or resume sibling seeds; otherwise recommend `split` and name the
seed candidates.

Use `keep grouped` when parts are sequential concerns inside one outcome: one
stable core noun, one deliverable, one acceptance check, and one part naturally
feeds the next. For example, deciding presentation content and then placing that
content into slides can be one presentation seed.

Use `ask first` when splitting would decide the user's intent for them: the core
noun is drifting, the output form is still exploratory, or the quieter sibling
is not concrete enough to name. Ask one focused question instead of creating
several speculative seeds.

Do not promote a known mixed seed as one active leaf. Split it first, or record
the explicit reason it is one grouped outcome.

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

## Review Handoff

For assumptions, user-only knowledge, or blanks the user should fill, mark the
exact item with `USER REVIEW NEEDED:` or `ASSUMPTION:` and open the file in the
user's preferred editor when known (`cmux markdown`, `$VISUAL` / `$EDITOR`,
`code`, `vim` / `nvim`, Obsidian, Notepad, etc.). If no preference is known or
opening is unavailable, ask once or give the path and sections to review.

Use these status labels in `00-status.md`:

- `captured`: raw idea saved with minimal context
- `enriched`: meaningful context, references, or alternatives were added
- `ready-for-leaf-work`: enough context exists to start full gate work without
  rediscovering the basics
- `deferred`: intentionally parked with a resume condition
- `killed`: intentionally not worth pursuing now

Do not mark seeds as `fallen`. `fallen` is only for committed
`.leaf/02-leaves/<slug>/` work that is closed later.

## Triage

End every pass with one recommendation:

| Recommendation | Use when |
|---|---|
| `kill` | no problem, obligation, curiosity, or discomfort survives inspection |
| `defer` | the idea is real but not worth attention until a named condition changes |
| `enrich` | one or two cheap facts/examples would decide whether it has weight |
| `split` | several independent future leaves are bundled together and need separate seeds |
| `promote to leaf-work` | the user wants structured work and Gate 1 can start |

When recommending promotion, name the first missing `leaf-work` gate and the
one question or artifact that should start it. If Learn is already complete and
the next phase is Example, run `leaf promote <slug>` after explicit user
approval and continue from `.leaf/02-leaves/<slug>/`.

## Response Shape

Report briefly:

- seed path
- status label
- evidence checked
- what was captured or changed
- file or sections opened for review, if any
- recommendation and why, including split/group/ask-first reasoning when relevant
- next action, if the user resumes later
