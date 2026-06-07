---
name: leaf-fall
description: Use when closing, archiving, retiring, completing, abandoning, superseding, parking, splitting, or invalidating an active `.leaf/leaves/{slug}/` work item into `.leaf/fallen/{slug}/`; trigger on `leaf fall`, "close this leaf", "archive this leaf", "mark this leaf done", "stop carrying this leaf", or preserving prior LEAF work without wt or execution coupling.
---

# LEAF Fall

Close an active `.leaf/leaves/<slug>/` work item into `.leaf/fallen/<slug>/`.
Falling preserves prior work; it is archive, not trash.

## Boundary

- Work only inside `.leaf/`.
- Use `leaf fall <slug> --reason <reason>` for the lifecycle move.
- Do not read, write, infer, or validate `.wt/` files.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets, or
  execution artifacts.
- Do not close a leaf only because execution finished. The leaf remains active
  until review/sync and enough retrospective closure are recorded.
- Do not mix seed states with fallen leaves. `killed` or `deferred` belongs to
  `.leaf/seeds/<slug>/`; `fallen` belongs to committed `.leaf/leaves/<slug>/`.

## First Read

Inspect local truth before changing anything:

```bash
git status --short --branch
find .leaf/leaves .leaf/fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If `.leaf/leaves/<slug>/` is missing, stop. If `.leaf/fallen/<slug>/` already
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

Then enrich `.leaf/fallen/<slug>/00-status.md` when the context supports it:

- `closure summary`: what this leaf established or why it stopped
- `reusable lessons`: what future seeds/leaves should reuse
- `unresolved limits`: what remains unknown, weak, or out of scope
- `successor`: optional `.leaf/seeds/...` or `.leaf/leaves/...` path

If the leaf lacks `04-Feedback/10-retrospect.md`, create a concise one with the
same four fields plus any evidence that caused closure. Keep raw wording,
unknowns, criteria, wireframe, design, task graph, review notes, references, and
retrospective artifacts intact.

## Report

Report:

- fallen path: `.leaf/fallen/<slug>/`
- source path that no longer exists: `.leaf/leaves/<slug>/`
- fall reason
- closure fields filled or intentionally left blank
- preserved context files
- confirmation that no `.wt/` or execution artifacts were created
