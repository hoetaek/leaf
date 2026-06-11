---
name: leaf-press
description: |
  Use when creating or refreshing a citable `pressed.md` digest inside a completed,
  reference-worthy LEAF folder. Trigger on `leaf press`, "press this leaf",
  "pressed markdown", "make this leaf citable", "인용용으로 눌러줘", "pressed 파일로
  정리", "중요 내용만 하나의 마크다운으로", or requests to summarize a LEAF item's
  intent, method, work done, limits, and lessons into one reusable Markdown file.
  Do not use for stage moves, execution, or wt artifacts.
---

# LEAF Press

Create a citable `pressed.md` digest inside the source leaf folder. Pressing does
not move, complete, fall, or execute the source; it writes a paper-style abstract
into the source `00-status.md` and compresses the reusable context into
`pressed.md` so later work can quote or cite it.

## Boundary

- Work only with `.leaf/` content.
- Write the digest to `<source leaf>/pressed.md`, not a shared pressed directory.
- Do not move sprouts, leaves, or fallen folders.
- On every press, update only the source `00-status.md` with `## Press Abstract`;
  do not change source gate files unless the user separately asks for source edits.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets, or
  execution artifacts.
- Do not present `pressed.md` as source truth. It is a digest with source links;
  the original LEAF files remain authoritative.
- Do not invent certainty. Mark missing, inferred, or unresolved points plainly.

## Reference Map

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: soul, reporting, fact-vs-assumption separation, user-language prose, review handoff |

## Source Resolution

Inspect local truth first:

```bash
git status --short --branch
find .leaf/02-leaves .leaf/01-sprouts .leaf/03-fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

Resolve the source for `{slug}`:

- Prefer `.leaf/02-leaves/{slug}/` when a completed reference-worthy leaf exists.
- Use a sprout only when the user explicitly asks for a provisional digest; name
  that it is not yet a completed leaf.
- Use a fallen source only when the user explicitly asks to summarize archived
  material; name that it is not a reference-worthy leaf.
- If several sources exist and the user did not specify one, state which source
  you will use and why.
- If no source exists, stop and list likely available slugs.

Write the digest to `<source>/pressed.md`. If it already exists, read it before
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
9. referenced materials under `01-Learn/02-references/`, only when the gate files
   point to them or the digest needs provenance

Missing files are acceptable. Note them in `pressed.md` only when their absence
affects interpretation.

## Status Abstract

Before writing `pressed.md`, write or refresh `## Press Abstract` in the source
`00-status.md`. This is a paper-style abstract for humans scanning the leaf, not
a stage change.

Use this shape after the status preamble:

```markdown
## Press Abstract

- pressed at: <local timestamp>
- pressed file: pressed.md

<One compact abstract-style paragraph: what the leaf tried to clarify, what
method it used, what it established or produced, and the main caveat or open
limit.>
```

If `## Press Abstract` already exists, replace that section rather than
appending a duplicate. Keep the existing status preamble and other sections
intact. If the source is too thin to support a useful abstract, write a short
abstract that says the source is insufficient and names the missing context. Use
the same core paragraph as `pressed.md`'s `Citation Summary`.

## Pressed Shape

Use this structure:

```markdown
# <human-readable title>

- source: <source path>
- pressed at: <local timestamp>
- citation handle: leaf:{slug}
- stage: <leaf / sprout / fallen if known>

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

Before writing `pressed.md`, separate:

- confirmed source facts
- agent inference from source facts
- unresolved or missing context

The digest must make those boundaries visible. If the source is too thin to
support a useful citation, create a short `pressed.md` that says so instead of
filling the gaps.

## After Press

Report per `../leaf-soul/SKILL.md`. Include:

- output path: `<source>/pressed.md`
- source path used
- `00-status.md` `## Press Abstract` written or refreshed
- whether this was a new digest or a refresh
- important missing source files or unresolved questions
- confirmation that no stage move, `.wt/`, or execution artifact was changed
