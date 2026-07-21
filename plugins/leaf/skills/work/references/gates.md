# Gates — Execution Checklist

Per-gate essence, pass conditions, and return conditions. Work the gates by
phase, stopping at the earliest missing gate.

```text
Learn:      ① Intent → ② Unknowns & context / reference exploration
Example:    ③ Criteria → ④ Wireframe with mock data
Architect:  ⑤ Design → ⑥ Critic → ⑦ Task graph → ⑧ Artifact / execution
Feedback:   ⑨ Review / sync → ⑩ Retrospect
```

**Gates ③–⑤ are a criteria → instance → generator engine**; the full mechanics
live in `engine.md`. Optional formats and per-domain templates: `patterns.md`.

**Adding a check to a gate requires recording, next to it, the actual failure
it prevents. Promoting a default (a patterns/reference-file format) into a
pass/fail condition counts as adding a check.** The Anti-Patterns section at
the end is the failure ledger those records live in.

---

## Gate folding (short loop)

**A gate always runs; what scales with the work is how much it produces.** ⑥
Critic already works this way — "always runs; only the depth scales." Fold
generalizes that grammar to ④, ⑤, and ⑦: when the uncertainty a gate exists to
close is already absent for *this* work, the gate is passed with a one-line
`folded:` record instead of a full artifact. **The floor is a one-line
judgment, never zero — folding shrinks the gate's output, not its judgment.**
So a folded gate is depth-minimal, not skipped; the file stays, its body
becomes one line. This is what makes work cost track problem size (the failure
it prevents: forcing eight full gate artifacts onto a one-word change).

**Which gates fold, and the "no uncertainty" condition each:**

| Gate | Folds when | The one line still carries |
|---|---|---|
| ④ Wireframe | instance = artifact — there is no separate cheap instance to validate (building one would make the deliverable twice) | the real asset/file inspected (≥1 concrete noun) |
| ⑤ Design | there is no variation axis to generalize (one value / one string swapped) | a one-line precedent declaration if a precedent surface exists, else `없음` |
| ⑥ Critic | *(no new clause — already depth-scaling)* | the existing quick self-pass line |
| ⑦ Tasks | one review unit (small on the `task-pr-size-guidance.md` tripwire) | the single task + its verification command |

**Never fold** ③ (it produces the fold judgment), ⑧ (the work itself), ⑨ (it
runs the fold audit), or ⑩ (it only gets shorter). ③–⑤ files still never merge
(`engine.md`); folding shortens a file, it does not delete or combine gate
files.

**Deciding, recording, and unfolding:**

