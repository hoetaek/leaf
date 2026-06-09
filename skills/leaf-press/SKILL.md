---
name: leaf-press
description: |
  Use when creating a citable pressed Markdown digest at `.leaf/04-pressed/{slug}.md`
  from an existing active leaf and writing a paper-style abstract into the
  source `00-status.md`. Trigger on `leaf press`, "press this leaf", "pressed
  markdown", "make this leaf citable", "인용용으로 눌러줘", "pressed 파일로 정리",
  "중요 내용만 하나의 마크다운으로", or requests to summarize a LEAF item's intent,
  method, work done, limits, and lessons into one reusable Markdown file. Use a
  seed or fallen source only when the user explicitly asks for that source. Do
  not use for promoting seeds to leaves, moving lifecycle state, executing tasks,
  or creating wt artifacts.
---

# LEAF Press

Create a citable Markdown digest from an active LEAF item. Pressing does not
promote, move, or execute the source; it writes a paper-style abstract into the
source `00-status.md` and extracts the important context into
`.leaf/04-pressed/{slug}.md` so later work can quote or cite it.

## Boundary

- Work only with `.leaf/` content.
- Do not move `.leaf/01-seeds/`, `.leaf/02-leaves/`, or `.leaf/03-fallen/` directories.
- On every press, update only the source `00-status.md` with `## Press Abstract`;
  do not change source gate files unless the user separately asks for source edits.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets, or
  execution artifacts.
- Do not present the pressed file as source truth. It is a digest with source
  links; the original LEAF files remain authoritative.
- Do not invent certainty. Mark missing, inferred, or unresolved points plainly.

## Reference map

Conduct and reporting are shared across the LEAF family, not duplicated here.
Read this sibling before writing the digest:

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: shared conduct/voice, overview-first reporting, fact-vs-guess separation, review handoff, and preferred language — follow it for the pressed file and final report |

## Source Resolution

Inspect local truth first:

```bash
git status --short --branch
find .leaf/01-seeds .leaf/02-leaves .leaf/03-fallen .leaf/04-pressed -maxdepth 1 -mindepth 1 2>/dev/null | sort
```

Resolve the source for `{slug}`:

- Prefer `.leaf/02-leaves/{slug}/` when an active leaf exists.
- Use `.leaf/03-fallen/{slug}/` only when the user explicitly asks to press a
  fallen/trash item.
- Use `.leaf/01-seeds/{slug}/` only when the user explicitly asks to press a
  seed.
- If several sources exist and the user did not specify one, state which source
  you will use and why.
- If neither exists, stop and list likely available slugs.

Create `.leaf/04-pressed/` when absent. Write the output to
`.leaf/04-pressed/{slug}.md`. If a pressed file already exists, read it before
replacing or refreshing it.

## Read Order

Read only the files needed to produce a faithful digest:

1. `00-status.md`
2. `01-Learn/01-intent.md`
3. `01-Learn/02-unknowns.md`
4. `02-Example/03-criteria.md`
5. `02-Example/04-wireframe.md`
6. `03-Architect/05-design.md`
7. `03-Architect/07-tasks.md`
8. `04-Feedback/` files, when present
9. referenced materials under `01-Learn/02-references/`, only when the gate
   files point to them or the digest needs provenance

Missing files are acceptable. Note them in the pressed file only when their
absence affects interpretation.

## Status Abstract

Before writing the pressed file, write or refresh `## Press Abstract` in the
source `00-status.md`. This is a paper-style abstract for humans scanning the
leaf, not a lifecycle state change.

Use this shape after the status preamble:

```markdown
## Press Abstract

- pressed at: <local timestamp>
- pressed file: .leaf/04-pressed/{slug}.md

<One compact abstract-style paragraph: what the leaf tried to clarify, what
method it used, what it established or produced, and the main caveat or open
limit.>
```

If `## Press Abstract` already exists, replace that section rather than
appending a duplicate. Keep the existing status preamble and other sections
intact. If the source is too thin to support a useful abstract, write a short
abstract that says the source is insufficient and names the missing context.
Use the same core paragraph as the pressed file's `Citation Summary`.

## Pressed File Shape

Use this structure:

```markdown
# <human-readable title>

- source: .leaf/<seeds-or-leaves>/{slug}
- pressed at: <local timestamp>
- citation handle: leaf:{slug}
- status: <state/current phase if known>

## Citation Summary

<One compact paragraph that can be quoted elsewhere. Say what this LEAF item was
trying to clarify, what approach it took, what it produced, and what caveat
matters most.>

## Intent

<The original purpose and why the work existed. Preserve important user wording
when it affects meaning.>

## Method

<How the work approached the problem: gates used, research path, examples,
criteria, comparisons, implementation strategy, or review loop.>

## What Was Done

<Concrete artifacts, decisions, explored alternatives, task graph, or outputs.>

## Key Points To Reuse

- <Reusable point, decision, pattern, or constraint>
- <Another point>

## Limits And Open Questions

- <Known limitation, unresolved question, weak evidence, or non-goal>

## Lessons

- <What was learned that should influence future work>

## Source Map

- `00-status.md`: <why it mattered>
- `01-Learn/01-intent.md`: <why it mattered>
- `<other source file>`: <why it mattered>
```

Keep the digest concise enough to quote. Prefer faithful compression over
coverage. Include direct quotations only when the original wording matters; keep
them short and source them in `Source Map`.

## Quality Bar

Before writing the pressed file, separate:

- confirmed source facts
- agent inference from source facts
- unresolved or missing context

The pressed file must make those boundaries visible. If the source is too thin
to support a useful citation, create a short pressed file that says so instead
of filling the gaps.

## After Press

Report per `../leaf-soul/SKILL.md` — overview-first, plain words, verified facts
separate from assumptions. Include:

- output path: `.leaf/04-pressed/{slug}.md`
- source path used
- `00-status.md` `## Press Abstract` written or refreshed
- whether this was a new pressed file or a refresh
- important missing source files or unresolved questions
- confirmation that no source lifecycle state transition, `.wt/`, or execution
  artifact was changed
