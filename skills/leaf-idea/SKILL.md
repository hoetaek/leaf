---
name: leaf-idea
description: Use when capturing, triaging, or deeply learning a sprout through LEAF Learn (① Intent and ② Unknowns & Context); trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, finding the real question behind a curiosity, surfacing what is worth learning, reference/benchmark exploration, big-picture mapping, substantial writing or knowledge tasks, wanting to understand a topic for its own sake, bundled ideas that may need splitting, or deciding whether an idea should die, defer, enrich, split, or learn.
---

# LEAF Idea

`leaf-idea` owns **Learn**: ① Intent and ② Unknowns & Context. Learn is where an
eager learner goes to *understand a topic for its own sake*. ① locks the why (the
problem definition) and the what (what the output is); ② explores the terrain —
facts, conventions, prior art, debates, hidden premises — until the user has
actually learned it and can judge it for themselves. Reaching a next phase is not
the point; the learning is.

## Core Model

- A sprout is one possible future leaf, not an inbox item.
- Learn is a place to dwell, not a stage to clear. The user may keep learning as
  long as the topic pulls them; depth is welcome, not waste.
- Concepts, taxonomies, models, policies, decision records, plans, documents,
  UI, and code changes can all be the thing a sprout is about. If one sprout
  bundles parts with independent cores, split it so each can be learned on its
  own.
- Older compatibility folders may exist as storage data only.

## First Read

Inspect local truth before asking:

```bash
git status --short --branch
find .leaf/01-sprouts .leaf/02-leaves .leaf/03-fallen -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

Resume a likely matching sprout instead of creating a duplicate. Use lowercase
ASCII kebab-case slugs.

## References

Read only what the current move needs:

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | always: conduct, language, report shape, fact/guess separation, review handoff |
| `references/gate-01-intent.md` | ① Intent pass/fail contract before locking intent |
| `references/gate-02-unknowns-context.md` | ② Unknowns & Context contract before the Learn-rest check |
| `../leaf-work/references/gate-authoring.md` | drafting, grilling, or revising durable Learn artifacts |
| `../leaf-work/references/clarity-ledger.md` | choosing the weakest row to aim the next Learn question |
| `../leaf-work/references/experiment-log.md` | a ② unknown needs an independent probe |
| `../leaf-work/references/layout.md` | file layout, naming, file-vs-folder-by-count |

## Workflow

1. **Triage before Learn.** Decide whether the idea should `kill`, `defer`,
   `enrich`, `split`, or run Learn. Capture is cheap; Learn starts only when the
   idea actually pulls at the user.
2. **Create or resume the sprout.** Use `leaf init` if needed, then
   `leaf new <slug>` unless a matching sprout already exists.
3. **Capture the snapshot.** In `01-Learn/01-intent.md`, preserve raw wording and
   the current hunch. In `01-Learn/02-unknowns.md`, record checked context and open
   questions. Also update `00-status.md` `## Overview` so a reader can see what
   this sprout is exploring without opening every gate file.
4. **Run ① Intent.** Use `references/gate-01-intent.md` as the contract. Separate
   raw wording, the sharp why (the problem definition), the provisional what (what
   the output is), and the locked intent. Surface guessed facts before locking it.
   When ① changes the why, the what, the core noun, or the split decision, update
   the status overview in the same turn.
5. **Open the learning terrain.** Be the explorer-companion, not a clerk who only
   files the unknowns the user happens to name. First map what is worth learning
   here — domain concepts, history, conventions, counter-examples, live debates,
   surprising connections to other fields — and offer it as a menu, surfacing
   threads the user did not know to ask about. Then follow the user's curiosity:
   go deep on the thread they pick, amplify it, and connect it to the next. The
   menu opens the space wider; it never narrows it.
6. **Run ② Unknowns & Context.** Use `references/gate-02-unknowns-context.md` as
   the contract. Group what is worth learning, gather references or internal
   facts, expose the premises the user is taking for granted, and keep
   fact/assumption boundaries visible. When ② changes what the user now
   understands or the scope of the curiosity, update the status overview in the
   same turn.
