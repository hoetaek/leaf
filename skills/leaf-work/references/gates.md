# Gates — Execution Checklist

Per-gate entry/exit/return conditions and examples. Use when a request is vague,
high-stakes, long-form, collaborative, research-heavy, or likely to sprawl. Work
the gates by phase, stopping at the earliest missing gate.

```text
Learn:      ① Intent → ② Unknowns & context / reference exploration
Example:    ③ Criteria → ④ Wireframe with mock data
Architect:  ⑤ Design → ⑥ Critic → ⑦ Task graph → ⑧ Artifact / execution
Feedback:   ⑨ Review / sync → ⑩ Retrospect
```

**Gates ③–⑤ are a criteria → instance → generator engine.** This file gives
their gate conditions; the full mechanics (contract, variation point, generator,
falsification, merge rule) live in `engine.md`. For per-domain templates, see
`patterns.md`.

---

## ① Intent

① belongs to `leaf-idea`. Its detailed contract lives in
`../../leaf-idea/references/gate-01-intent.md`; use that file as the pass/fail
test for raw wording, why/what separation, provisional what, locked intent,
split checks, and return conditions.

`leaf-work` consumes ① only after Learn passes. If ③ or later reveals that the
intent is wrong, return to `leaf-idea` or revise the sprout's Learn files
against `../../leaf-idea/references/gate-01-intent.md` before continuing.

## ② Unknowns & Context

② belongs to `leaf-idea`. Its detailed contract lives in
`../../leaf-idea/references/gate-02-unknowns-context.md`; use that file as the
pass/fail test for unknown categories, reference exploration, experiment
boundaries, fact vs assumption labels, the Premise Inventory, and the
Learn-close condition.

`leaf-work` consumes ② as the basis for ③ Criteria: what the user can now choose
between, what facts support the choice, and which explicit assumptions or user
review items remain. If those are missing, return to Learn instead of inventing
criteria here.

## ③ Criteria

State the change the work is meant to cause and what must be true for the
concrete example to pass. Criteria combines the old Purpose and Requirements
gates because both are pre-instance judgment: the intended effect and the
checks that make that effect observable.

Criteria has two parts:

- **Purpose / arbiter** — one sentence describing the intended effect, not the
  artifact shape. It answers the necessity locked at ①: the problem lives
  upstream, purpose is the change that resolves it.
- **Requirements / test** — observable acceptance checks, evidence needs, scope,
  tone, format, deadline, non-goals, quality bars, and tradeoff principles.

**Score the Clarity Ledger before locking criteria.** Its five rows are exactly
the dimensions criteria must pin down — Intent → Purpose; Topology, Success,
Constraints, Output form → Requirements:

| Row | Stable? |
|---|---|
| Intent — necessity (why), desired effect, and core noun | … |
| Topology — independent outcomes/components/surfaces/deliverables named or deferred | … |
| Success — completion is observable | … |
| Constraints — non-goals, boundaries, preserved behavior | … |
| Output form — artifact, lifecycle, handoff shape | … |

A row is stable when named or explicitly deferred — never when silently
mutating. Any row that cannot be locked becomes an explicit risk or assumption
in the requirements, not vague wording. See `references/clarity-ledger.md`.

Gate to continue:

- Purpose is one sentence.
- Success criteria describe the intended effect, not just the artifact shape.
- Acceptance/evaluation checks are clear enough to reject a bad wireframe.
- Major claims have evidence needs.
- Unknowns are either assumptions, research tasks, or reviewer questions.
- Non-goals and tradeoff principles are explicit when they matter.
- Every Clarity Ledger row is named or explicitly deferred; any that cannot be
  locked is surfaced as an explicit risk or assumption, not hidden inside vague
  requirements.

Write acceptance checks in a two-column form when EARS helps but raw EARS would
be hard to scan. The left side is the human-readable check; the right side keeps
the testable EARS-style form:

```markdown
| Plain check | EARS |
|---|---|
| <reader-friendly acceptance check> | WHEN <trigger>, THE SYSTEM SHALL <observable behavior>. |
```

Use the EARS column for behavior, trigger, condition, preserved behavior, or
regression-sensitive checks. Keep a plain check even when the EARS sentence is
precise; criteria must be readable by the reviewer, not only testable by the
implementer.

