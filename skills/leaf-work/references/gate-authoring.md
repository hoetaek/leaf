# Gate Authoring

Use this when writing or revising a durable gate artifact. Gate files are not
one-shot generation; they become authoritative after they pass the gate foci
and the current approval policy. Default approval is by phase; individual gates
escalate to explicit user approval only when they lock high-impact decisions,
change previously approved direction, start ⑧ execution, or the user asks to
review them.

## Cycle

1. **Draft** the smallest artifact that can be challenged in one pass.
2. **Grill** it against the current gate's foci in `gates.md`; ask one focused
   question when the answer cannot be inferred or checked cheaply.
3. **Revise** inline. Keep fact, assumption, decision, open risk, and any
   user-review placeholder visibly separate.
4. **Display or open for review** with the lightest surface that lets the
   reviewer verify the substance when this is a phase boundary, an escalated
   gate, or a user-requested checkpoint.
5. **Confirm or record delegation.** If explicit approval is required, wait for
   it before downstream gates consume the artifact. Otherwise record the
   self-review result and continue inside the current phase.

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
- ⑤ Design: consumes the validated ④ contract, generalizes across variation
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
change. Quote only the few lines the user must approve or inspect. For longer
artifacts, prefer an editor, rendered pane, or existing collapsed tool output
over reprinting the file in chat.

Mark assumptions, placeholders, or reviewer questions with `USER REVIEW NEEDED:`
or `ASSUMPTION:`. Open the file in the user's preferred editor when known
(`cmux markdown`, `$VISUAL` / `$EDITOR`, `code`, `vim` / `nvim`, Obsidian,
Notepad, etc.); otherwise ask once or give the path and sections.

## Escalation Triggers

Ask for explicit user approval before consuming a gate artifact when any of
these are true:

- it changes ① Intent or the top-level work item;
- it locks or changes decisive ③ Criteria;
- ④ Wireframe needs the actual operator, reader, or stakeholder to confirm fit;
- ⑤ Design commits public, costly, security/privacy-sensitive, or hard-to-revert
  structure;
- ⑦ Task Graph is being promoted into ⑧ Artifact / Execution; approval covers
  ⑤ Design, ⑥ Critic verdict, ⑦ Task Graph, execution scope, risks, and the
  first execution chunk;
- ⑧ Artifact is being marked passed, delivered, or externally shared;
- a return invalidates a previously approved phase boundary;
- the user explicitly asked to review the gate.
