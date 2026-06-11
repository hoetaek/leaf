---
name: leaf-clean
description: |
  Use when tending the `.leaf/` workspace itself rather than the content of any
  leaf: pressing a completed, reference-worthy leaf into a citable `pressed.md`
  digest; moving non-reference-worthy work into fallen with an explicit fallen
  reason; or migrating an old workspace layout reported by `leaf doctor`.
  Trigger on `leaf press`, "press this leaf", "make this leaf citable",
  "인용용으로 눌러줘", "중요 내용만 하나의 마크다운으로", linking or citing
  between leaves ("leaf 간 인용", `linked.md`), discarding, archiving,
  abandoning, superseding, parking, splitting, invalidating, or closing LEAF
  work, `leaf fall`, old folder names such as `seeds` / `01-seeds` / top-level
  `pressed`, "구조 마이그레이션", or `leaf doctor` warnings about old layout or
  legacy status fields. Do not use for capturing ideas, producing leaf content,
  execution, or wt artifacts.
---

# LEAF Clean

Tend the `.leaf/` workspace itself, not the content of any leaf. Three
operations share this skill:

- **Press** — compress a completed, reference-worthy leaf into a citable
  `pressed.md` digest plus a `## Press Abstract` in its `00-status.md`, and
  record cross-leaf citations in `linked.md` when the leaf cites or is cited by
  other leaves.
- **Fall** — move work that should stop being carried, and is not worth keeping
  as a reference leaf, into `fallen` with an explicit fallen reason.
- **Migrate** — repair an old workspace layout. `leaf doctor` detects legacy
  leftovers and delegates the fix to "the migration operator"; this skill is
  that operator.

## Boundary

- Work only inside `.leaf/`.
- Move and rewrite structure; never delete content. Keep raw wording, unknowns,
  criteria, wireframe, design, task graph, review notes, references, and
  retrospective artifacts intact.
- Do not read, write, infer, or validate `.wt/` files.
- Do not create wt TaskDocuments, workflows, branches, PRs, commits, tickets,
  or execution artifacts.
- Do not invent certainty. Mark missing, inferred, or unresolved points plainly.

## Reference Map

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: soul, reporting, fact-vs-assumption separation, user-language prose, review handoff |

## First Read

Inspect local truth before changing anything:

```bash
git status --short --branch
leaf doctor
find .leaf/01-sprouts .leaf/02-leaves .leaf/03-fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

Route from what you find:

- `leaf doctor` reports layout findings (`old_stage_dir_present`,
  `pressed_stage_dir_present`, `legacy_state_field`,
  `legacy_fall_reason_field`) → run **Migrate** first, even when the user asked
  for press or fall; the other operations assume the canonical layout.
- The user wants a citable digest → **Press**.
- The user wants work out of the carried set → **Fall**.

## Migrate

`leaf doctor` is the diagnosis; this operation is the repair. Map each finding
to its fix:

| Doctor finding | Repair |
|---|---|
| `old_stage_dir_present` (e.g. `.leaf/01-seeds`) | If the canonical dir (`.leaf/01-sprouts`) is missing or empty, rename the old dir to the canonical name. If both hold items, move item folders one by one into the canonical dir; on a slug collision, stop and ask. |
| `pressed_stage_dir_present` (top-level `.leaf/04-pressed/`) | Move each `{slug}.md` digest into the matching item folder as `pressed.md`. If no matching folder exists, report it and leave the digest in place. |
| `legacy_state_field` | Rewrite the status `state` field as the canonical `stage` field. |
| `legacy_fall_reason_field` | Rewrite `fall reason` as `fallen reason`. |
| `stage_dir_mismatch`, `duplicate_slug` | Do not pick a side silently; show both truths and ask which is canonical. |

Rules:

- Prefer `git mv` for tracked paths so history follows the move.
- Never merge folders by overwriting; a collision means stop.
- Migration changes locations and field names, never meaning: do not rewrite
  prose while migrating.
- Re-run `leaf doctor` after the repair. Report remaining findings by name
  instead of declaring the workspace clean.

## Press

Create a citable `pressed.md` digest inside the source leaf folder. Pressing
does not move, complete, fall, or execute the source; it writes a paper-style
abstract into the source `00-status.md` and compresses the reusable context
into `pressed.md` so later work can quote or cite it.

Press-specific boundary:

- Write the digest to `<source leaf>/pressed.md` and links to
  `<source leaf>/linked.md`, not a shared directory. Press never writes into
  other leaf folders; incoming citations are discovered by reading them.
- On every press, update only the source `00-status.md` with
  `## Press Abstract`; do not change source gate files unless the user
  separately asks for source edits.
- Do not present `pressed.md` as source truth. It is a digest with source
  links; the original LEAF files remain authoritative.

### Source Resolution

Resolve the source for `{slug}`:

- Prefer `.leaf/02-leaves/{slug}/` when a completed reference-worthy leaf
  exists.