Writing-friendly criteria forms:

```text
WHEN a reader finishes the document, THEY SHOULD be able to <understanding/action>.
GIVEN <audience/context>, THE DOCUMENT SHALL explain <concept> without assuming <missing knowledge>.
THE DRAFT SHALL support <claim> with <evidence type>.
THE DOCUMENT SHALL NOT cover <non-goal>.
```

When behavior is observable (product, web, CLI, API work), prefer EARS forms —
this is a preference for observable behavior, not a requirement for every
writing artifact:

```text
WHEN <trigger>, THE SYSTEM SHALL <observable behavior>.
WHEN <condition>, THE SYSTEM SHALL CONTINUE TO <preserved behavior>.     (regression-sensitive)
GIVEN <precondition> AND <precondition>, WHEN <trigger>, THE SYSTEM SHALL <response>.  (compound trigger)
```

State regression-sensitive behavior explicitly with `SHALL CONTINUE TO` —
brownfield changes preserve more than they add. Name relevant non-functional
constraints (performance, security, accessibility, compatibility) when they
apply; see `patterns.md` for the web/product forms.

## ④ Wireframe

Where ② experiments on the *world* (*is this true?*), this gate experiments on
your *answer* (*is this answer right?*) — the friction here is a concrete
instance hitting criteria and contract, and surviving it is what proves the
answer. When the instance falsifies a criterion, revise ③ using the purpose as
arbiter; when it exposes a wrong or missing world fact, return to ② and record
the experiment there.

Validate one concrete case before generalizing. The wireframe must pass ③
Criteria; refining it can also reveal that a criterion was wrong. When that
happens, use the purpose inside ③ as the arbiter, revise criteria or wireframe
explicitly, and keep the disagreement visible.

This is the answer probe of the experiment machine in
`references/experiment-log.md` — a cheap concrete instance, checked
independently (a real reader/operator walkthrough), establishes an indubitable
fact about the answer before the whole artifact is built: leaf before tree. Its
technique repertoire (baseline comparison, red-team cases, blind comparison,
LLM-as-judge) is how you make that check independent and cheap. Iterating an
instance toward a metric is tuning, not this probe — see the neighbor note in
that file.

Gate to continue:

- The text-first wireframe passed before any artifact-specific wireframe or
  generalized design, unless the task is explicitly collapsed.
- The actual operator(s)/reader(s) walked through it and confirmed fit.
- Information architecture and workflow survive the walk-through.
- Mock data is realistic enough to expose edge cases (empty, error, large,
  multi-language) — not just the happy path.
- For UI/web wireframes, the decisive states/conditions (empty, loading, error,
  populated, role/permission, locale, large data) are rendered as separate views,
  not only described — and each holds.
- Any visual treatment is validated as a concrete case; reusable component,
  token, responsive, and interaction rules are deferred to ⑤.
- Every placeholder is traced to a declared contract (declarative or ostensive)
  and checked against the real asset's constraints. An unaccounted placeholder
  means only an instance was validated, not the contract.
- For user-facing, ambiguous, or high-risk wireframes, a cold reader (blind
  reader) check passed: shown only the wireframe, mock data, labels, and
  visible sequence, the reader can infer the actor, purpose, expected outcome,
  next action, and important states. A wrong inference means the wireframe —
  not the reader — needs revision.

Form by artifact type: interactive → text-first screen sketch (prose or
ASCII-art layout), then a rendered HTML view; CLI/config → command transcript +
generated TOML/YAML + failure cases; API/data → request/response examples + error
cases + state table; text → outline with placeholder evidence; proposal →
one-page skeleton with stand-in numbers; research paper → section skeleton with
placeholder findings.

**For UI/web work, render the wireframe after it passes text-first.** Text first
— a prose or ASCII-art screen sketch, walked through with the operator/reader —
is the gate; it must pass before any pixels. Once it does, build a rendered HTML
view the user can open and judge by eye, using the brownfield edit recipe in
`references/brownfield-html-capture.md`: on a brownfield change, capture the real
rendered page as **locked context** and inject the change into its **variation
points** (mock data, declared axes/ranges); greenfield, assemble a self-contained
HTML mock of the sketched screen. This is still one concrete throwaway instance,
not the generalized ⑤ design.

