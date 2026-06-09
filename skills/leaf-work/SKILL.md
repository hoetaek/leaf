---
name: leaf-work
description: |
  Use when turning a vague idea into structured knowledge work or writing —
  documents, essays, articles, memos, research papers, proposals, reports,
  specs, study notes, presentations, prototypes, or any non-code or mixed
  deliverable in a repo-local `.leaf/` workspace. Use when the request is vague,
  high-stakes, long-form, collaborative, research-heavy, or likely to sprawl,
  and when a rough intent must become an executable plan. Trigger on "어떤 순서",
  "글쓰기 프로세스",
  "문서 작성 순서", "논문 작성", "발표 자료", "초안 구조", "자료 조사",
  "레퍼런스 벤치마킹", "아이디어 발산", "작업을 어떻게 쪼갤지".
---

# LEAF Work

**Leaf before tree.** Validate one cheap, inspectable instance before growing
it into the whole artifact. The core move is never to **generate the whole
artifact upfront** — that is how you produce confident-looking slop and lose the
way before you can even tell the direction is wrong. It is to **learn first, make
one instance right, then expand** — closing the cheapest decisive uncertainty
before starting the next kind of work.

LEAF names the four kinds of uncertainty to close, in order:

| Phase | What it makes you able to do | Gates |
|---|---|---|
| **Learn** | Judge what the work needs — learned, not guessed *(run in `leaf-idea` on a seed; inherited here)* | ① Intent · ② Unknowns & Context |
| **Example** | Prove one cheap instance right before scaling | ③ Criteria · ④ Wireframe |
| **Architect** | Generalize that instance into a shippable generator | ⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact |
| **Feedback** | Confirm it still holds, then settle what was established and what was learned | ⑨ Review/sync · ⑩ Retrospect |

Learn runs in the `leaf-idea` skill on a seed; `leaf-work` begins at ③ Example
after `leaf promote <slug>`. The gate contracts in `references/gates.md` cover all
ten gates and are shared by both skills.

## Always-on rules

- **Use the leaf CLI as the persistent body of the work.** `leaf-work` assumes
  work lives in a repo-local `.leaf/` workspace and enters at ③ Example, on an
  active leaf under `.leaf/02-leaves/<slug>/`. Learn (① Intent, ② Unknowns &
  Context) runs first in the `leaf-idea` skill on a seed; `leaf promote <slug>`
  moves that seed to an active leaf and is the boundary into this skill. If the
  work has not been through Learn — no seed exists, or ① and ② have not passed —
  route to `leaf-idea` first rather than starting gate work here. If an active
  leaf already exists, resume it. Do not create loose `01-Learn/` …
  `04-Feedback/` folders at the repo root.
- **Inherit Learn; do not redo it.** A promoted leaf arrives with ① Intent locked
  and ② Unknowns & Context resolved in `01-Learn/`. Start by reading them: the
  one-sentence intent and its why, the sourced facts and flagged assumptions, the
  references gathered, and what the user can already choose between (the input ③
  Criteria consumes). Trust that work and reopen Learn only on a return.
- **Conduct and voice are shared — follow `leaf-soul`.** How you report
  (overview-first, a Verify / Decide list up top), show rendered HTML work (open it
  in Chrome DevTools and screenshot each state — never make the user hunt), hand
  off reviewables (`USER REVIEW NEEDED:` / `ASSUMPTION:`), separate fact from
  guess, and which language you write are defined once for the whole LEAF family in
  `../leaf-soul/SKILL.md`. Read it and follow it; the rules here are the gate
  *method* specific to this skill.
- **The scaffold is the foundation — so the work stands firm.** Learn, Example,
  and Architect lay that foundation; execution builds only once it is solid, not
  on guesses. A promoted leaf already carries the CLI scaffold — `00-status.md`
  and the four phase folders — created in `leaf-idea` via `leaf new`; you inherit
  it, you do not recreate it. The scaffold is the *body* of LEAF: it makes "which
  gate am I in / what is the first missing gate" a place you can point to, and
  each gate file fills in as context settles. If a task is too small to deserve
  that foundation, do not invoke leaf-work at all — there is no LEAF without a
  body.
- **Start from `00-status.md` when it exists.** Use it as the project dashboard:
  current phase/gate, first missing gate, next action, and gate progress. It is
  an index, not the source of truth; gate files remain authoritative.
- **Read `references/gates.md`, tell the user the current gate, and act by it.**
  With the scaffold and `00-status.md` in hand, open `gates.md`, identify which
  gate the work is at, name that gate to the user, and then proceed exactly as
  that gate's entry/exit/return conditions direct. SKILL.md gives the shape;
  gates.md gives the pass/fail test you act on.
- **Use the gate authoring loop for durable gate files.** When creating or
  revising a gate artifact, draft the smallest useful version, challenge it
  against the gate's foci, revise, and show only the review surface needed for
  the current approval policy. See `references/gate-authoring.md`.
- **Keep `00-status.md` current.** Update it when a gate starts, becomes ready
  for review, completes, needs explicit approval, is approved, returns to an
  earlier gate, is blocked/deferred, or when the next action changes
  materially. Treat returns as log events, not gate states; note
  blocked/deferred reasons in `Next / Waiting on`, not as separate states.
- **Gates loop; they are not a pipeline.** When a downstream gate overturns an
  assumption or surfaces a new unknown, return to the earliest gate the
  discovery overturns — usually ② Unknowns & Context; ① Intent itself when the
  problem definition fell — update it, then resume only the gates that depended
  on what changed. A return into ① or ② reopens Learn; do it in place in the
  active leaf's `01-Learn/` against the shared `references/gates.md` ①②, since
  that folder came with the leaf. If the discovery unsettles the whole idea, flag
  that the seed-level frame needs rethinking, not just a gate edit. Log each
  return to `04-Feedback/10-retrospective/mid-process-discoveries.md`. Linear
  one-pass progress through all ten gates is unrealistic for real work.
