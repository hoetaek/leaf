---
name: leaf-work
description: |
  Use to carry a LEAF sprout after Learn from ③ Example through a shipped result:
  prove one cheap instance against criteria, design the generator, run the critic,
  slice the task graph, draft or execute, and review. Use for documents, essays,
  articles, memos, research papers, proposals, reports, specs, study notes,
  presentations, prototypes, code, or mixed deliverables in a repo-local `.leaf/`
  workspace. Enter after ① Intent and ② Unknowns & Context pass in `leaf-idea`.
  Trigger on criteria, 와이어프레임, 설계, task graph, 작업 쪼개기, 초안 작성,
  실행, 리뷰, or 검토 절차. For vague, early, or idea-stage work, use `leaf-idea`
  first.
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
| **Learn** | Judge what the work needs — learned, not guessed *(run in `leaf-idea`; inherited here)* | ① Intent · ② Unknowns & Context |
| **Example** | Prove one cheap instance right before scaling | ③ Criteria · ④ Wireframe |
| **Architect** | Generalize that instance into a shippable generator | ⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact |
| **Feedback** | Confirm it still holds, then settle what was established and what was learned | ⑨ Review/sync · ⑩ Retrospect |

Learn runs in `leaf-idea`; `leaf-work` continues the same sprout at ③ Example
after Learn passes, carries it through ⑧ Artifact / Execution, then invokes
`leaf-clean` to move the passed sprout into `.leaf/02-leaves/` before ⑨ Review.
Learn's detailed gate contracts live in
`../leaf-idea/references/gate-01-intent.md` and
`../leaf-idea/references/gate-02-unknowns-context.md`; `references/gates.md`
covers the handoff plus ③ onward. ⑩ Retrospect invokes `leaf-profile` when the
leaf reveals a user language preference, recurring requirement, agent mistake,
wrong-answer note, or cross-leaf fact that belongs in `.leaf/PROFILE.md`.

## Always-on rules

- **Use the leaf CLI as the persistent body of the work.** `leaf-work` assumes
  work lives in one repo-local `.leaf/` project folder. Work remains a sprout
  through Learn, Example, Architect, and ⑧ Artifact / Execution. Once ⑧ is
  explicitly passed or delivered, invoke `leaf-clean` to move that same folder
  from `.leaf/01-sprouts/` to `.leaf/02-leaves/`; ⑨ Review and ⑩ Retrospect
  then continue in the leaf folder. If the work
  has not been through Learn — no sprout exists, or ① and ② have not passed —
  **invoke the `leaf-idea` skill with the Skill tool** to run Learn first rather
  than starting gate work here; being entered without Learn is the signal to hand
  off to `leaf-idea`, not to improvise the missing gates. If a matching sprout
  already exists, resume it. Do not create loose `01-Learn/` … `04-Feedback/`
  folders at the repo root.
- **Inherit Learn; do not redo it.** The sprout already has ① Intent locked and
  ② Unknowns & Context resolved in `01-Learn/`. Start by reading them: the
  one-sentence intent and its why, sourced facts, flagged assumptions, gathered
  references, and what the user can already choose between. Trust that work and
  reopen Learn only on a return.
- **Conduct and voice are shared — invoke the `leaf-soul` skill.** How you report
  (overview-first, a Verify / Decide list up top), show rendered HTML work (open it
  in Chrome DevTools and screenshot each state — never make the user hunt), hand
  off reviewables (`USER REVIEW NEEDED:` / `ASSUMPTION:`), separate fact from
  guess, and which language you write are defined once for the whole LEAF family in
  the `leaf-soul` skill. Invoke the `leaf-soul` skill with the Skill tool at the
  start and follow it — do not just read its file; the rules here are the gate
  *method* specific to this skill. Whenever the work needs another LEAF skill —
  `leaf-idea` for Learn, `leaf-clean` to move a passed sprout into leaves, make
  it citable, or retire it — invoke that skill with the Skill tool rather than
  only referencing its file. Use `leaf-profile` when `.leaf/PROFILE.md` needs to
  be read or updated.
- **The scaffold is the foundation — so the work stands firm.** Learn, Example,
  and Architect lay that foundation; execution builds only once it is solid, not
  on guesses. The sprout carries the CLI scaffold — `00-status.md` and the four
  phase folders — created in `leaf-idea` via `leaf new`; you inherit it, you do
  not recreate it. The scaffold is the *body* of LEAF: it makes "which gate am I
  in / what is the first missing gate" a place you can point to, and each gate
  file fills in as context settles. If a task is too small to deserve that
  foundation, do not invoke leaf-work at all — there is no LEAF without a body.