- **Decide (not pure self-assessment).** ③'s end is the *earliest* fold
  proposal; each gate re-affirms its own fold on arrival (true per-gate depth,
  not one central switch). The agent proposes; a human approves. In
  interactive work that approval is at ③'s end. Under `autopilot` there is no
  ③-end pause, so fold eligibility is pre-approved at the triple lock (the
  approval point moves forward, like Shape Up's appetite fixed at intake);
  without that pre-approval autopilot runs the full loop.
- **Record (falsifiable).** `folded: <reason — ≥1 concrete noun: the file,
  asset, or rejected alternative> [+ one line of remaining judgment]`. A line
  that would copy-paste onto any task ("simple, so skipped") fails this and is
  not a fold.
- **Audit (post-hoc, in ⑨).** ⑨ adds a fold-audit line: does the actual
  diff/artifact match ③'s fold reason (e.g. "one review unit")? ⑨ also
  inherits any check a folded ④ deferred — the artifact against the real
  asset's constraints. A mismatch unfolds the gate.
- **Unfold (a normal path, not a failure).** When a fold reason breaks, expand
  that gate and record the return, following the existing return grammar: a
  surfaced **variation axis → unfold ④ first** (declare the contract), then ⑤
  if needed — never let ⑤ invent the contract; **size over one review unit →
  unfold ⑦**. Unfolding is the ordinary falsification of an assumption the
  fold consumed.

---

## ① Intent

① belongs to `learn`; its contract is
`../../learn/references/gate-01-intent.md`. `work` consumes ① only after Learn
passes. If ③ or later reveals the intent is wrong, return to `learn`.

## ② Unknowns & Context

② belongs to `learn`; its contract is
`../../learn/references/gate-02-unknowns-context.md`. `work` consumes ② as the
basis for ③ Criteria; if the facts and assumptions ③ needs are missing, return
to Learn instead of inventing criteria here.

## ③ Criteria

**State the change the work must cause (purpose) and the observable checks
that make a concrete instance pass or fail.** Purpose is authored furthest
from the solution, so it is the arbiter inside the ③–⑤ engine; requirements
are the checks that can reject a bad instance.

Gate to continue:

- Purpose is one sentence describing the intended effect, not the artifact
  shape.
- Success checks describe the effect and are concrete enough to reject a bad
  wireframe.
- Non-goals and tradeoff principles are explicit when they matter.
- When the request has multiple possible outcomes, the topology — independent
  outcomes, surfaces, or deliverables that can succeed or fail independently —
  is named or explicitly deferred.
- The output form is consumed from the Learn-locked `what`; needing a
  different form is a return to `learn`, not a silent override.

Check-writing forms (EARS two-column table, writing-friendly forms): see
`patterns.md`. The Clarity Ledger (`clarity-ledger.md`) is a lens for aiming
these checks, not a required score sheet.

Return here when ④'s instance falsifies a criterion — the purpose arbitrates;
when the purpose itself is disputed, the appeal climbs to ①'s why.

## ④ Wireframe

**Validate one cheap concrete instance against ③ before generalizing — the
instance can falsify a criterion, and every placeholder in it must trace to a
declared contract.** This is the answer probe: ② experiments on the world
("is this true?"), ④ experiments on your answer ("is this answer right?").

Folds when the instance *is* the artifact (see Gate folding); the folded line
still names the real asset inspected.

Gate to continue:

- The text-first wireframe passed before any artifact-specific wireframe or
  generalized design.
- The actual operator(s)/reader(s) walked through it and confirmed fit.
- Mock data is realistic enough to expose edge cases (empty, error, large,
  multi-language), not just the happy path.
- Every placeholder is traced to a declared contract (declarative or
  ostensive) and checked against the real asset's constraints.

Rendered HTML views for UI work: `brownfield-html-capture.md`. Deeper probe
techniques (cold reader, decisive states, experiment machinery):
`experiment-log.md` and `patterns.md` — defaults, never conditions.

Return to ④ when ⑧ drafting reveals the structure does not match the
workflow, ⑤ has to invent a schema the contract should have locked, or a ⑨
reviewer cannot find their way — and record the return.

## ⑤ Design — the generator

**Generalize the validated instance into the rules that produce every valid
instance the ④ contract allows — consuming that contract, never rediscovering
it.**

Folds when there is no variation axis to generalize (see Gate folding); the
folded line still declares any precedent surface the change sets.

Gate to continue:

- Each section or component has a role, and the reader can follow why the
  order works.
- The design matches the criteria and the validated wireframe.
- It explains behavior across each variation point's full range — empty,
  overflow, edge, timing, failure — not just the validated case.
- Precedent Fit is recorded when the design introduces or changes names,
  structure, placement, taxonomy, workflow, or interface shape that future
  work may copy, and each main artifact has a one-sentence responsibility
  statement (a strained multi-clause statement is a split signal). ⑤ is the
  sole owner of this concern.
- Brownfield claims about the current system are cheap-checked against actual
  code, docs, or rendered behavior — never designed against a remembered
  system.

Before/after architecture sketches: `engine.md`. Durable decision rationale
(RALPLAN-DR): `decision-rationale.md`. Responsibility sidecars: the
`leaf sidecar` skill. All defaults, never conditions.

Return to ④ when a new variation axis surfaces that the contract never
declared.

## ⑥ Critic

**Falsify the ⑤ generalization before ⑦ builds on it — it always runs; only
the depth scales with risk** (security, irreversibility, public precedent,
cross-module coupling → deep pass; low stakes → quick self-pass).

Gate to continue:

- Verdict is `APPROVE`, or `ITERATE` revisions have been applied and
  re-reviewed; `REJECT` returns to ⑤ (or ② when an assumption was
  overturned).
- The critic explicitly checks whether the design is the best available
  answer to the why locked at ①, not merely whether it satisfies the drafted
  ③ criteria.
- Even a quick self-pass leaves a one-line record in `06-critic.md`, so the
  pass is visible rather than silently skipped.

Full review criteria and depth triggers: `design-critic.md`.

## ⑦ Task Graph

**Slice the work into reviewable units whose dependencies are real, not
conversational.**

Folds when the work is one review unit (see Gate folding); the folded line
still names the single task and its verification.

Gate to continue:

- Each slice is independently reviewable — size labeled against the tripwires
  in `task-pr-size-guidance.md` (small / medium / large-justified; crossing a
  tripwire means justify or split).
- Dependencies are real, and parallelizable work is visible from them.
- Each task has a check that proves it helped.

Split oversized work inside `07-tasks.md`; a truly independent LEAF cycle
becomes a sibling sprout, never a nested folder.

## ⑧ Artifact / Execution

**Produce the result — after explicit user approval of the Architect snapshot
(⑤ design, ⑥ verdict, ⑦ tasks, scope, risks, first chunk) — and log each
session so the work can stop and resume.** The artifact lives wherever you
keep your work; `08-execution.md` records what was done, one entry per
session. Start with the most load-bearing uncertain chunk; prefer ugly but
checkable work over polished unsupported prose.

Gate to continue:

- The result can be reviewed against criteria.
- Open placeholders are visible and honestly marked.
- The next chunk's inputs are known.

Record ⑧ as passed only when the user explicitly says so. Then move the
project folder to `.leaf/02-leaves/<slug>/`, update `00-status.md` for
Feedback, and run `leaf doctor` before ⑨.

## ⑨ Review / Sync

**Confirm the artifact satisfies the criteria, and bring the plan documents
back to the truth the artifact revealed.** Runs in `.leaf/02-leaves/<slug>/`;
may loop with ⑧ on defects.

Gate to continue:

- The artifact satisfies the criteria, with claims supported by evidence.
- The sync rule has been applied: a changed claim updates criteria; a changed
  flow or precedent surface updates the design (including the precedent and
  responsibility statements ⑤ left); changed workload updates the task graph.
  No stale planning notes remain beside a changed draft.
- Fold audit (only when a gate folded): the actual diff/artifact matches each
  `folded:` reason (e.g. "one review unit"), and any check a folded ④ deferred
  — the artifact against the real asset's constraints — has been paid here. A
  mismatch unfolds that gate and records the return.

Domain-quality lenses (council or persona skills, when exposed) augment these
checks; they never replace them.

## ⑩ Retrospect

**Close the leaf on two axes: what this work established and where its
authority ends (content), and what should change in the next loop (process).**

Gate to continue:

- Limitations — established claims, unresolved questions, boundary conditions
  — and Lessons — reusable process changes — are recorded separately, plainly
  enough for future work to cite without rereading the leaf.
- The lessons feed forward: mid-process discoveries are checked for what ②
  should have caught, and the findings update the next work's ② approach (and
  `.leaf/PROFILE.md` via the `profile` skill when they should apply across
  leaves).

⑩ passing does not end the work: immediately follow `using-leaf` ("Ending a
leaf") to decide keep / press / fall.

---

## Anti-Patterns — the failure ledger

Recorded failures that existing checks prevent; new checks must add their
failure here or next to the check itself.

- Searching, benchmarking, or researching before listing what is unknown.
- Treating mid-work research detours as expected, rather than as a signal that
  ② was incomplete.
- Drafting polished prose before criteria are clear.
- Building the wireframe before criteria are clear.
- Treating table of contents as a task graph.
- Letting AI invent evidence or audience.
- Keeping an old outline after review changes the thesis.
- Doing "final polish" while claims are still unsupported.
- ⑤ inventing a schema/shape the ④ contract should have locked.
- Starting ⑨ Review while the passed work is still under `.leaf/01-sprouts/`.
- Stopping after the retrospect file is written, leaving the leaf without a
  keep/press/fall decision.
- Forcing full gate artifacts onto work with no uncertainty to close — the cost
  a fold prevents. (And its inverse: folding a gate whose uncertainty is real,
  or with a copy-paste reason that names no concrete noun.)
