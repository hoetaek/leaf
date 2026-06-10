---
name: leaf-idea
description: Use when capturing, triaging, or running the LEAF Learn phase (① Intent and ② Unknowns & Context) on a seed — the entry point for a new piece of work before it is promoted to leaf-work; trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, locking intent, surfacing unknowns, reference/benchmark exploration (자료 조사, 레퍼런스 벤치마킹, 아이디어 발산), big-picture or architecture mapping for understanding, starting a substantial writing or knowledge task (글쓰기 프로세스, 문서 작성 순서, 논문 작성, 발표 자료, 초안 구조, "어떤 순서"), bundled ideas that may need separate seeds, or requests to decide whether an idea should die, defer, enrich, split, learn, or become structured LEAF work.
---

# LEAF Idea

This skill owns the **Learn phase** of LEAF — gates ① Intent and ② Unknowns &
Context — on seeds in `.leaf/01-seeds/<slug>/`. It carries a rough idea to the
point where you can judge what the work needs: capture it, lock the intent, learn
the unknowns, reach readiness, then hand off to `leaf-work` at ③.

The entry is triage, and an idea is allowed to die there. Capture is cheap; before
spending Learn on an idea, decide whether it is worth it — kill, defer, enrich,
split, or run Learn. Triage is the gate; Learn is the body once the idea earns it.

`leaf-work` takes over at ③ Example, after `leaf promote <slug>`. Do not pretend a
seed is ready for criteria, wireframe, design, tasks, or execution — that is the
other skill's job, and it begins only once Learn passes here.

A seed is one possible future leaf, not an inbox. Before capturing or promoting,
decide whether the rough input is one work item, several independent future
leaves, or one unstable frame that needs a question before it can be split.

## Boundary

- Use `.leaf/01-seeds/<slug>/`; do not write loose planning files elsewhere.
- One seed should represent one possible `leaf-work` thread. Do not make one
  seed carry multiple independent outcomes that could become sibling leaves.
- Create or resume a seed with the `leaf` CLI. Run `leaf init` first when
  `.leaf/` is absent, then `leaf new <slug>` unless the seed already exists.
- Work in `00-status.md` and the `01-Learn/` phase — `01-Learn/01-intent.md` (①),
  `01-Learn/02-unknowns.md` (②), and its sidecars `01-Learn/02-references/` and
  `01-Learn/02-experiments/`. Do not write into `02-Example/` or later phases.
- Do not fill ③ Criteria, ④ Wireframe, ⑤ Design, or tasks from this skill.
  Mention the next gate only as a recommendation; `leaf promote <slug>` hands the
  seed to `leaf-work` for ③ onward.
