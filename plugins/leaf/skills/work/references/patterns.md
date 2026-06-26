# Application Patterns

Per-domain templates. Every domain runs the same gate skeleton — only the
*content* of each gate differs. So this file gives the skeleton once, then a
difference table, then the two domains that diverge enough to need their own note.

## The shared skeleton

```text
① Intent:             the topic, tension, question, or ask.
② Unknowns & Context: list missing domain concepts, standards, external
                      comparators, local grammar, and internal facts BEFORE researching;
                      update each entry with sourced answers, assumptions,
                      materials, unresolved questions, and candidate frames.
③ Criteria:           intended effect + audience, scope, evidence, tone,
                      length, deadline, non-goals, and tradeoff principles.
   Output form:       decided at intent or absorbed into criteria.
④ Wireframe:          skeleton in the chosen form with placeholder/stand-in
                      content; the actual reader/operator walks through and
                      confirms fit.
⑤ Design:             generalize the validated case into structure / section
                      roles / component rules, including Precedent Fit when the
                      result enters an existing corpus or system. Give each main
                      artifact a one-sentence responsibility statement
                      (consumes the ④ contract — see engine.md).
⑦ Task graph:         real dependencies, not the table of contents.
```

## Difference table

| Domain | ③ Criteria | ④ wireframe shape | ⑤ design structure | ⑦ task order |
|---|---|---|---|---|
| **General document** | reader can decide / understand / do Y; audience, scope, evidence, tone, deadline, non-goals | section skeleton with placeholder evidence | information architecture + section roles | research → outline → draft → review → polish |
| **Essay / article** | thesis, intended reader shift, audience, examples, counterargument, length, voice | outline: thesis · evidence list · counter-arg · implication | hook → claim → evidence arc → counterpoint → implication | thesis → evidence list → outline → body → intro/end → edit |
| **Research paper** | research question, contribution, method, evidence, validity | section skeleton (problem/related/method/results/discussion/limitations) with placeholder findings | problem → related work → method → results → discussion → limitations | literature → RQ → method/data → results → discussion → intro |
| **Strategy memo** | decision-maker can choose a path; options, criteria, recommendation, risks, decision needed | memo skeleton (context/options/evaluation/recommendation/risks/next) with stand-in numbers | context → options → evaluation → recommendation → risks → next action | criteria → option evidence → recommendation → memo draft |
| **Proposal** | approver can say yes/no; problem, approach, cost, timeline, risk, success metric | one-page skeleton (why now / what changes / plan / proof / ask) with stand-in numbers | why now → what changes → plan → proof → ask | clarify ask → define plan → estimate cost/risk → draft |

Notes that don't fit a table cell:

- **Research paper:** write the intro late (results/contribution may shift);
  write methods and results earlier when evidence exists.
- **② phrasing per domain** — essay: "what frames, counter-arguments, or
  canonical examples am I guessing?"; research: "unknown literature, methods,
  datasets, theoretical frames"; strategy: "missing market facts, criteria,
  stakeholder views"; proposal: "approver criteria, prior comparable asks,
  internal resource/constraint facts." Answer each entry in place as references
  or user-provided facts arrive.

## Web / product development (diverges — own note)

This domain is where the ④ instance/contract and ⑤ generator distinction matters
most, so it does not collapse into the table.

```text
① Intent:             "Improve the dashboard" / "make a landing page."
② Unknowns & Context: missing user-job facts, existing design-system rules,
                      accessibility standards, current behavior, comparable
                      product patterns, user workflows, benchmark notes, and
                      copy/adapt/avoid decisions.
③ Criteria:           target user completes/understands the job, e.g. "operators
                      identify critical items within 10 seconds"; behavior,
                      accessibility, responsive constraints, performance budget,
                      state handling, non-goals, product principles.
   Output form:       visual mockup, interactive prototype, production PR,
                      design doc, or demo.
④ Wireframe:          ASCII screen sketch with mock data FIRST — layout,
                      sections, drill-down paths, empty/loading/error states.
                      Then HTML/Figma if the browser/canvas/visual treatment is
                      needed to validate the concrete case. Operator(s) walk
                      through. Lock the contract.
⑤ Design:             generalize the validated case into component boundaries,
                      state model, data flow, interaction rules, responsive
                      rules, visual system rules, and empty/loading/error
                      ownership — covering each variation point's full range
                      (see engine.md).
⑦ Task graph:         model → shell → components → interactions → states → review.
```

Web/product criteria forms:

```text
WHEN an operator opens the dashboard, THE SYSTEM SHALL show critical items without requiring navigation.
THE PROTOTYPE SHALL distinguish status by text/icon as well as color.
THE WORK SHALL prioritize scanability over decorative density.
```

Three augmentations matter most for development work:

- **Regression-sensitive behavior is explicit.** Brownfield changes preserve
  more than they add; state what must keep working, or the agent will only
  test the new path:

  ```text
  WHEN <condition>, THE SYSTEM SHALL CONTINUE TO <preserved behavior>.
  ```

- **Compound triggers** use GIVEN/AND when a behavior depends on preconditions,
  not just the triggering event:

  ```text
  GIVEN <precondition> AND <precondition>, WHEN <trigger>, THE SYSTEM SHALL <response>.
  ```

- **Add a non-functional section** for performance, security, accessibility,
  compatibility, privacy, migration, and similar constraints when they apply —
  they rarely fit a WHEN/SHALL line but still reject bad designs at ⑤.

**Brownfield note:** when the work changes an existing system, identify the
behavior to preserve (write it as `SHALL CONTINUE TO` criteria) and the
existing component responsibility the change extends or replaces *before*
proposing a new structure. For existing web pages, the real rendered page can
become the ④ artifact-specific wireframe — see
`references/brownfield-html-capture.md`.

## CLI / config / API (diverges — own note)

④ wireframe is a command transcript or generated config (TOML/YAML) with
expected output and failure cases, or representative request/response examples
plus an error/state table. The contract is usually declarative (schema, types,
enums, exit codes) — lock it explicitly in ④ so ⑤ never invents it.
