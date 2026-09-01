---
name: work
description: |
  Use to carry a LEAF sprout after Learn from ③ Example through a shipped result:
  validate one cheap instance against criteria, design the generator, run the critic,
  slice the task graph, draft or execute, and review. Use for documents, essays,
  articles, memos, research papers, proposals, reports, specs, study notes,
  presentations, prototypes, code, or mixed deliverables in a repo-local `.leaf/`
  workspace. Enter after ① Intent and ② Unknowns & Context pass in `learn`.
  Trigger on criteria, 와이어프레임, 설계, task graph, 작업 쪼개기, 초안 작성,
  실행, 리뷰, or 검토 절차. For vague, early, or idea-stage work, use `learn`
  first.
---

# LEAF Work

**Leaf before tree.** Validate one cheap, inspectable instance before growing
it into the whole artifact. The core move is never to **generate the whole
artifact upfront** — that is how you produce confident-looking slop and lose the
way before you can even tell the direction is wrong. It is to **learn first, make
one instance right, then expand** — closing the cheapest decisive uncertainty
before starting the next kind of work.

LEAF closes four kinds of uncertainty in order:

| Phase | What it makes you able to do | Gates |
|---|---|---|
| **Learn** | Judge what the work needs — learned, not guessed *(run in `learn`; inherited here)* | ① Intent · ② Unknowns & Context |
| **Example** | Prove one cheap instance right before scaling | ③ Criteria · ④ Wireframe |
| **Architect** | Generalize that instance into a shippable generator | ⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact |
| **Feedback** | Confirm it still holds, then settle what was established and what was learned | ⑨ Review/sync · ⑩ Retrospect |

`work` normally starts after `learn` has passed ① Intent and ② Unknowns &
Context. It carries the same sprout from ③ through ⑧, moves passed work to
`.leaf/02-leaves/` before ⑨, runs ⑨/⑩, then follows `using-leaf`
("Ending a leaf") to keep, press, or fall. An execution-ready implementation
uses the narrower execution-first lane below and does not enter the normal
lifecycle unless new uncertainty forces an escalation.

## Fast-track lifecycle lane

When `using-leaf` selects fast-track, this is still normal LEAF gate work, not
the execution-first lane. Read `../using-leaf/references/fast-track.md`, consume
its request-scoped procedure budget, and keep every gate's pass/return and safety
contract. The budget reduces ceremony; it does not make validation optional.

## Execution-first lane

Use this lane only when `using-leaf`'s five execution-ready conditions hold and
the user has requested the implementation. The user request authorizes one
in-scope, local, reversible first experiment; it does not relax any hard stop.

1. Perform one bounded **저장소 상태 확인** against the request.
2. Make **최초 실행 증거**: a failing regression test, reproduction, measurement,
   or minimal prototype.
3. Repeat **작고 되돌릴 수 있는 구현과 검증** while that evidence stays observable.
4. Record only **실제로 생긴 결정·위험·부채** in the nearest existing delivery
   surface. Then perform final verification and any route-appropriate lightweight
   implementation review or retrospect that can still change the result, using
   that same surface, before handoff.

## Fast terminal

보존할 결정·위험·부채가 없으면 최종 검증과 해당하는 경량 구현
review/retrospect 결과를 handoff하고 종료한다.
**최초 실행 증거 뒤에도** 이 작업만을 위한 phase gate, checkpoint, cumulative polish, 독립 문서
검토, live UI를 만들거나 실행하지 않고, **`.leaf/` scaffold를 만들지 않는다**.
완료된 구현을 사후에 ③–⑩으로 재구성하는 것도 금지한다.

기록할 항목이 있더라도 먼저 기존 issue, PR, commit, final handoff에 남긴다.
실행 결과가 security, privacy, permission,
legal, public-contract, irreversible, external, costly, deployment, or
large-structure 미결정 사항을 드러내 추가 발견·설계가 필요할 때만 direct path를
멈추고 Learn부터 normal Work로 승격한다. 사용자가 durable LEAF 기록 자체를
명시적으로 요청한 경우도 같은 정상 진입 경로를 사용한다.

## Always-on rules

- **Use the leaf CLI as the body.** Normal Work lives in one `.leaf/` project
  folder. If no matching sprout has passed ①/②, invoke `learn`; do not improvise
  post-Learn gates or create loose phase folders. The execution-first lane does
  not create a project folder unless it escalates and restarts at Learn.
- **Inherit Learn in normal Work.** Start from `00-status.md`, then read ①
  Intent and ② Unknowns & Context. Trust them unless a downstream gate forces a
  return. The execution-first lane consumes its five-condition routing judgment
  and request until its evidence has been made.
- **Conduct and voice come from `soul`.** Invoke `soul` at the start
  and follow it for reporting, language, fact/guess boundaries, review handoff,
  and rendered artifact display. `work` owns gate method and progress.
  Whenever work needs another LEAF skill — `learn` for Learn, `polish`
  for document cleanup, or `press` to press a reference-worthy leaf — invoke
  that skill rather than only referencing its file. The keep/press/fall
  decision and the fall and keep actions live in `using-leaf`
  ("Ending a leaf"). Use `profile` when `.leaf/PROFILE.md` needs to be read
  or updated.
