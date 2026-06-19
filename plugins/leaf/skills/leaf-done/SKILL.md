---
name: leaf-done
description: |
  Use when a leaf is ending and the agent must decide with the user whether to
  keep it as-is, press it into a citable `pressed.md` digest, or move it to
  fallen as archived/non-reference work. Trigger after ⑩ Retrospect, on
  `leaf press`, "press this leaf", "make this leaf citable", "인용용으로 눌러줘",
  "fall this leaf", archiving, abandoning, superseding, parking, invalidating,
  or deciding whether completed work is worth carrying as a reference. Do not
  use for gate document cleanup or producing artifacts.
---

# LEAF Done

Close a leaf deliberately. The decision is not "done or not done"; it is what
kind of done this work deserves.

- **Keep** — the leaf remains in place with no citable digest yet.
- **Press** — the leaf is reference-worthy, so write `pressed.md` and a
  `## Press Abstract` in `00-status.md`.
- **Fall** — the work should stop being carried, or is completed but not worth
  future citation; move it to fallen with an explicit reason.

## Boundary

- Work only inside `.leaf/`.
- Run `leaf doctor` before and after close-out. If it reports a workspace issue,
  fix the diagnosed issue before trusting inventory output.
- Do not produce or revise the artifact itself.
- Do not clean gate prose except the small status/digest/closure text required
  by this close-out.
- Do not read, write, infer, or validate `.wt/` files.

## First Read

```bash
git status --short --branch
leaf doctor
leaf review <slug>
```

Resolve the source:

- Prefer an existing leaf when the slug exists in more than one stage.
- Use a sprout only when the user explicitly asks for provisional close-out.
- Use fallen only when the user explicitly asks to summarize archived material.
- If the source is ambiguous, ask which one is canonical.

## Decision

Recommend one option, then let the user decide when the choice is not already
explicit:

- Press when the work established reusable knowledge, a durable design decision,
  a pattern, a citable artifact, or a lesson future leaves should reuse.
- Keep when the work is still useful but not ready for citation, review, or
  retrospective closure.
- Fall when the work was abandoned, superseded, invalidated, split away, parked,
  or completed but not reference-worthy.

Name the reason in one sentence. Do not keep a leaf just because effort was
spent; keep it because future work should be able to cite it.

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

## Fall

Use fall when the item should stop being carried and should not become a
reference leaf. The reason must be explicit, such as `abandoned`, `superseded`,
`parked`, `split`, `invalidated`, `archived`, or
`completed-not-reference-worthy`.

Before falling, verify:

- the user wants it removed from the carried set, or the leaf itself records the
  close-out decision;
- it is not being kept as a reference-worthy leaf;
- the fallen reason is named;
- closure notes exist, or you can write a concise closure note from available
  context.

Run:

```bash
leaf fall <slug> --reason "<fallen reason>"
```

Then enrich the fallen status or retrospect only when source context supports
it:

- closure summary;
- reusable lessons;
- unresolved limits;
- successor, if one is evidenced or proposed.

## Keep

If the right decision is to keep the leaf without pressing or falling, update
only the minimal status note needed to make that decision visible. Do not create
`pressed.md`.

## Report

Report:

- decision: keep / press / fall;
- source path used;
- user decision or source evidence used;
- files written or moved;
- `linked.md` written or skipped;
- `leaf doctor` result;
- confirmation that no `.wt/` or execution artifacts were created.