- If the user wants a real artifact, plan, draft, task graph, or execution path
  now, switch to `leaf-work`. (Reference and benchmark *exploration* to learn the
  unknowns is ②'s job here; building from them is `leaf-work`.)
- `leaf promote <slug>` is the boundary from seed to active leaf. Do not run it
  merely because an idea was captured; run it only when the user explicitly
  commits the work and the next LEAF move is after Learn.

## Reference map

Conduct is shared across the LEAF family, while the Learn gate contract lives
with `leaf-idea`. **Invoke the `leaf-soul` skill with the Skill tool** at the
start and follow it — it is shared conduct to run, not just a file to skim. When
the work needs another LEAF skill — `leaf promote` into `leaf-work` at ③,
`leaf-press`, or `leaf-fall` — invoke that skill rather than only reading its
file. Read the gate-contract references below:

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | the shared conduct/voice: how to report, show rendered work, hand off reviewables, separate fact from guess, and which language to write — follow it |
| `references/learn-gates.md` | the authoritative pass/fail test for ① Intent and ② Unknowns & Context — act by it |
| `../leaf-work/references/gate-authoring.md` | drafting, grilling, or revising the intent or unknowns artifact |
| `../leaf-work/references/clarity-ledger.md` | as a lens in ① / ② — glance at its five rows to aim the next question or learning gap |
| `../leaf-work/references/experiment-log.md` | a ② unknown needs an independent, cheap probe to settle ("is this true?") |
| `../leaf-work/references/layout.md` | naming, file-vs-folder-by-count, and the `00-status.md` template |

## First Read

Inspect local truth before asking:

```bash
git status --short --branch
find .leaf/01-seeds .leaf/02-leaves -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort
```

If a likely matching seed already exists, resume it instead of creating a
duplicate. Use lowercase ASCII kebab-case slugs.

## Split Check

Run this check before creating a seed, during enrichment when the idea starts to
branch, whenever the user adds a new direction, and before recommending
promotion.

Use `split now` when bundled parts have independent core nouns, deliverables,
success checks, reviewers/arbiters, lifecycles, likely-change axes, or
review/promote paths. Treat a newly introduced CLI command, config surface,
viewer/editor surface, policy/doc change, data/source pipeline, or separately
testable user workflow as a split signal unless it clearly serves the same
outcome.

Use `keep grouped` when parts are sequential concerns inside one outcome: one
stable core noun, one deliverable, one acceptance check, one lifecycle, and one
part naturally feeds the next. If a part could become its own PR, review, or
resume path, do not call it grouped just because it is related.

Use `ask first` when splitting would decide the user's intent for them: the core
noun is drifting, the output form is still exploratory, or the quieter sibling
is not concrete enough to name. Ask one focused question instead of creating
several speculative seeds.

If the user asked to capture the work and the split is clear, create or resume
sibling seeds. Otherwise recommend `split` and name the seed candidates. Do not
promote a known mixed seed as one active leaf; split it first, or record the
explicit reason it is one grouped outcome.

## Capture

Record a compact idea snapshot first — enough to triage, not yet the full Learn
pass:

- raw user wording, preserving phrasing that may matter later
- current hunch: what this might become, stated as tentative
- why it surfaced: problem, obligation, curiosity, discomfort, or unknown
- possible output forms, if visible
- related seeds, leaves, docs, files, or prior decisions checked

Write raw wording and current hunch in `01-Learn/01-intent.md`; write context
checked and open questions in `01-Learn/02-unknowns.md`. If triage says `kill` or
`defer`, stop here. If the idea earns Learn, deepen it through ① and ② below.

## Gate ① Intent

Lock why the work is needed and, derived from that, what is actually wanted. Read
`references/learn-gates.md` ① for the full contract; the moves below are what
this gate adds in practice.

**Surface your guessed facts and ask before locking.** Stating an intent forces
assumptions — about the purpose, the audience, the output form, the deadline,
what the user already has. Do not bury them in a confident sentence. List the
facts you are *guessing* rather than know, mark each `ASSUMPTION:`, and ask the
user to confirm or correct them before locking the one-sentence intent. A wrong
guess locked here unravels every later gate; a guess the user confirms becomes a
fact the work can stand on.

Then record: raw wording preserved; the why followed to where it lands (a problem
definition, an external obligation with the deeper why asked, curiosity locked as
exploratory, or a felt sense deferred to ②); the current one-sentence intent
stated separately from the raw wording, with any gap from the surface request
noted; a stable core noun. The work is still allowed to die or change shape.

### Why / What Separation

Why/what separation is mandatory in ①. Use `references/learn-gates.md` as the
single pass/fail contract for the exact fields, questions, and close check.

## Gate ② Unknowns & Context

Learn what the work needs until you can say what to choose between and on what
basis — that point is what ③ Criteria consumes. Read
`references/learn-gates.md` ② for the full contract, and use
`../leaf-work/references/clarity-ledger.md` as a lens to aim learning at the
weakest dimension.

Categorize unknowns by kind (domain concepts / standards & conventions /
selection criteria) and by source — **external** (comparable cases, prior art,
benchmarks, authoritative sources, recent domain changes) and **internal** (what
the user or team already holds).

**Sketch the big picture as study material.** When the work touches an existing
system, product architecture, workflow, domain model, multiple actors/surfaces,
or enough references that the shape is easy to lose, create a compact ASCII map
under `01-Learn/02-references/` (for example `architecture-map.md`). Map the
current known topology: actors, surfaces, components, data/content flow,
ownership boundaries, external dependencies, preserved constraints, and the
places marked `UNKNOWN` or `ASSUMPTION`. This is a learning artifact, not ⑤
Design: do not invent the after architecture here. Use the map to find missing
facts, split signals, and candidate choices; summarize only those distilled
facts and gaps back into `01-Learn/02-unknowns.md`.

**Drive the external facts: find what to search, then ask the user.** Standards,
conventions, and comparables live outside and shift over time — do not guess
them. Before searching, name the specific external facts this work depends on and
turn them into concrete search targets (e.g. "the accepted structure of a
공적서", "how comparable reports open", "the current rubric wording"). Show the
user that list and ask: which should I look up, which do you already know the
answer to, and what would you search that I have not named? Let the user confirm
or reprioritize the targets and supply the internal facts only they hold, before
you spend effort searching.

**Build your own context files — this is not lazy gathering.**
`01-Learn/02-references/` is not an on-demand scratchpad you touch only when
stuck; in Learn you always populate it. Pull both the **external** references
(comparable cases, prior art, benchmarks, authoritative sources) and the
**internal** ones (the user's or team's own documents, data, prior decisions)
into your own context files there — one folder or file per source — and keep each
in a form you can see and judge by eye, not reduced to a one-line note. For UI or
web work, capture the rendered reference page as a self-contained `.html` file
(and a screenshot); for documents, keep the source excerpt or PDF; for data or
code, the snippet. Open them *for* the user and judge together what to copy,
adapt, avoid, or reject — show, do not just link (see `../leaf-soul/SKILL.md`:
Show the work). Deliberately gather both poles: **models** worth emulating or benchmarking against
(what "great" looks like for this work) and **anti-models** — the cautionary cases
that failed or took the shortcut that sinks this kind of work — and label each, so
judgment has something to copy *and* a concrete example of what to steer clear of.
A failure mode you can point at is often sharper than a success you admire.
References are study material for judgment, not a design to copy wholesale; the
locked instance comes later at ④, not from a pasted reference.

**Then extract the essentials out.** `02-references/` holds the raw gathered
context — the inside; `02-unknowns.md` holds the distilled facts the later gates
read — the outside. From the context files, summarize only what the work truly
needs (the established fact, the verdict, the convention to follow) back out into
`02-unknowns.md`, with its source. When an unknown needs an independent probe to
settle, use the experiment machine (`../leaf-work/references/experiment-log.md`)
and keep the process in `01-Learn/02-experiments/{name}.md`.

### Premise Inventory

The Premise Inventory is mandatory in ②. Use `references/learn-gates.md` as the
single pass/fail contract for the exact tables, labels, assumption rule, and
close check.

Before closing Learn, always ask the user this explicitly:

> 정말로 이 일을 판단하고 기준화하기 위해 배우고 알아야 하는 사실을 다
> 알았나요? 아직 확인해야 할 사실, 참고해야 할 사례, 내부 맥락, 또는
> 검증하지 않은 가정이 남아 있나요?

Treat this as a hard close check, not a nicety. If the user names any remaining
fact, reference, internal context, or unverified assumption, keep the seed at
`enriched`, record the gap in `01-Learn/02-unknowns.md`, and continue Learn
instead of recommending promotion. Close the gate only when blocking unknowns
have sourced answers or flagged assumptions, the fact/assumption boundary is
visible, the user has answered the Learn-close question, and the user can state
what to choose between and on what basis.

## Review Handoff

Hand off reviewables per `../leaf-soul/SKILL.md` (mark `USER REVIEW NEEDED:` /
`ASSUMPTION:`, open in the user's editor, show HTML in a browser).

Use these status labels in `00-status.md`:

- `captured`: raw idea saved with minimal context
- `enriched`: meaningful context, references, or alternatives were added
- `ready-for-leaf-work`: ① Intent and ② Unknowns & Context have passed; ③
  Criteria can start in `leaf-work` after `leaf promote <slug>`
- `deferred`: intentionally parked with a resume condition
- `killed`: intentionally not worth pursuing now

Do not mark seeds as `fallen`. `fallen` is only for committed
`.leaf/02-leaves/<slug>/` work that is closed later.

## Triage

End every pass with one recommendation:

| Recommendation | Use when |
|---|---|
| `kill` | no problem, obligation, curiosity, or discomfort survives inspection |
| `defer` | the idea is real but not worth attention until a named condition changes |
| `enrich` | one or two cheap facts/examples would decide whether it has weight |
| `split` | several independent future leaves are bundled together and need separate seeds |
| `promote to leaf-work` | ① and ② have passed, the Learn-close question was asked and cleared, and the user commits to Example onward |

Promotion is the Learn→Example boundary. Recommend it only when ① Intent and ②
Unknowns & Context have passed here and the user has explicitly confirmed that
there are no remaining facts, references, internal context, or assumptions Learn
must settle first. Name what ③ Criteria should consume — what the user can now
choose between and on what basis. Run `leaf promote <slug>` after explicit user
approval and continue from `.leaf/02-leaves/<slug>/` in `leaf-work`.

## Response Shape

Report per `../leaf-soul/SKILL.md` — overview-first, a **Verify / Decide** list up
top, plain words, gathered material organized and shown rather than dumped.

Report briefly:

- a one-line overview and the **Verify / Decide** points up top
- seed path
- status label
- evidence checked
- what was captured or changed
- file or sections opened for review, if any (HTML shown, not just linked)
- recommendation and why, including split/group/ask-first reasoning when relevant
- next action, if the user resumes later
