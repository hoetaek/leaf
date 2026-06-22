---
name: press
description: |
  Use when a reference-worthy leaf should be pressed into a citable digest.
  Trigger on `leaf press`, "press this leaf", "make this leaf citable",
  "인용용으로 눌러줘", or "write the press abstract". Writes `pressed.md` and a
  `## Press Abstract` in `00-status.md` without touching source truth. The
  keep / press / fall decision and the fall and keep actions live in
  `using-leaf` ("Ending a leaf"), not here. Do not use for gate document
  cleanup or producing the artifact itself.
---

# LEAF Press

Press a reference-worthy leaf into a citable digest. Pressing condenses the leaf
into `pressed.md` — smaller in size, preserved in value, like a pressed
specimen — so future work can cite it without rereading the whole leaf. A
pressed digest is also the natural LEAF boundary for agent-readable knowledge:
new `pressed.md` files start with OKF-compatible YAML frontmatter so tools can
route, index, and link them as typed concepts without parsing the prose body.

Pressing is one outcome of ending a leaf. The keep / press / fall decision, and
the fall and keep actions, live in `using-leaf` ("Ending a leaf"). Enter this
skill once press is the chosen outcome.

## Boundary

- Work only inside `.leaf/`.
- Run `leaf doctor` before and after pressing. If it reports a workspace issue,
  fix the diagnosed issue before trusting inventory output.
- Do not produce or revise the artifact itself.
- Do not clean gate prose beyond the `## Press Abstract` this skill writes.
- Do not read, write, infer, or validate `.wt/` files.

## First Read

```bash
leaf doctor
leaf review <slug>
```

Resolve the source:

- Press only an existing `.leaf/02-leaves/<slug>/` leaf. A sprout is not
  pressable: finish ⑧, move it into leaves, and run Feedback first. Fallen work
  is not pressable because falling means it is not being kept as reusable
  reference knowledge.
- If the slug exists in more than one stage, ask which lifecycle state should be
  resolved before pressing.

## When To Press

Press only reference-worthy work: work that established reusable knowledge, a
durable design decision, a pattern, a citable artifact, or a lesson future
leaves should reuse. If the work is not that, it is a fall, not a press — return
to `using-leaf` ("Ending a leaf"). Do not press just because effort was spent;
press because future work should be able to cite it.

## Press

Create or refresh `<source>/pressed.md`. Also write or replace
`## Press Abstract` in `<source>/00-status.md`.

Use this `pressed.md` shape:

```markdown
---
type: Leaf Pressed Digest
title: <human-readable title>
description: <one-sentence summary for indexes and previews>
resource: <source path>
tags: [leaf, <short-topic-tag>]
timestamp: <ISO 8601 local timestamp>
citation_handle: leaf:{slug}
stage: leaf
---

# <human-readable title>

## Citation Summary

<One compact paragraph: what the leaf clarified, how it worked, what it
established or produced, and the main caveat.>

## Intent

<Why the work existed. Preserve user wording when it affects meaning.>

## Method

<Gates used, research path, criteria, comparisons, implementation strategy, or
review loop.>

## What Was Done

<Concrete artifacts, decisions, explored alternatives, task graph, or outputs.>

## Key Points To Reuse

- <Reusable point, decision, pattern, or constraint>

## Limits And Open Questions

- <Known limitation, unresolved question, weak evidence, or non-goal>

## Lessons

- <What should influence future work>

## Source Map

- `<source file>`: <why it mattered>
```

Rules:

- `pressed.md` is a digest, not source truth; original LEAF files remain
  authoritative.
- The frontmatter is for consumption and indexing. Keep `type` as
  `Leaf Pressed Digest`; set `resource` to the source LEAF path; keep
  `citation_handle` as a producer-defined field; set `stage` to `leaf`; choose
  a few short tags that help retrieval without inventing a broad taxonomy.
- Do not move active workflow state (`current gate`, `next action`, open review
  status) into pressed frontmatter. That state belongs in `00-status.md`; pressed
  frontmatter describes stable citable knowledge.
- Keep direct quotations short and only when original wording matters.
- If the source is thin, say so instead of filling gaps.
- Write `linked.md` only when source files evidence a relation to another leaf
  or the user names one. If no links exist, skip it.
- When writing `linked.md`, use exportable graph rows under `# Links`:

  ```markdown
  - `cites` -> `leaf:other-slug` - optional note
  ```

  Allowed predicates are `cites`, `refines`, `supersedes`, `depends_on`,
  `derived_from`, and `related_to`. Keep prose notes after the target; do not
  invent a broad ontology before real pressed leaves need it.

## Report

Report:

- source path used;
- files written (`pressed.md`, `## Press Abstract`);
- `linked.md` written or skipped;
- `leaf doctor` result;
- confirmation that no `.wt/` or execution artifacts were created.
