---
name: leaf-idea
description: Use when capturing, triaging, or running LEAF Learn (① Intent and ② Unknowns & Context) on a sprout; trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, locking intent, surfacing unknowns, reference/benchmark exploration, big-picture mapping, substantial writing or knowledge tasks, bundled ideas that may need splitting, or deciding whether an idea should die, defer, enrich, split, learn, or continue.
---

# LEAF Idea

`leaf-idea` owns **Learn**: ① Intent and ② Unknowns & Context. It turns a rough
idea into a sprout whose why, provisional what, unknowns, facts, assumptions,
and split decision are clear enough for ③ Criteria.

## Core Model

- A sprout is one possible future leaf, not an inbox item.
- Learn runs in the same sprout that later continues to ③; do not require a
  stage move.
- `leaf-work` takes over at ③ Example after Learn passes, still in the same
  sprout.
- Concepts, taxonomies, models, policies, decision records, plans, documents,
  UI, and code changes can all be valid artifacts. If implementation becomes a
  separate lifecycle, reviewer, success check, or code surface, split it into a
  separate sprout.
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
| `references/gate-02-unknowns-context.md` | ② Unknowns & Context pass/fail contract before closing Learn |
| `../leaf-work/references/gate-authoring.md` | drafting, grilling, or revising durable Learn artifacts |
| `../leaf-work/references/clarity-ledger.md` | choosing the weakest row to aim the next Learn question |
| `../leaf-work/references/experiment-log.md` | a ② unknown needs an independent probe |
| `../leaf-work/references/layout.md` | file layout, naming, file-vs-folder-by-count |

## Workflow

1. **Triage before Learn.** Decide whether the idea should `kill`, `defer`,
   `enrich`, `split`, or run Learn. Capture is cheap; Learn starts only if the
   idea earns it.
2. **Create or resume the sprout.** Use `leaf init` if needed, then
   `leaf new <slug>` unless a matching sprout already exists.
3. **Capture the snapshot.** In `01-Learn/01-intent.md`, preserve raw wording and
   current hunch. In `01-Learn/02-unknowns.md`, record checked context and open
   questions.
4. **Run ① Intent.** Use `references/gate-01-intent.md` as the contract. Separate
   raw wording, sharp why, provisional what, and locked intent. Surface guessed
   facts before locking the intent.
5. **Run ② Unknowns & Context.** Use `references/gate-02-unknowns-context.md` as the
   contract. Group unknowns, gather references or internal facts, expose
   premises, and keep fact/assumption/user-review boundaries visible.
6. **Ask the Learn-close question with the evidence in view.** Before asking,
   show what was actually gathered so the user judges sufficiency from the
   files, not from memory. If anything remains, keep the sprout enriched and
   continue Learn. If the user confirms no Learn-blocking unknowns remain,
   recommend `continue to Example`.

Show the gathered references as a file tree first:

```bash
find .leaf/01-sprouts/<slug>/01-Learn/02-references -type f | sort
```

Render the result as a tree with a one-line note per file saying what it
covers. An empty or thin tree is evidence too — name it plainly instead of
hiding it; "no references were needed because <reason>" must be said, not
implied.

Learn-close question:

> 정말로 이 일을 판단하고 기준화하기 위해 배우고 알아야 하는 사실을 다
> 알았나요? 아직 확인해야 할 사실, 참고해야 할 사례, 내부 맥락, 또는
> 검증하지 않은 가정이 남아 있나요?

## Learn Handoff

① produces the why, provisional what, and first questions ② must answer before
the right what can be chosen.

② produces the facts, assumptions, candidate judgments, premises, and explicit
review items that ③ Criteria consumes.

Do not continue to ③ merely because files exist. Continue only when ① and ② pass
their gate contracts and the user has answered the Learn-close question.

## Split Check

Run this before creating a sprout, when the idea branches, when the user adds a
new direction, and before recommending ③.

| Verdict | Use when |
|---|---|
| `split now` | bundled parts have independent core nouns, artifacts, success checks, reviewers, lifecycles, likely-change axes, or review/continuation paths |
| `keep grouped` | parts are sequential concerns inside one outcome: one noun, one artifact, one acceptance check, one lifecycle |
| `ask first` | splitting would decide the user's intent: the noun drifts, output form is exploratory, or a quieter sibling is not concrete enough |

If split is clear and the user asked to capture the work, create or resume
sibling sprouts. Otherwise recommend the split and name candidates. Do not carry
a known mixed sprout into Example as one leaf unless the grouping reason is
explicit.

## Status Labels

Use these in `00-status.md`:

| Label | Meaning |
|---|---|
| `captured` | raw idea saved with minimal context |
| `enriched` | meaningful context, references, premises, or alternatives were added |
| `ready-for-example` | ① and ② passed; ③ Criteria can start in `leaf-work` in the same sprout |
| `deferred` | parked until a named condition changes |
| `killed` | not worth pursuing now |

Do not mark active sprouts as `fallen` by editing status alone. `fallen` is a
stage reached through an explicit leaf CLI action with a fallen reason.

## Boundaries

- Work only in `00-status.md` and `01-Learn/` from this skill.
- Do not fill ③ Criteria, ④ Wireframe, ⑤ Design, or tasks here.
- If the user wants to build the selected artifact, draft, task graph, or
  execution path now, switch to `leaf-work`.
- Reference and benchmark exploration belongs in ②; building from those
  references belongs in later gates.

## Response Shape

Report per `leaf-soul`: overview first, decision points up top, facts separate
from assumptions, and user-facing prose in the user's language.

Include briefly:

- sprout path and status label
- evidence checked
- what was captured or changed
- split/group/ask-first reasoning when relevant
- recommendation: `kill`, `defer`, `enrich`, `split`, or `continue to Example`
- next action if the user resumes later