- Use a sprout only when the user explicitly asks for a provisional digest;
  name that it is not yet a completed leaf.
- Use a fallen source only when the user explicitly asks to summarize archived
  material; name that it is not a reference-worthy leaf.
- If several sources exist and the user did not specify one, state which source
  you will use and why.
- If no source exists, stop and list likely available slugs.

If `pressed.md` already exists, read it before replacing or refreshing it.

### Read Order

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

Missing files are acceptable. Note them in `pressed.md` only when their absence
affects interpretation.

### Status Abstract

Before writing `pressed.md`, write or refresh `## Press Abstract` in the source
`00-status.md`. This is a paper-style abstract for humans scanning the leaf,
not a stage change.

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
abstract that says the source is insufficient and names the missing context.
Use the same core paragraph as `pressed.md`'s `Citation Summary`.

### Pressed Shape

Use this structure:

```markdown
# <human-readable title>

- source: <source path>
- pressed at: <local timestamp>
- citation handle: leaf:{slug}
- stage: <leaf / sprout / fallen if known>

## Citation Summary

<One compact paragraph that can be quoted elsewhere. Say what this LEAF item
was trying to clarify, what approach it took, what it produced, and what caveat
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
coverage. Include direct quotations only when the original wording matters;
keep them short and source them in `Source Map`.

### Linked Leaves

Leaves cite each other like papers. When the source files evidence a relation
to another leaf — it built on another leaf's result, reused its criteria or
design, superseded part of it, or the user names the relation — record it in
`<source leaf>/linked.md` alongside `pressed.md`.

```markdown
# Links: {slug}

- source: <source path>
- linked at: <local timestamp>
- citation handle: leaf:{slug}

## Cites

- `leaf:{other-slug}` — <what this leaf took from it and why>
  - evidence: <source file in this leaf that shows the relation>

## Cited By

<Snapshot computed at press time by scanning other leaves' linked.md; may be
stale until this leaf is pressed again.>

- `leaf:{other-slug}` — <what it took from this leaf>
```

Rules:

- Record only links the source files evidence or the user names. Never invent
  a relation to make the leaf look connected.
- A cited handle may point to a leaf that is not yet pressed, or to a sprout or
  fallen item; keep the link and name its stage.
- Fill `Cited By` by scanning, read-only:

  ```bash
  grep -rln "leaf:{slug}" .leaf/*/*/linked.md
  ```

- If no links exist in either direction, skip `linked.md` and say so in the
  report instead of writing an empty file.

### Quality Bar

Before writing `pressed.md`, separate:

- confirmed source facts
- agent inference from source facts
- unresolved or missing context

The digest must make those boundaries visible. If the source is too thin to
support a useful citation, create a short `pressed.md` that says so instead of
filling the gaps.

## Fall

Move LEAF work into `fallen` when it should stop being carried and should not
become a reference-worthy leaf. Fallen is for discarded, archived, invalidated,
superseded, or completed-but-not-worth-reusing work.

Fall-specific boundary:

- Do not use fall as a generic "done" button. If a completed item is useful
  future reference, it belongs in `leaves`, optionally pressed via the Press
  operation above.
- Do not auto-create successor work. Link an existing or proposed successor
  path only when needed.

If the source is missing, stop. If a fallen folder for the slug already exists,
stop and do not merge or overwrite.

### Closure Check

Before falling, verify:

- the user explicitly wants this work removed from the carried set
- the work is not being kept as a reference-worthy leaf
- the fallen reason can be named, such as `abandoned`, `superseded`, `parked`,
  `split`, `invalidated`, `archived`, or `completed-not-reference-worthy`
- review/sync notes exist when the work reached execution or external review,
  or the reason they are unnecessary is stated
- `04-Feedback/10-retrospect.md` exists, or you can write a minimal closure
  note before or after the CLI move

The AI should fill useful closure content when context is available; do not
make the user reconstruct why the work fell.

### Fall Move

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

If the source lacks `04-Feedback/10-retrospect.md`, create a concise one with
the same fields plus any evidence that caused closure.

## Report

Report per `../leaf-soul/SKILL.md`. Always include which operations ran
(migrate / press / fall) and confirmation that no `.wt/` or execution artifacts
were created. Per operation, include:

- **Migrate**: each doctor finding repaired, each path moved or field
  rewritten, findings intentionally left for the user, and the post-repair
  `leaf doctor` result
- **Press**: output path `<source>/pressed.md`, source path used,
  `00-status.md` `## Press Abstract` written or refreshed, whether this was a
  new digest or a refresh, `linked.md` written / refreshed / skipped with the
  cites and cited-by counts, and important missing source files or unresolved
  questions
- **Fall**: fallen path, source path that was moved, fallen reason, whether the
  work was incomplete, discarded, or completed-not-reference-worthy, and
  closure fields filled or intentionally left blank
