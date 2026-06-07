# Clarity Ledger

The ledger names the five dimensions of intent that downstream gates depend on,
so you can aim at the **weakest row** instead of the next topic that happened to
come up — conversation order is rarely the right order. Its home is ① and ③: ①
locks the Intent row, and ③ Criteria scores the full set before locking
criteria. During ② it is only a *lens* — glance at the weakest row to aim
learning, never to force a decision there.

A row is stable not when the agent knows it, but when the *user* has learned it
well enough to judge it — to choose, in their own words, and defend the choice;
"defer this for now" is itself a valid, defensible choice, not a gap. The weakest
row is whatever the user still cannot judge. The ledger is therefore asking, row
by row, where learning has not yet reached judgment.

## The five rows

| Row | The question it answers | What "stable" looks like |
|---|---|---|
| **Intent** | Why is this needed, what change does the user actually want, and what is the core noun? | The necessity (problem, obligation, curiosity, or deferred felt sense) and the desired effect are each one sentence, and the core noun (essay / report / spec / proposal / video / config / dashboard …) does not drift across answers. |
| **Topology** | How many independent outcomes / components / surfaces / deliverables are in scope? | Each is named, or explicitly deferred. The count is not silently mutating between "one thing" and "a few related things." |
| **Success** | How will completion be observed? | At least one acceptance check that a reviewer (or the user) could apply to a candidate result and disagree about, not "looks good." |
| **Constraints** | What is non-negotiable — non-goals, boundaries, preserved behavior? | The things that must not change, the audience that must not be alienated, the prior decisions that must not be overturned are named. |
| **Output form** | What artifact, lifecycle, and handoff shape? | The format, where it lives, who consumes it, and what counts as delivery are explicit — not assumed from the current conversation medium. |

## The rule

**Aim the next question at the weakest row.**

If Intent is weak, do not start sourcing references (② External facts) — the
question is what the user wants, not what others have done. If Output form is
weak, do not start drafting criteria — the criteria depend on whether the
deliverable is a document, a deck, or a code change.

Read the rows in order each time you must choose what to ask. Weak rows
upstream silently corrupt downstream gates: a weak Topology row makes ③
Criteria fragment-by-fragment; a weak Output form row makes ⑤ Design generate
for the wrong medium.

## Core noun stability

**If the core noun drifts across answers, stop.** The user says "an idea,"
then "an essay," then "a report," then "a task list" — each shifts what
downstream gates must produce. Ask explicitly: *which of these is the actual
work item?* until one noun stabilizes.

A drifting noun is not refinement. It is a sign that **the user is also
unsure**, and any work done on the wrong noun has to be re-done. The cheapest
move is one direct question, not three more clarifying probes that assume the
latest noun.

## Topology confirmation

Before ② research starts, confirm topology when the request can produce more
than one independent outcome. Name the top-level outcomes, surfaces,
integrations, or deliverables that could succeed or fail separately, then ask
whether any should be added, removed, merged, split, or explicitly deferred.
The most-described component must not silently stand in for quieter siblings.

## Examples

### Intent weak

```
User: I want to write something about how AI changes our team's workflow.

Weakest row: Intent. "Something about" + "changes" is too loose.
Next question: What effect do you want the reader to have after reading? A
decision? A vocabulary? A shared concern?
```

### Topology weak

```
User: I need a report for leadership about support ticket trends and the new
feature impact.

Weakest row: Topology. Two outcomes ("ticket trends", "feature impact") may
be one report, two reports, or a report + a one-pager.
Next question: Is this one document, or two? If one, which finding leads?
```

### Success weak

```
User: I want a clear explainer of our new permissions model.

Weakest row: Success. "Clear" has no test.
Next question: What should the reader be able to do after reading — write a
new permission rule? Audit an existing one? Explain it to legal?
```

### Constraints weak

```
User: Rewrite our onboarding doc to be more welcoming.

Weakest row: Constraints. "More welcoming" can drift into changing tone,
omitting hard steps, or rebranding.
Next question: What must not change — the required steps, the legal disclaimer,
the current vocabulary your support team uses?
```

### Output form weak

```
User: I need to communicate our Q3 architecture decision to the company.

Weakest row: Output form. Doc? Email? All-hands slide? Recorded video?
Architecture decision record in the repo?
Next question: What form do they consume — a written doc they can reference,
a live talk, or both?
```

### Core noun drift

```
User: I have an idea about our auth migration.
User: So the essay should explain why we picked OIDC.
User: Maybe a memo to the platform team is better.
User: Or just a task list of what's left.

Halt. Ask: which one is the actual work item — essay, memo, or task list?
They are three different deliverables with three different audiences.
```

## How it connects to other gates

- ① Intent locks the necessity (why), the desired effect derived from it, and
  the core noun (the Intent row). When multiple outcomes are plausible, it also
  records the first topology confirmation or the explicit reason to defer it.
- ② Unknowns uses the ledger only as a lens: glance at the weakest row to aim
  learning (which domain/standards/external/internal study would close it),
  never to force the row closed.
- ③ Criteria is where the ledger is scored and locked — Intent becomes Purpose;
  Topology, Success, Constraints, and Output form become Requirements. A row that
  cannot be locked is carried as an explicit risk or assumption.
- ④ Wireframe locks the contract for the chosen Topology and Output form.

The ledger is not a separate artifact to maintain — during ② it is a lens for
choosing the next question, and at ③ it is the dimension checklist criteria are
scored against. Once a row is stable, it stops appearing in questions.