- **Start from `00-status.md` when it exists.** Use it as the project dashboard:
  current phase/gate, first missing gate, next action, and gate progress. It is
  an index, not the source of truth; gate files remain authoritative.
- **Read the relevant gate reference, tell the user the current gate, and act by it.**
  With the scaffold and `00-status.md` in hand, identify which gate the work is
  at, open the reference that owns that gate, name the gate to the user, and then
  proceed exactly as that gate's entry/exit/return conditions direct. For ①/②
  returns, use `../leaf-idea/references/gate-01-intent.md` for ① and
  `../leaf-idea/references/gate-02-unknowns-context.md` for ②; for ③ onward,
  use `references/gates.md`. SKILL.md gives the shape; the relevant gate
  reference gives the pass/fail test you act on.
- **Use the gate authoring loop for durable gate files.** When creating or
  revising a gate artifact, draft the smallest useful version, challenge it
  against the gate's foci, revise, and show only the review surface needed for
  the current approval policy. See `references/gate-authoring.md`.
- **Keep `00-status.md` current.** Update it when a gate starts, becomes ready
  for review, completes, needs explicit approval, is approved, returns to an
  earlier gate, is blocked/deferred, or when the next action changes
  materially. Keep its `## Overview` aligned with the gate files: if Criteria,
  Wireframe, Design, Tasks, Review, or a return changes the purpose, expected
  output, scope, split decision, or what the LEAF is doing, update the overview
  in the same pass. Treat returns as log events, not gate statuses; note
  blocked/deferred reasons in `Next / Waiting on`, not as separate statuses.
- **Gates loop; they are not a pipeline.** When a downstream gate overturns an
  assumption or surfaces a new unknown, return to the earliest gate the
  discovery overturns — usually ② Unknowns & Context; ① Intent itself when the
  problem definition fell — update it, then resume only the gates that depended
  on what changed. A return into ① or ② reopens Learn; do it in place in the
  sprout's `01-Learn/` against `../leaf-idea/references/gate-01-intent.md` or
  `../leaf-idea/references/gate-02-unknowns-context.md`. If the discovery
  unsettles the whole idea, flag that the sprout frame needs rethinking, not just
  a gate edit.
  Log each return to `04-Feedback/10-retrospective/mid-process-discoveries.md`.
  Linear one-pass progress through all ten gates is unrealistic for real work.
- **Phase transitions need explicit user approval by default; gate transitions
  inside the current phase are delegated unless they are high-impact.** AI may
  complete and consume ordinary gate artifacts inside the current phase after
  the gate authoring loop passes. The standing exception is Architect: before
  starting ⑧ Artifact / Execution, require explicit user approval of the
  approved Architect snapshot -- ⑤ Design, ⑥ Critic verdict, ⑦ Task Graph,
  execution scope, risks, and the first execution chunk. Escalate a gate for
  explicit approval when it changes ① Intent, locks or changes decisive ③
  Criteria, needs an operator/reader to validate ④ Wireframe, commits a costly
  or public ⑤ Design choice, starts ⑧ Artifact / Execution, marks ⑧ Artifact as
  passed or delivered, returns across an approved phase boundary, or the user
  asks to review that gate. At approval points, AI proposes the next phase or
  approved snapshot; the user decides. Skip the ⑧ start approval only when the
  user explicitly pre-authorized auto-execution for this sprout; if the work is so
  small or low-risk that this feels wasteful, do not invoke leaf-work.
- **Move to leaves after ⑧, then finish with leaf-clean.** ③ Criteria,
  ④ Wireframe, Architect, and ⑧ Artifact / Execution continue in the sprout
  project folder that Learn created. Immediately after ⑧ is explicitly passed
  or delivered, invoke the `leaf-clean` skill with the Skill tool to complete the
  stage transition from `.leaf/01-sprouts/<slug>/` to
  `.leaf/02-leaves/<slug>/`; do not hand-move the folder yourself. After that,
  run ⑨ Review / Sync and ⑩ Retrospect in the leaf folder. Immediately after
  ⑩ Retrospect passes, decide explicitly with the user whether the leaf should be
  pressed as a citable reference or moved to fallen as not worth carrying. Invoke
  `leaf-clean` again for that press/fall close-out. `leaf-work` is not complete
  until both the post-⑧ leaf transition and the post-⑩ close-out decision have
  been handled.
- **Persistent files live inside one project folder.** Keep the four phase
  folders and two-digit gate prefixes inside the sprout project folder. For all
  naming and file-vs-folder-by-count, read `references/layout.md`.

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
- the approved Architect snapshot for explicit approval before ⑧ execution;
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
| `../leaf-profile/SKILL.md` | the repo-local acquired profile: user language, recurring requirements, mistakes, wrong-answer notes, and facts that apply across leaf work |
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
