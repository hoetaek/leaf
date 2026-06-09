---
name: leaf-fall
description: Use when trashing, discarding, closing, retiring, completing, abandoning, superseding, parking, splitting, or invalidating an active `.leaf/02-leaves/{slug}/` work item into `.leaf/03-fallen/{slug}/`; trigger on `leaf fall`, "trash this leaf", "discard this leaf", "close this leaf", "mark this leaf done", or "stop carrying this leaf".
---

# LEAF Fall

Trash an active `.leaf/02-leaves/<slug>/` work item into `.leaf/03-fallen/<slug>/`.
Falling removes the leaf from the active carrying set; it is trash, not a
citation or long-term reference surface.

## Boundary

- Work only inside `.leaf/`.
- Use `leaf fall <slug> --reason <reason>` for the lifecycle move.
- Do not read, write, infer, or validate `.wt/` files.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets, or
  execution artifacts.
- Do not close a leaf only because execution finished. The leaf remains active
  until review/sync and enough retrospective closure are recorded.
- Do not mix seed states with fallen leaves. `killed` or `deferred` belongs to
  `.leaf/01-seeds/<slug>/`; `fallen` belongs to committed `.leaf/02-leaves/<slug>/`.

## Reference map

Conduct and reporting are shared across the LEAF family, not duplicated here.
**Invoke the `leaf-soul` skill with the Skill tool** before acting and follow it —
do not just read its file. When the work needs another LEAF skill, invoke that
skill rather than only referencing it. The sibling reference below backs it up:

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: shared conduct/voice, overview-first reporting, fact-vs-guess separation, review handoff, and preferred language — follow it for closure notes and reports |

## First Read

Inspect local truth before changing anything:

```bash
git status --short --branch
find .leaf/02-leaves .leaf/03-fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If `.leaf/02-leaves/<slug>/` is missing, stop. If `.leaf/03-fallen/<slug>/` already
exists, stop and do not merge or overwrite.

## Closure Check

Before falling, verify:

- the user explicitly wants this active leaf closed
- the fall reason can be named, such as `completed`, `abandoned`, `superseded`,
  `parked`, `split`, or `invalidated`
- `04-Feedback/09-review.md` or equivalent review/sync notes exist when the work
  reached execution or external review
- `04-Feedback/10-retrospect.md` exists, or you can write a minimal closure note
  before/after the CLI move
- any successor seed or leaf is linked only as a path; do not auto-create it

The CLI only writes flexible status slots. The AI should fill useful closure
content when context is available.

## Fall

Run:

```bash
leaf fall <slug> --reason "<reason>"
```

Then enrich `.leaf/03-fallen/<slug>/00-status.md` when the context supports it:

- `closure summary`: what this leaf established or why it stopped
- `reusable lessons`: what future seeds/leaves should reuse
- `unresolved limits`: what remains unknown, weak, or out of scope
- `successor`: optional `.leaf/01-seeds/...` or `.leaf/02-leaves/...` path

If the leaf lacks `04-Feedback/10-retrospect.md`, create a concise one with the
same four fields plus any evidence that caused closure. Keep raw wording,
unknowns, criteria, wireframe, design, task graph, review notes, references, and
retrospective artifacts intact.

## Report

Report per `../leaf-soul/SKILL.md` — overview-first, plain words, verified facts
separate from assumptions. Include:

- fallen path: `.leaf/03-fallen/<slug>/`
- source path that no longer exists: `.leaf/02-leaves/<slug>/`
- fall reason
- closure fields filled or intentionally left blank
- files left inspectable in fallen trash
- confirmation that no `.wt/` or execution artifacts were created
