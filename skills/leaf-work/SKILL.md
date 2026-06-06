---
name: leaf-work
description: |
  Use when turning a vague idea into structured knowledge work or writing —
  documents, essays, articles, memos, research papers, proposals, reports,
  specs, study notes, presentations, prototypes, or any non-code or mixed
  deliverable. Use when the request is vague, high-stakes, long-form,
  collaborative, research-heavy, or likely to sprawl, and when a rough intent
  must become an executable plan. Trigger on "어떤 순서", "글쓰기 프로세스",
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
| **Learn** | Judge what the work needs — learned, not guessed | ① Intent · ② Unknowns & Context |
| **Example** | Prove one cheap instance right before scaling | ③ Criteria · ④ Wireframe |
| **Architect** | Generalize that instance into a shippable generator | ⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact |
| **Feedback** | Confirm it still holds, then settle what was established and what was learned | ⑨ Review/sync · ⑩ Retrospect |

## Always-on rules

- **Scaffolding is the first act — so the work stands on a firm foundation.**
  Learn, Example, and Architect lay that foundation; execution builds only once
  it is solid, not on guesses. So invoking leaf-work means standing up the
  minimal scaffold first — the four phase folders (`01-Learn/` … `04-Feedback/`)
  and `00-status.md` at the root — before working any gate. The scaffold is the
  *body* of LEAF: it makes "which gate am I in / what is the first missing gate"
  a place you can point to, and each gate file fills in — laying the foundation
  course by course — as the work records itself and context settles. If a task
  is too small to deserve that foundation, do not invoke leaf-work at all — there
  is no LEAF without a body.
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
  against the gate's foci, revise, show only the review surface needed for
  approval, and wait for explicit confirmation before downstream gates consume
  it. See `references/gate-authoring.md`.
- **Keep `00-status.md` current.** Update it when a gate starts, becomes ready
  for approval, is approved, returns to an earlier gate, is blocked/deferred, or
  when the next action changes materially. Treat returns as log events, not gate
  states; note blocked/deferred reasons in `Next / Waiting on`, not as separate
  states.
- **Gates loop; they are not a pipeline.** When a downstream gate overturns an
  assumption or surfaces a new unknown, return to the earliest gate the
  discovery overturns — usually ② Unknowns & Context; ① Intent itself when the
  problem definition fell — update it, then resume only the gates that depended
  on what changed. Log each return
  to `04-Feedback/10-retrospective/mid-process-discoveries.md`. Linear one-pass
  progress through all ten gates is unrealistic for real work.
- **Every gate transition needs explicit user approval.** AI may *propose* a
  gate is ready ("the current unknowns have sourced answers; start ③?") or that
  an artifact should be promoted to a passed snapshot — but the user decides. AI
  never unilaterally declares a gate passed, moves forward, or returns to an
  earlier gate. Propose, the user decides.
- **Persistent files live under four phase folders.** Use `01-Learn/`,
  `02-Example/`, `03-Architect/`, and `04-Feedback/` at the top level. Inside
  each phase, files keep their two-digit gate prefix (`01-intent.md`,
  `02-unknowns.md`, `03-criteria.md`, `04-wireframe/`,
  `05-design-<artifact>.md`, …). For all naming and file-vs-folder-by-count,
  read `references/layout.md`.

## Response shape

When using this skill, report — starting with the opening preview once ① has a
one-sentence intent, then the working state.

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
- open questions that block the next gate;
- a short task graph when entering Architect;
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
| `references/gates.md` | when judging gate readiness, creating/revising a gate artifact, handling a return, or needing examples |
| `references/gate-authoring.md` | when drafting, grilling, revising, or presenting a gate artifact for approval |
| `references/engine.md` | you are inside ③–⑤ and need the full contract / variation point / generator mechanics + diagram |
| `references/clarity-ledger.md` | you are scoring criteria dimensions at ③ — or using it as a lens in ① / ② to aim the next question or learning gap |
| `references/decision-rationale.md` | you are inside ⑤ and a non-obvious choice needs durable rationale |
| `references/design-critic.md` | ⑥ triggers fired and you need to run a critic pass |
| `references/layout.md` | you are writing files: naming, folder layout, and what each gate file records |
| `references/patterns.md` | you want a per-domain application template |
