# Gate Authoring

Use this when writing or revising a durable gate artifact. Gate files are not
one-shot generation; they become authoritative only after the user confirms
that this version is fit for the next gate.

## Cycle

1. **Draft** the smallest artifact that can be challenged in one pass.
2. **Grill** it against the current gate's foci in `gates.md`; ask one focused
   question when the answer cannot be inferred or checked cheaply.
3. **Revise** inline. Keep fact, assumption, decision, and open risk visibly
   separate.
4. **Display for review** with the lightest surface that lets the user verify
   the substance.
5. **Confirm** before downstream gates consume it. Until then, the draft is
   working material, not an approved gate.

## Grill Foci

- ① Intent: raw wording preserved, why followed to its real endpoint, core noun
  stable, topology named or explicitly deferred.
- ② Unknowns: weakest clarity-ledger row used only to aim learning, unknowns
  grouped, blocking items resolved or carried as explicit assumptions, fact vs
  assumption boundary visible.
- ③ Criteria: purpose states the desired effect, every ledger row is locked or
  carried as explicit risk, success checks can reject a bad instance.
- ④ Wireframe: text-first instance uses realistic data, every placeholder has a
  contract and variation point, the intended user can walk through it. For
  user-facing, ambiguous, or high-risk wireframes, record a cold/blind reader
  check: from the wireframe, mock data, labels, and visible sequence alone,
  the reader can infer the actor, purpose, expected outcome, next action, and
  important states.
- ⑤ Design: consumes the approved ④ contract, generalizes across variation
  points, names alternatives and rationale for non-obvious choices.
- ⑥ Critic: verdict is `APPROVE`, `ITERATE`, or `REJECT`, and revisions route
  back to the smallest affected gate.
- ⑦ Tasks: slices are reviewable, dependency claims are real, each task has a
  check that proves it helped.
- ⑧ Execution: result or handoff can be reviewed against criteria; placeholders
  and next inputs are visible.
- ⑨ Review/sync: criteria, design, and task graph stay true after feedback.
- ⑩ Retrospect: established claims, unresolved limits, and reusable process
  lessons are separated.

## Review Surface

Default to summarize instead of dumping files: path, section, and one-line
change. Quote only the few lines the user must approve. For longer artifacts,
prefer an editor, rendered pane, or existing collapsed tool output over
reprinting the file in chat.
