---
name: learn
description: Use when capturing, triaging, or deeply learning a sprout through LEAF Learn (① Intent and ② Unknowns & Context); trigger on idea backlog, "save this idea", "maybe later", brainstorm fragments, early document/product/research topics, finding the real question behind a curiosity, surfacing what is worth learning, reference/benchmark exploration, big-picture mapping, substantial writing or knowledge tasks, wanting to understand a topic for its own sake, bundled ideas that may need splitting, or deciding whether an idea should die, defer, enrich, split, or learn.
---

# LEAF Learn

`learn` owns **Learn**: ① Intent and ② Unknowns & Context. Learn is where an
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

## Research Tool Check

Before triage or local truth, check whether the current runtime exposes
`insane-search`/`insane-search-codex` and
`insane-research`/`insane-research-codex` on two separate surfaces:

1. **Skills/plugins.** Inspect the available skills list and, if needed, the
   plugin cache. If `insane-search-codex:insane-search` or an
   `insane-research-codex:*` skill is listed, it is available as a skill; read
   that skill's `SKILL.md` before using it.
2. **MCP/tools/apps.** Use `tool_search` only to discover deferred MCP tools or
   app tools. A `tool_search` miss means only that no matching MCP tool was
   exposed; it does not mean the skill/plugin is unavailable.

Record the result as checked context with separate entries for skills and MCP
tools, then continue normally. These capabilities are accelerators, not
blockers: if neither surface exposes them, use built-in search, fetch, browser,
and explicit limitation notes.

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
| `../soul/SKILL.md` | always: conduct, language, report shape, fact/guess separation, review handoff |
| `references/gate-01-intent.md` | ① Intent pass/fail contract before locking intent |
| `references/gate-02-unknowns-context.md` | ② Unknowns & Context contract before the Learn-rest check |
| `references/research-quality.md` | deep external research, source-quality grading, citation-backed synthesis, or blocked public-source access |
| `../work/references/gate-authoring.md` | drafting, grilling, or revising durable Learn artifacts |
| `../work/references/clarity-ledger.md` | choosing the weakest row to aim the next Learn question |
| `../work/references/experiment-log.md` | a ② unknown needs an independent probe |
| `../work/references/layout.md` | file layout, naming, file-vs-folder-by-count |

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
5. **Dispatch the four scout subagents in parallel.** Be the explorer-companion,
   not a clerk who only files the unknowns the user names. Fan out Terrain,
   Method, Judgment, Context as parallel subagents (see `## Parallel Scouts`)
   instead of one linear sweep; each writes to `01-Learn/02-references/`.
6. **Run ② Unknowns & Context as the leader.** Use
   `references/gate-02-unknowns-context.md` as the contract. Check the user's
   prior knowledge early (the **known knowns**) so teaching targets the gap, and
   surface every cell of the knowledge map — known/unknown × known/unknown — not
   only the named unknowns. Synthesize the scouts' findings into `02-unknowns.md`
   and a reading map (see Parallel Scouts → Reading map), not an answer. When ②
   changes what the user now understands or the scope of the curiosity, update the
   status overview in the same turn.
7. **Quiz, then offer Learn-rest options.** Before resting, invoke `polish`
   on the Learn/status surface as one report — this is the Learn phase boundary —
   open the gathered references for the user (live page, per `soul`), then quiz
   the user on the core (see Parallel Scouts → Quiz). Then show concrete
   curiosity threads the user could keep pulling, not only a generic "anything
   else?" question. Learn need not "finish": if a thread still pulls, keep
   exploring. This is a resting point, not a gate to clear.
8. **Lock the why / what / wireframe before resting.** Mandatory at every Learn
   close: record an explicit, user-approved triple in `00-status.md`'s preamble.
   Run it as a per-item **ask → approve → write** loop — for each of why, what,
   and wireframe, ask the open questions first (the `why` must be a sharp problem
   definition), get the user's approval of the wording, then write it. Never
   auto-fill the triple; a drafted proposal is only a starting point to confirm.
   `none — <reason>` is a valid approved answer (understanding-only, killed, or
   deferred sprouts). The lock is a **return-condition lock**: work consumes
   it, and ④/⑧/⑨ falsifying it reopens it via a recorded return. Contract:
   `references/gate-02-unknowns-context.md`.

Show the gathered references as a file tree first:

```bash
find .leaf/01-sprouts/<slug>/01-Learn/02-references -type f | sort
```

Render the result as a tree with a one-line note per file saying what it
covers. An empty or thin tree is evidence too — name it plainly instead of
hiding it; "no references were needed because <reason>" must be said, not
implied.

Learn-rest options + question (ask after the quiz):

Before asking, synthesize 3-5 concrete threads from the gathered references,
scout findings, unchecked assumptions, and quiz gaps. Each option must name a
specific concept, case, debate, or assumption; do not offer generic categories.
Phrase each option so the user can feel what judging it would unlock. If no
meaningful thread remains, say that plainly and offer resting here.

Use this shape:

> 더 탐색해볼 만한 결은 이렇게 보입니다:
> - 개념: <specific concept> — <why it is worth pulling>
> - 사례: <specific case> — <what it would clarify>
> - 논쟁: <specific disagreement> — <what fork it exposes>
> - 가정: <unchecked premise> — <what changes if it fails>

> 알고 싶던 걸 충분히 알게 됐나요? 아직 당신을 끌어당기는 결, 즉 더 파고 싶은
> 개념, 보고 싶은 사례, 짚어보고 싶은 논쟁, 확인하지 않은 가정이 남아 있나요?

Then lock the triple (Workflow step 8): ask why / what / wireframe one at a time,
write each to `00-status.md` only after the user approves its wording.