**Render the decisive states, not just the happy path.** One screen is many
screens under different condition values — empty / loading / error / populated,
short vs overflowing data, first-run vs returning, role or permission variants,
locale and large-data edges. Render the states that could break the answer as
separate views — a small gallery in `02-Example/04-wireframe/` — so the user can
examine the variations side by side and confirm each holds. Name the axis and the
value behind each rendered state, and trace it to a ③ criterion or a declared
contract axis; a state you cannot render from the contract is a gap in the
contract, not just a missing screen.

**Open the renders for the user — do not just save them.** Once the views exist,
open them in a browser via Chrome DevTools (or a browser MCP), step through each
state yourself, and screenshot it, so the user reviews by looking — not by opening
files or clicking through the UI to reach a state. Pair each view with the one
thing to check there. Verification is a glance, not a chore.

Return to ④ when ⑧ drafting reveals the structure does not match the workflow;
⑤ design rules keep colliding with the layout or validated case; a ⑨ reviewer
says "I cannot find X"; ⑤ has to invent a schema/type/shape; or swapping a
placeholder's mock for the real asset breaks the instance.

## ⑤ Design — the generator

Build the generator, not another instance. Full mechanics and variation-point
coverage in `engine.md`. Design decisions: central thesis/message; section order
and each section's job; evidence placement; argument arc; definitions; rejected
alternatives; (for product artifacts) component boundaries, state model,
interaction rules, responsive rules, accessibility/focus rules, visual system
rules. Data/state contracts are *consumed* from the ④ contract, not decided here.

**Brownfield designs** open with an explicit before/after architecture sketch
when structure, workflow, ownership, or component boundaries change. Draw the
current architecture first, then the intended architecture: a **Static Model**
(Purpose, Components, Business Rules) and a **Dynamic Model** (workflow or
behavior) for both before and after. The sketch may be ASCII, Mermaid, boxes and
arrows, or a compact table, but it must show what is preserved, what is replaced
or extended, and where responsibility moves. Check brownfield assumptions —
which existing component is extended vs replaced, whether the stated current
structure is actually true — against local code, docs, or current rendered
behavior where the check is cheap; do not design against a remembered system.

Gate to continue:

- Each section has a role, not just a title.
- The reader can follow why the order works.
- Important rejected structures/component models are named when non-obvious.
- Non-obvious choices record RALPLAN-DR rationale: principles the choice must
  respect, decision drivers, ≥2 viable options (or invalidation rationale for
  rejected ones), and a steelman antithesis with its answer. Self-evident
  choices skip this — the rule is for choices a reviewer would otherwise have
  to reconstruct. See `references/decision-rationale.md`.
- The design matches audience, criteria, and the validated wireframe.
- It explains how the validated case generalizes to realistic data volume,
  breakpoints, states, and edge cases.
- For brownfield structural change, the before/after architecture sketch is
  present and cheap-checks the current model against the actual system before
  committing to the after model.
- Public terminology and model terms pass a project glossary / canonical
  terminology check: when a term conflicts with existing docs or the project
  glossary, the conflict is called out and the canonical term proposed before
  ⑦ tasking. Terminology drift is the cheapest defect to fix here and the most
  expensive to fix after it ships.

Useful structures:

- Decision memo: Context → Options → Criteria → Recommendation → Risks → Next steps.
- Explainer: Problem → Mental model → Examples → Edge cases → Summary.
- Research paper: Problem → Related work → Question → Method → Results → Discussion → Limitations.
- Essay: Tension → Claim → Evidence → Counterargument → Implication.
- Report: Executive summary → Findings → Evidence → Impact → Recommendations.
- Proposal: Problem → Opportunity → Approach → Plan → Cost/risk → Decision request.

## ⑥ Critic

Falsify the generator before tasking it. ⑤ generalized one validated instance
(④) into rules that produce *every* allowed instance — and that generalization
is an unverified inductive leap. ④ could falsify a criterion; this gate gives ⑤
the same scrutiny before ⑦ Tasks builds on it. **It always runs — the question
is depth, not whether.**

Depth scales with risk. A design touching any of these gets a deep pass
(external reviewer, multiple lenses, recorded rationale); a low-stakes design
gets a quick self-pass:

- security, privacy, safety, compliance, or permission boundaries
- migrations, destructive changes, data loss risk, or irreversible operations
- public terminology, interface, API, policy, workflow, or document structure
- cross-team, cross-module, or cross-artifact coupling
- large user-facing behavior, argument, narrative, or visual shifts
- one asserted option with weak alternatives or unclear decision drivers

Verdicts: `APPROVE` / `ITERATE` / `REJECT`. The reviewer may be the user,
another human, another agent, or a subagent.

Gate to continue:

- Verdict is `APPROVE`, **or** `ITERATE` revisions have been applied and the
  design re-reviewed.
- The critic explicitly checks whether the design is the best available answer
  to the necessity / why locked at ① Intent, not merely whether it satisfies the
  currently drafted ③ Criteria.
- `REJECT` returns to ⑤ Design (and possibly to ② Unknowns when an
  assumption was overturned).
- Even a quick self-pass leaves a one-line record of what was checked, so the
  pass is visible rather than silently skipped.

Full review criteria, depth triggers, and output shape:
`references/design-critic.md`.

## ⑦ Task Graph

Plan work units and dependencies. Not a table of contents — a section can depend
on research, data, diagrams, interviews, or a reviewer decision.

Gate to continue:

- Tasks are reviewable chunks.
- Dependencies are real, not just conversation order.
- Parallel work is identified.
- Each task has a check that proves it helped.

**Task/PR size is a reviewability gate.** Label each slice `small` (1–5
meaningful files, ≤ 200 meaningful changed lines), `medium` (6–10 files,
≤ 400 lines), or `large-justified` (larger, kept together for an explicit
coupling reason). These thresholds are tripwires, not schema rules — crossing
one means "justify or split", not "forbidden". See
`references/task-pr-size-guidance.md`.

When a task is too large for one line, split it into several reviewable tasks in
the same `03-Architect/07-tasks.md`. If the work truly needs an independent LEAF
cycle, create a separate sibling project folder and reference that path from the
task graph. Do not create nested `<##>-sub-*` folders.

Example:

```text
T1: Capture intent and open questions. Blocks T2.
T2: Gather references/benchmarks to sharpen criteria. Blocks T3.
T3: Define criteria and audience. Blocks T4.
T4: Build text-first wireframe with mock data; walk-through with operator/reader.
    Add an artifact-specific wireframe only if needed. Blocks T5.
T5: Draft design/outline with section or component jobs. Blocks T6/T7.
T6: Draft evidence-heavy body sections or prototype core. Blocks T8.
T7: Draft intro after outline is stable. Parallel with T6 if thesis is clear.
T8: Review claim/evidence or criteria/design alignment. Blocks polish.
T9: Final edit for tone, length, formatting, or interaction polish.
```

## ⑧ Artifact / Execution

Produce or revise the actual result. The artifact itself — text, code, video,
whatever — lives wherever you keep your work; `03-Architect/08-execution.md`
records *what was done*, one entry per work session, not the artifact.

Do not start this gate by default until the user explicitly approves the
Architect snapshot: ⑤ Design, ⑥ Critic verdict, ⑦ Task Graph,
execution scope, risks, and the first execution chunk. This is the exact line
between scaffolding and execution. Skip it only when the user explicitly
pre-authorized auto-execution for this sprout; if the work is too small or
low-risk to justify that approval surface, it should not have invoked
leaf-work.

Rules: start with the most load-bearing uncertain chunk, not necessarily the
first section; prefer ugly but checkable work over polished unsupported prose;
keep claims and evidence linked; mark placeholders honestly. Log each session in
`08-execution.md`: what you did, what came of it, what's next. Establishing a
fact about a chunk (does this work? does it pass?) is an answer probe — use the
experiment machine in `references/experiment-log.md`. Iterating a chunk toward a
metric (repeated keep/discard) is tuning — a distinct activity that also logs
here but is not that fact-gathering machine.

Gate to continue:

- The result can be reviewed against criteria.
- Open placeholders are visible.
- The next chunk's inputs are known.

Record the result as passed in `08-execution.md` only after the user explicitly
says so. When ⑧ is passed or delivered, move the project folder from
`.leaf/01-sprouts/<slug>/` to `.leaf/02-leaves/<slug>/`, update `00-status.md`
for Feedback / ⑨ Review, and run `leaf doctor` before starting ⑨.