7. **Ask the Learn-rest question with the evidence in view.** Before asking the
   user to review Learn or answer the rest question, invoke `leaf-clean` for the
   touched Learn/status surface so the user reviews the current report, not draft
   notes. Then show what was actually gathered so the user judges from the files,
   not from memory. Learn does not need to "finish": if a thread still pulls,
   keep exploring. This is a resting point, not a gate to clear.

Show the gathered references as a file tree first:

```bash
find .leaf/01-sprouts/<slug>/01-Learn/02-references -type f | sort
```

Render the result as a tree with a one-line note per file saying what it
covers. An empty or thin tree is evidence too — name it plainly instead of
hiding it; "no references were needed because <reason>" must be said, not
implied.

Learn-rest question:

> 알고 싶던 걸 충분히 알게 됐나요? 아직 당신을 끌어당기는 결 — 더 파고 싶은
> 개념, 보고 싶은 사례, 짚어보고 싶은 논쟁, 확인하지 않은 가정 — 이 남아
> 있나요?

## Status Overview

`00-status.md` is the reader's table of contents for the LEAF. It is not the
source of truth for detailed reasoning, but it must summarize the current shape:

- `request`: the user's request in the user's words;
- `purpose`: why this sprout exists, after ① separates why from what;
- `expected output`: the artifact, decision, model, document, code change, or
  result this sprout is currently aiming at (understanding a concept can itself
  be the output);
- `current scope`: what is included, excluded, split, or still undecided;
- `consistency rule`: reminder that this overview changes when the sprout changes.

Do not let `00-status.md` become stale. Whenever `01-intent.md`,
`02-unknowns.md`, split decisions, Learn-rest status, or a later return changes
what the sprout is exploring, revise the overview before reporting back.

## Split Check

Run this before creating a sprout, when the idea branches, when the user adds a
new direction, and before going deep on a topic.

| Verdict | Use when |
|---|---|
| `split now` | bundled parts have independent core nouns, artifacts, success checks, reviewers, lifecycles, likely-change axes, or review/continuation paths |
| `keep grouped` | parts are sequential concerns inside one outcome: one noun, one artifact, one acceptance check, one lifecycle |
| `ask first` | splitting would decide the user's intent: the noun drifts, output form is exploratory, or a quieter sibling is not concrete enough |

If split is clear and the user asked to capture the work, create or resume
sibling sprouts. Otherwise recommend the split and name candidates. Do not learn
a known mixed sprout as one topic unless the grouping reason is explicit.

## Status Labels

Use these in `00-status.md`:

| Label | Meaning |
|---|---|
| `captured` | raw idea saved with minimal context |
| `enriched` | meaningful context, references, premises, or alternatives were added |
| `explored` | the terrain has been learned deeply enough that the user can judge it for themselves; the topic can rest here |
| `deferred` | parked until a named condition changes |
| `killed` | not worth pursuing now |

Do not mark active sprouts as `fallen` by editing status alone. `fallen` is a
stage reached through an explicit leaf CLI action with a fallen reason.

## Boundaries

- Work only in `00-status.md` and `01-Learn/` from this skill.
- Do not fill ③ Criteria, ④ Wireframe, ⑤ Design, or tasks here. Building the
  artifact, draft, task graph, or execution path is a different kind of work for
  a different skill; Learn stays learning.
- Reference and benchmark exploration is learning and belongs in ②; building
  from those references is not Learn's job.

## Response Shape

Report per `leaf-soul`: overview first, decision points up top, facts separate
from assumptions, and user-facing prose in the user's language.

Include briefly:

- sprout path and status label
- evidence checked
- what was captured or changed
- split/group/ask-first reasoning when relevant
- recommendation: `kill`, `defer`, `enrich`, `split`, or `keep exploring`
- next thread the user might pull if they resume later
