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
specimen — so future work can cite it without rereading the whole leaf.

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

- Prefer an existing leaf when the slug exists in more than one stage.
- Use a sprout only when the user explicitly asks for provisional press.
- Use fallen only when the user explicitly asks to summarize archived material.
- If the source is ambiguous, ask which one is canonical.

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
# <human-readable title>

- source: <source path>
- pressed at: <local timestamp>
- citation handle: leaf:{slug}
- stage: <leaf / sprout / fallen if known>

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
- Keep direct quotations short and only when original wording matters.
- If the source is thin, say so instead of filling gaps.
- Write `linked.md` only when source files evidence a relation to another leaf
  or the user names one. If no links exist, skip it.

## Report

Report:

- source path used;
- files written (`pressed.md`, `## Press Abstract`);
- `linked.md` written or skipped;
- `leaf doctor` result;
- confirmation that no `.wt/` or execution artifacts were created.