- **Phase transitions need explicit user approval by default; gate transitions
  inside the current phase are delegated unless they are high-impact.** AI may
  complete and consume ordinary gate artifacts inside the current phase after
  the gate authoring loop passes. The standing exception is Architect: before
  starting ⑧ Artifact / Execution, require explicit user approval of the
  promoted Architect snapshot -- ⑤ Design, ⑥ Critic verdict, ⑦ Task Graph,
  execution scope, risks, and the first execution chunk. Escalate a gate for
  explicit approval when it changes ① Intent, locks or changes decisive ③
  Criteria, needs an operator/reader to validate ④ Wireframe, commits a costly
  or public ⑤ Design choice, starts ⑧ Artifact / Execution, marks ⑧ Artifact as
  passed or delivered, returns across an approved phase boundary, or the user
  asks to review that gate. At approval points, AI proposes the next phase or
  promoted snapshot; the user decides. Skip the ⑧ start approval only when the
  user explicitly pre-authorized auto-execution for this leaf; if the work is so
  small or low-risk that this feels wasteful, do not invoke leaf-work.
- **Promote after Learn.** Seeds hold rough ideas and Learn-phase work, run in
  `leaf-idea`. When ① Intent and ② Unknowns & Context have passed there and the
  user approves moving to Example, `leaf promote <slug>` brings the work into this
  skill; continue from `.leaf/02-leaves/<slug>/`. ③ Criteria, ④ Wireframe,
  Architect, execution, and Feedback belong in active leaf storage, not
  `.leaf/01-seeds/`.
- **Persistent files live inside the leaf project folder.** Use
  `.leaf/01-seeds/<slug>/` for exploratory and Learn-phase work and
  `.leaf/02-leaves/<slug>/` for committed active work after `leaf promote <slug>`.
  Inside that project folder, keep the four phase folders and two-digit gate
  prefixes. For all naming and file-vs-folder-by-count, read
  `references/layout.md`.

## Response shape

Report per `leaf-soul` (overview-first, a **Verify / Decide** list up top, plain
words). When using this skill, report — starting with the opening preview once ①
has a one-sentence intent, then the working state.

**Opening preview** — phrase the four phases as the capability each builds *for
this specific intent*, not generic labels:

- **Learn:** by the end you can judge what this specific work needs — having
  learned the facts, conventions, and alternatives, not guessed them.
- **Example:** you can prove one cheap instance right before scaling it up.
- **Architect:** you can generalize that passed case into reusable structure,
  task order, and a shippable result.
- **Feedback:** you can confirm the plan still holds and carry forward what was
  established and what you learned.

It is orientation, not a fixed plan — keep it short and revise it when the intent
changes. Then report:

- the current phase and gate, plus the first missing gate, if any;
- why the next move belongs to Learn, Example, Architect, or Feedback;
- the proposed next artifact to create or revise;
- files or sections opened for user review, if any;
- open questions that block the next gate;
- a short task graph when entering Architect;
- the promoted Architect snapshot for explicit approval before ⑧ execution;
- review checks that prove the next pass is useful.

For small requests, compress this into a concise plan and draft. For substantial
documents, produce the planning artifact first and confirm before full drafting.

## The ③–⑤ engine (the heart)

The middle three gates — ③ Criteria, ④ Wireframe, ⑤ Design — are a single
**criteria → instance → generator chain**: ③ writes the test before any answer
exists, ④ locks one concrete instance and its contract, and ⑤ generalizes that
contract into every valid instance. The one inviolable rule is to never merge
across a produce/consume edge (③→④, ④→⑤) — keep them separate so their
disagreement stays visible. When you enter ③, read `references/engine.md` for
the full mechanics: contract, variation points, the falsification loop, and the
diagram.

## Reference map

| Read | When |
|---|---|
| `../leaf-soul/SKILL.md` | the shared conduct/voice: how to report, show rendered work, hand off reviewables, separate fact from guess, and which language to write |
| `references/gates.md` | when judging gate readiness, creating/revising a gate artifact, handling a return, or needing examples |
| `references/gate-authoring.md` | when drafting, grilling, revising, or presenting a gate artifact for review or approval |
| `references/engine.md` | you are inside ③–⑤ and need the full contract / variation point / generator mechanics + diagram |
| `references/clarity-ledger.md` | you are scoring criteria dimensions at ③ — or using it as a lens in ① / ② to aim the next question or learning gap |
| `references/experiment-log.md` | a gate's question needs an experiment — an independent, cheap probe that turns a guess into a fact you can't doubt: ② probing the world ("is this true?"), ④ probing one instance of the answer; gives the core, the fact/guess boundary, the fact ladder, and the technique repertoire |
| `references/decision-rationale.md` | you are inside ⑤ and a non-obvious choice needs durable rationale |
| `references/design-critic.md` | you are at ⑥ — every design gets at least a quick self-pass; read this for critic depth, output shape, or a durable critic pass |
| `references/brownfield-html-capture.md` | UI/web work needs a rendered ④ HTML view after the text-first pass — capture the real page (brownfield) or mock the sketch (greenfield), and render the decisive states; not a replacement for the text-first ④ |
| `references/task-pr-size-guidance.md` | you are slicing ⑦ tasks/PRs and need the reviewability size tripwires (small / medium / large-justified) |
| `references/layout.md` | you are writing files: naming, folder layout, and what each gate file records |
| `references/patterns.md` | you want a per-domain application template |