- **Act by the relevant gate reference in normal Work.** Identify the current gate, read its
  reference, tell the user the gate, and follow its pass/return conditions.
  For ①/② returns use `../learn/references/gate-01-intent.md` and
  `../learn/references/gate-02-unknowns-context.md`; use `references/gates.md`
  for ③ onward.
- **Polish at each formal phase boundary, then cross it with `leaf next`.** At
  every formal phase boundary (the end of Learn, Example, Architect, Feedback,
  and before close-out), run `leaf checkpoint <slug> --<gate>` on the gate files.
  Discovery-heavy Work invokes `polish` on the cumulative whole. Fast-track
  performs a lightweight cumulative self-polish and invokes full `polish` plus
  its independent reviewer only when its budget trigger is present. Either pass
  removes the phase's
  `<!-- leaf:polish-pending -->` marker; then run `leaf next <slug>` to advance —
  and show the result per `soul` only when its display condition holds. `leaf
  next` is the boundary event: if the phase is still unpolished it
  **pauses (멈칫)** asking you to polish, and `leaf doctor` flags any skipped
  boundary as `boundary_unpolished`. A gate with an in-phase user review (e.g. ④
  Wireframe) may get a local polish just before it.
- **Keep status current.** Update `00-status.md` whenever phase, gate, next
  action, approval need, return, or closure state changes. Keep its `## Overview`
  aligned with the gate files: when a gate or return changes the purpose, scope,
  expected output, or split decision, revise the overview in the same pass. Gate
  files remain authoritative.
- **Return early when facts change.** Gates loop. Return to the earliest gate
  invalidated by a discovery and resume only the dependent gates. Log each
  return to `04-Feedback/10-retrospective/mid-process-discoveries.md` so ⑩
  Retrospect can review it.
- **Ask at approval points.** Ordinary gates inside a phase may proceed after
  self-review. Phase boundaries, high-impact gates, and ⑧ start/pass need
  explicit user approval unless pre-authorized. An execution-ready user request
  pre-authorizes its in-scope reversible implementation; external or destructive
  effects still require their own explicit authorization.
- **Move and close.** After ⑧ is explicitly passed or delivered, move the same
  folder from `.leaf/01-sprouts/` to `.leaf/02-leaves/`, update status, and run
  `leaf doctor`. After ⑩ passes, follow `using-leaf` ("Ending a leaf") to
  keep, press, or fall.
- **Fold a gate with no uncertainty to close.** A gate always runs, but when
  ④/⑤/⑦ has no uncertainty left to close, pass it with a one-line `folded:`
  record (naming a concrete noun) instead of a full artifact — after the human approves the fold (at ③'s end interactively, or
  pre-approved at the triple under `autopilot`). Full
  rules — which gates fold, the record schema, and the unfold path — live in
  `references/gates.md` → Gate folding.
- **Use `references/layout.md` before writing files.** It owns folder layout,
  gate filenames, status values, and file-vs-folder rules.

## Response shape

Report per `soul`, then add only the `work` state the user needs:

- current phase/gate and first missing gate;
- why the next move belongs to this phase;
- next artifact or action, plus any blocking review decision;
- task graph or approved Architect snapshot only when entering execution.

For small requests, compress to conclusion plus next action. For substantial
documents, produce the gate artifact first and confirm before full drafting.

## The ③–⑤ engine (the heart)

③ Criteria, ④ Wireframe, and ⑤ Design are one **criteria → instance →
generator** chain. Do not merge across produce/consume edges (③→④, ④→⑤);
disagreement must stay visible. When entering ③–⑤, read `references/engine.md`.

## Reference map

| Read | When |
|---|---|
| `../soul/SKILL.md` | shared conduct, voice, review handoff |
| `../profile/SKILL.md` | effective profile and profile updates |
| `references/gates.md` | when judging gate readiness, creating/revising a gate artifact, handling a return, or needing examples |
| `references/gate-authoring.md` | when drafting, grilling, revising, or presenting a gate artifact for review or approval |
| `references/engine.md` | you are inside ③–⑤ and need the full contract / variation point / generator mechanics + diagram |
| `references/loop-contract.md` | a gate needs repeated passes and you must decide whether it is actually loop-shaped, then define observe/choose/act/verify/record/stop behavior |
| `references/clarity-ledger.md` | you want the five-dimension lens for aiming ③ checks or a ① / ② question at the weakest row |
| `references/experiment-log.md` | a gate's question needs an experiment — an independent, cheap probe that turns a guess into a fact you can't doubt: ② probing the world ("is this true?"), ④ probing one instance of the answer; gives the core, the fact/guess boundary, the fact ladder, and the technique repertoire |
| `references/decision-rationale.md` | you are inside ⑤ and a non-obvious choice needs durable rationale |
| `references/design-critic.md` | you are at ⑥ — every design gets at least a quick self-pass; read this for critic depth, output shape, or a durable critic pass |
| `references/brownfield-html-capture.md` | UI/web work needs a rendered ④ HTML view after the text-first pass — for brownfield UI, anchor in the real screen with pins + close-up previews; for greenfield, mock the sketch; render decisive states; not a replacement for the text-first ④ |
| `references/task-pr-size-guidance.md` | you are slicing ⑦ tasks/PRs and need the reviewability size tripwires (small / medium / large-justified) |
| `references/layout.md` | you are writing files: naming, folder layout, and what each gate file records |
| `references/patterns.md` | you want a per-domain application template |
