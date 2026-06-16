---
name: leaf-learn
description: Use when capturing, triaging, or deeply learning a sprout through LEAF Learn (① Intent and ② Unknowns & Context); trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, finding the real question behind a curiosity, surfacing what is worth learning, reference/benchmark exploration, big-picture mapping, substantial writing or knowledge tasks, wanting to understand a topic for its own sake, bundled ideas that may need splitting, or deciding whether an idea should die, defer, enrich, split, or learn.
---

# LEAF Learn

`leaf-learn` owns **Learn**: ① Intent and ② Unknowns & Context. Learn is where an
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
5. **Dispatch the four scouts in parallel.** Be the explorer-companion, not a
   clerk who only files the unknowns the user names. Fan out Terrain, Method,
   Judgment, Context (see `## Parallel Scouts`) instead of one linear sweep; each
   writes to `01-Learn/02-references/`.
6. **Run ② Unknowns & Context as the leader.** Use
   `references/gate-02-unknowns-context.md` as the contract. Synthesize the
   scouts' findings into `02-unknowns.md` and a reading map (see Parallel Scouts →
   Reading map), not an answer. When ② changes what the user now understands or
   the scope of the curiosity, update the status overview in the same turn.
7. **Quiz, then ask the Learn-rest question.** Before resting, invoke `leaf-clean`
   for the touched Learn/status surface, show the gathered references, then quiz
   the user on the core (see Parallel Scouts → Quiz). Learn need not "finish": if
   a thread still pulls, keep exploring. This is a resting point, not a gate to
   clear.

Show the gathered references as a file tree first:

```bash
find .leaf/01-sprouts/<slug>/01-Learn/02-references -type f | sort
```

Render the result as a tree with a one-line note per file saying what it
covers. An empty or thin tree is evidence too — name it plainly instead of
hiding it; "no references were needed because <reason>" must be said, not
implied.

Learn-rest question (ask after the quiz):

> 알고 싶던 걸 충분히 알게 됐나요? 아직 당신을 끌어당기는 결 — 더 파고 싶은
> 개념, 보고 싶은 사례, 짚어보고 싶은 논쟁, 확인하지 않은 가정 — 이 남아
> 있나요?

## Parallel Scouts

② is run as a fan-out, not a single linear sweep. The leader dispatches up to
four scout subagents at once, each searching the topic from one angle and
writing what it finds to `01-Learn/02-references/` (one file per topic, named for
what it covers). The families are fixed so coverage is legible; what each scout
hunts for is fitted to this sprout.

| Scout | Question it answers | What it digs for (fit to the sprout) |
|---|---|---|
| **A. Terrain** | What exists? | external references & authoritative prior art, domain concepts & terminology, internal assets (existing code, docs, prior decisions, data), available tools/ecosystem |
| **B. Method** | How is it done? | best practices & methodology, real-world cases & benchmarks, failure cases & anti-patterns |
| **C. Judgment** | Where does it fork? | trade-offs & selection criteria, live debates & expert disagreement, hidden premises & constraints |
| **D. Context** | Why is it this way? | history & evolution, recent domain changes, analogies from adjacent fields, stakeholders |

Rules for the fan-out:

- **C is never dropped.** A/B/D answer "what is true / how / why"; C is what turns
  collection into judgment. Skipping it leaves the user with a pile of material
  and no way to decide — the exact failure Learn exists to prevent.
- **The scouts return grounds, not verdicts.** Each writes "here is what I found
  and where" — threads the user can pull and verify — never "the answer is X." The
  conclusion is the user's to reach.
- **The leader, not a scout, owns the learner's own state.** What the user already
  knows and where they are likely to be misled comes from dialogue, not search.
  Do not spawn a fifth scout for it.
- **Scale honestly.** Skip a scout with nothing to find and name the skip and its
  reason; do not pad references for appearance.

### Reading map

After the scouts return, synthesize — do not dump. The leader's output is a
reading map: which threads to read first to find the 실마리, in what order, and
what each one lets the user judge for themselves. Summarize each reference back
into `02-unknowns.md` so later work does not have to re-read every file.

### Quiz

Handing over references is not the same as the user learning them. Before resting
Learn, pose a few short questions that make the user retrieve and apply the core —
the trade-off, the why-it-is-this-way, the what-would-break-it — not trivia
recall. The point is the user generating the understanding themselves, not
proving they read the files. Treat gaps the quiz reveals as fresh threads: send
the relevant scout back or point to the reading, then re-check. Keep it light and
curious, never an exam.

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