## Parallel Scouts

② is run as a fan-out, not a single linear sweep. The leader dispatches up to
four scout subagents at once, each searching the topic from one angle —
including active web search for external references, prior art, and benchmarks —
and writing what it finds to `01-Learn/02-references/` (one file per topic, named
for what it covers). The families are fixed so coverage is legible; what each scout
hunts for is fitted to this sprout.

Because `$leaf:learn` semantically includes the scout fan-out, the command is an
explicit delegation request for those scouts.

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
- **The leader, not a scout, owns the learner's own state — check it at entry.**
  Ask what the user already knows about the topic before going deep: this prior
  knowledge (the **known knowns**) comes from dialogue, not search — do not spawn
  a fifth scout for it. It targets teaching at the gap and is the baseline the
  closing knowledge quiz measures against; note too where the user is likely to be
  misled.
- **Scouts search actively, via the insane skills.** Terrain and Method scouts
  search the web for conventions, prior art, comparable cases, and recent domain
  changes, saving each find under `01-Learn/02-references/`. Write each scout's
  prompt to use the insane capability the Research Tool Check confirmed (skill or
  MCP tool) — `insane-research` for deep or source-heavy research, `insane-search`
  for blocked sources — falling back to built-in search, fetch, and browser when
  neither is exposed. Full web-search contract:
  `references/gate-02-unknowns-context.md`.
- **Default to high-quality research when external facts can change the
  judgment.** For disputed claims, statistics, law/policy/medical/financial
  facts, or source-heavy synthesis, follow `references/research-quality.md` for
  its source-rating, verification, citation, and access-path rules.
- **Scale honestly.** Skip a scout with nothing to find and name the skip and its
  reason; do not pad references for appearance.

### Reading map

After the scouts return, synthesize — do not dump. The leader's output is a
reading map: which threads to read first to find the 실마리, in what order, and
what each one lets the user judge for themselves. Summarize each reference back
into `02-unknowns.md` so later work does not have to re-read every file.

### Quiz

Handing over references is not the same as the user learning them. Before resting
Learn, pose a few short **multiple-choice** questions that check the user
understands the core knowledge — the key concepts, why it is the way it is, the
trade-offs and where the topic forks. Give each question 3–4 options whose wrong
answers are plausible-but-wrong (common misconceptions or near-misses), so
picking the right one takes understanding the concept, not recognizing a
keyword — this keeps it a check of understanding, not trivia recall, and the
point stays surfacing real understanding, not proving the files were read. After
the user answers, briefly confirm why the right option is right and the others
wrong, so the understanding is generated, not just selected. **Evaluate
knowledge, not judgment:** once the knowledge a decision rests on has been
investigated and understood, the ability to judge is assumed to follow, so do not
quiz the user on what to choose. Treat gaps the quiz reveals as fresh threads:
send the relevant scout back or point to the reading, then re-check. Keep it
light and curious, never an exam.

## Status Overview

`00-status.md` is the reader's table of contents for the LEAF, and its top is
what the `leaf` TUI preview shows. The preview renders only the **first 8
non-empty lines** of the file (`src/preview.rs` `STATUS_PREVIEW_LINES`), so the
three load-bearing items live in the **preamble, right under the title**, above
the operational fields:

- `why`: the problem definition locked at ① — keep it sharp;
- `what`: the locked deliverable form `work` will produce, or `none —
  <reason>`;
- `wireframe`: the cheap-preview form of that deliverable, or `none — <reason>`
  (understanding-only outputs may use a one-paragraph explanation, worked
  example, or quiz instead of a built wireframe).

These three are written only through the Learn-close lock — an ask → approve →
write loop, per item, never agent-authored (see Workflow step 8 and
`references/gate-02-unknowns-context.md`). The operational status parser
(`stage` / `current phase` / `current gate`) ignores these keys, so the triple
sits safely in the preamble above the operational fields. `leaf doctor` does
read the triple — it warns (`status_triple_missing` / `status_triple_unfilled`)
when a sprout or leaf lacks the why/what/wireframe lines or still carries the
scaffold `TODO` placeholder, so the summary the preview and detail header
surface is guaranteed present (a `none — …` value is a valid answer and is not
flagged; fallen and pressed are exempt).

The `## Overview` section below the preamble keeps the rest:

- `request`: the user's request in the user's words;
- `current scope`: what is included, excluded, split, or still undecided;
- `consistency rule`: why / what / wireframe and this overview change when the
  sprout changes.

Do not let `00-status.md` become stale. Whenever `01-intent.md`,
`02-unknowns.md`, split decisions, the locked triple, Learn-rest status, or a
later return changes what the sprout is exploring, revise it before reporting
back.

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

This table is the *signal* for whether parts diverge. When the work is to
**decide how to split** — which single grain to cut along, how the pieces order
and link — use the `split` skill, which reuses this Split Check for its
"whether/when" layer.

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
- Do not *build* ④ Wireframe, ⑤ Design, the task graph, or the artifact here —
  that is `work`. You DO decide and lock the *form* of the what and the
  wireframe at Learn close (the why / what / wireframe triple); you do not
  construct the wireframe instance, and ③ Criteria's detailed acceptance checks
  still belong to `work`.
- Reference and benchmark exploration is learning and belongs in ②; building
  from those references is not Learn's job.

## Response Shape

Report per `soul`: overview first, decision points up top, facts separate
from assumptions, and user-facing prose in the user's language.

Include briefly:

- sprout path and status label
- evidence checked
- what was captured or changed
- split/group/ask-first reasoning when relevant
- recommendation: `kill`, `defer`, `enrich`, `split`, or `keep exploring`
- next thread the user might pull if they resume later
