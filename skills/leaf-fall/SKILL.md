---
name: leaf-fall
description: Use when discarding, archiving, abandoning, superseding, parking, splitting, invalidating, or closing non-reference-worthy LEAF work into fallen with an explicit fallen reason.
---

# LEAF Fall

Move LEAF work into `fallen` when it should stop being carried and should not
become a reference-worthy leaf. Fallen is for discarded, archived, invalidated,
superseded, or completed-but-not-worth-reusing work.

## Boundary

- Work only inside `.leaf/`.
- Use `leaf fall <slug> --reason <fallen reason>` for the stage move when the CLI
  supports the source.
- Do not use fall as a generic "done" button. If a completed item is useful
  future reference, it belongs in `leaves`, optionally with `pressed.md`.
- Do not read, write, infer, or validate `.wt/` files.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets, or
  execution artifacts.
- Do not auto-create successor work. Link an existing or proposed successor path
  only when needed.

## Reference Map

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: soul, reporting, fact-vs-assumption separation, user-language prose, review handoff |

## First Read

Inspect local truth before changing anything:

```bash
git status --short --branch
find .leaf/01-sprouts .leaf/02-leaves .leaf/03-fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If the source is missing, stop. If a fallen folder for the slug already exists,
stop and do not merge or overwrite.

## Closure Check

Before falling, verify:

- the user explicitly wants this work removed from the carried set
- the work is not being kept as a reference-worthy leaf
- the fallen reason can be named, such as `abandoned`, `superseded`, `parked`,
  `split`, `invalidated`, `archived`, or `completed-not-reference-worthy`
- review/sync notes exist when the work reached execution or external review, or
  the reason they are unnecessary is stated
- `04-Feedback/10-retrospect.md` exists, or you can write a minimal closure note
  before or after the CLI move

The AI should fill useful closure content when context is available; do not make
the user reconstruct why the work fell.

## Fall

Run:

```bash
leaf fall <slug> --reason "<fallen reason>"
```

Then enrich the fallen `00-status.md` when context supports it:

- `fallen reason`: why this stopped or why it is not reference-worthy
- `closure summary`: what this work established, if anything
- `reusable lessons`: process lessons future sprouts/leaves should reuse
- `unresolved limits`: what remains unknown, weak, or out of scope
- `successor`: optional path or proposed slug

If the source lacks `04-Feedback/10-retrospect.md`, create a concise one with the
same fields plus any evidence that caused closure. Keep raw wording, unknowns,
criteria, wireframe, design, task graph, review notes, references, and
retrospective artifacts intact.

## Report

Report per `../leaf-soul/SKILL.md`. Include:

- fallen path
- source path that was moved
- fallen reason
- whether the work was incomplete, discarded, or completed-not-reference-worthy
- closure fields filled or intentionally left blank
- confirmation that no `.wt/` or execution artifacts were created
