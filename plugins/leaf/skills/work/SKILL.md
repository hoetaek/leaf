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

`work` starts after `learn` has passed ① Intent and ② Unknowns &
Context. It carries the same sprout from ③ through ⑧, moves passed work to
`.leaf/02-leaves/` before ⑨, runs ⑨/⑩, then follows `using-leaf`
("Ending a leaf") to keep, press, or fall.

## Always-on rules

- **Use the leaf CLI as the body.** Work lives in one `.leaf/` project folder.
  If no matching sprout has passed ①/②, invoke `learn`; do not improvise
  post-Learn gates or create loose phase folders.
- **Inherit Learn.** Start from `00-status.md`, then read ① Intent and ②
  Unknowns & Context. Trust them unless a downstream gate forces a return.
- **Conduct and voice come from `soul`.** Invoke `soul` at the start
  and follow it for reporting, language, fact/guess boundaries, review handoff,
  and rendered artifact display. `work` owns gate method and progress.
  Whenever work needs another LEAF skill — `learn` for Learn, `polish`
  for document cleanup, or `press` to press a reference-worthy leaf — invoke
  that skill rather than only referencing its file. The keep/press/fall
  decision and the fall and keep actions live in `using-leaf`
  ("Ending a leaf"). Use `profile` when `.leaf/PROFILE.md` needs to be read
  or updated.
- **Act by the relevant gate reference.** Identify the current gate, read its
  reference, tell the user the gate, and follow its pass/return conditions.
  For ①/② returns use `../learn/references/gate-01-intent.md` and
  `../learn/references/gate-02-unknowns-context.md`; use `references/gates.md`
  for ③ onward.
- **Polish at each phase boundary, then cross it with `leaf next`.** At every
  phase boundary (the end of Learn, Example, Architect, Feedback, and before
  close-out), run `leaf checkpoint <slug> --<gate>` on the gate files to polish,
  then invoke `polish` on the cumulative whole — all phases as
  one connected report, not just the latest. Polishing removes the phase's
  `<!-- leaf:polish-pending -->` marker; then run `leaf next <slug>` to advance —
  and open the result for the user to review per `soul` (live web UI), not a path
  to chase. `leaf next` is the boundary event: if the phase is still unpolished it
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
  explicit user approval unless pre-authorized.
- **Move and close.** After ⑧ is explicitly passed or delivered, move the same
  folder from `.leaf/01-sprouts/` to `.leaf/02-leaves/`, update status, and run
  `leaf doctor`. After ⑩ passes, follow `using-leaf` ("Ending a leaf") to
  keep, press, or fall.
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
| `references/clarity-ledger.md` | you are scoring criteria dimensions at ③ — or using it as a lens in ① / ② to aim the next question or learning gap |
| `references/experiment-log.md` | a gate's question needs an experiment — an independent, cheap probe that turns a guess into a fact you can't doubt: ② probing the world ("is this true?"), ④ probing one instance of the answer; gives the core, the fact/guess boundary, the fact ladder, and the technique repertoire |
| `references/decision-rationale.md` | you are inside ⑤ and a non-obvious choice needs durable rationale |
| `references/design-critic.md` | you are at ⑥ — every design gets at least a quick self-pass; read this for critic depth, output shape, or a durable critic pass |
| `references/brownfield-html-capture.md` | UI/web work needs a rendered ④ HTML view after the text-first pass — for brownfield UI, anchor in the real screen with pins + close-up previews; for greenfield, mock the sketch; render decisive states; not a replacement for the text-first ④ |
| `references/task-pr-size-guidance.md` | you are slicing ⑦ tasks/PRs and need the reviewability size tripwires (small / medium / large-justified) |
| `references/layout.md` | you are writing files: naming, folder layout, and what each gate file records |
| `references/patterns.md` | you want a per-domain application template |