Safety keywords: evidence, no collision, status, doctor.

## ⑨ Review / Sync

Keep the plan true after drafting and feedback. This gate runs in
`.leaf/02-leaves/<slug>/` after ⑧ has passed and `leaf-work` has completed the
sprout-to-leaf transition. It may loop with ⑧ when review finds a defect, but a
return to execution must preserve the leaf folder as the current body unless
`leaf-done` closes it.

Review checks: Does the artifact satisfy criteria? Are claims supported by
evidence? Does section order still fit the argument? Did drafting reveal missing
research or a weaker thesis? Are non-goals respected? Is detail right for the
audience? Does the wireframe still fit?

Sync rule: if review changes the claim/effect, update criteria; if it changes
the argument flow, update the design/outline; if it changes workload, update the
task graph. Do not keep stale planning notes beside a changed draft.

## ⑩ Retrospect

Close the leaf on two axes — what this work established (content) and how the
work went (process). ⑩ runs after ⑨ Review / Sync. Both halves look back at the
finished whole; they differ only in subject.

**Limitations — the content retrospective.** Record what was established and
where its authority ends: the claims or results that now hold (and against
which criteria they passed), what remains unresolved, and the boundaries —
conditions under which the conclusions do not apply. This is what makes a
finished leaf citable as prior work: future work builds on the established
part without re-deriving it, and the unresolved and boundary entries are
where a future leaf's ① necessity is born — the same way a paper's
limitations section can start the next study. Write it to
`10-retrospective/limitations.md` (or a Limitations section of the single
retrospect file).

**Lessons — the process retrospective.** Improve the next loop. Capture: What
sequence worked? What did we draft too early? Which criterion was missing?
Which review check caught the most? What template or skill should change?
**Which unknowns surfaced mid-work that should have been caught at ②?** Open
the discoveries note and, for each unplanned detour, ask which category was
missed — a domain concept, a convention, a selection criterion, or an
external/internal source — and feed it back into the next project's ② checklist.

Gate to future work:

- What was established, what remains unresolved, and where the boundaries lie
  are stated plainly enough for future work to cite without rereading this
  leaf.
- Lessons are phrased as reusable process changes, not vague feelings.
- The ② checklist for similar future work is updated with the categories or
  example questions the team kept missing.

### Profile update

After the retrospect is written, invoke `leaf-profile` if this episode revealed
a user language preference, recurring requirement, agent mistake, wrong-answer
note, reusable correction, or cross-leaf fact that should affect future leaf
work.

The test question is: "Would the next leaf's agent behave differently, or avoid
repeating this mistake, for knowing this?" If yes, update `.leaf/PROFILE.md` and
show the diff. If no, keep the note in the retrospect, review, or pressed file.

PROFILE entries may specialize `leaf-soul` conduct but never negate it. When an
episode repeatedly demands a negation, surface it as a `leaf-soul` change signal
instead of writing the entry.

**Close-out — ⑩ 다음은 바로 `leaf-done`.** ⑩ passing does not end the work.
Immediately after the retrospect passes, invoke `leaf-done`. Do not stop at the
retrospect file, and do not report leaf-work complete before that decision is
handled.

---

## Anti-Patterns

- Searching, benchmarking, or researching before listing what is unknown.
- Treating mid-work research detours as expected, rather than as a signal that
  ② was incomplete.
- Drafting polished prose before criteria are clear.
- Forcing criteria before the user has enough references to know what they want.
- Building the wireframe before criteria are clear.
- Starting with title/outline when the reader decision is unknown.
- Treating table of contents as a task graph.
- Letting AI invent evidence or audience.
- Keeping an old outline after review changes the thesis.
- Doing "final polish" while claims are still unsupported.
- Asking for broad review when the artifact needs a specific gate check.
- ⑤ inventing a schema/shape the ④ contract should have locked.
- Skipping the retrospective's review of mid-process discoveries.
- Starting ⑨ Review while the passed work is still under `.leaf/01-sprouts/`.
- Stopping after the retrospect file is written, leaving the leaf without a
  keep/press/fall decision and the `leaf-done` handoff.
