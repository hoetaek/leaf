# Decision Rationale (RALPLAN-DR)

Use during ⑤ Design when a choice is **non-obvious** — when a future reviewer
(or a future you) would otherwise have to reconstruct why this option was
picked over the alternatives. The shape below makes that reasoning durable so
it can be inspected, challenged, and re-used without re-deriving it.

The form is small and disciplined on purpose. Self-evident choices skip it.
Forcing it on every micro-decision turns ⑤ into noise.

## When to apply

Use RALPLAN-DR for choices that:

- a reviewer in ⑥ Critic would have to assess
- shape several downstream tasks or interfaces
- pick one option from a real field of alternatives, not one obvious answer
- preserve a constraint the team has previously argued about
- introduce a name, structure, placement, taxonomy, format, workflow, policy
  language, component boundary, file location, or interface shape that future
  work may copy as precedent
- define, split, merge, or move an artifact responsibility boundary
- are easy to forget the *why* of within a few weeks

## When to skip

Skip when:

- the choice follows directly from ③ Criteria or the ④ contract
- there is one option and ruling out alternatives would be artificial
- the choice is a local naming or formatting decision with no downstream
  coupling and no precedent risk
- the artifact responsibility statement is obvious, short, and not contested
- the rationale is already captured in ③ Criteria itself

## The four fields

### Principles (3–5)

Constants the decision must respect. Not goals, not preferences — rules the
choice cannot violate even if it would be more convenient.

Examples: "no breaking changes to public API in this release"; "no new
production data store this quarter"; "every user-facing string must be
translatable"; "follow the existing terminology unless it blocks the work";
"stay below 200ms p50 for this route."

### Decision drivers

The top forces that actually selected one option over the others. Not every
factor — the ones with real weight. If you remove a driver and the choice
flips, it was a driver. If you remove it and the choice stays, it was
background.

Examples: "operability for our 2-person on-call rotation"; "fewest schema
migrations on the live table"; "reversibility within one quarter if user
research disagrees."

### Viable options

At least two **real** options with bounded pros/cons. Strawmen invalidate the
rationale — if the other options are obviously worse, the field is not real
and you should either find better alternatives or admit there is only one and
state why the others were ruled out (invalidation rationale).

Each viable option states: what it is, how it scores against the principles
and drivers, and what its strongest pro and worst con are.

If only one option survives, document the invalidation rationale for the
rejected ones in their own bullet — not "we didn't consider X" but "X was
considered and ruled out because Y."

### Steelman antithesis

The strongest argument against the chosen option, written as if a credible
opponent who has read the principles and drivers is making it. Then your
answer.

The answer is not a dismissal. It either concedes a real residual risk (and
names how it will be mitigated or accepted), or it identifies a flaw in the
antithesis (an assumption the opponent makes that does not hold here).

A steelman that is easy to answer is usually a strawman in disguise. If you
cannot find a serious counter, the choice may not be as decided as it feels —
sit with the field longer.

## Output shape

```text
## Decision: <one-line summary of the choice>

Principles
- ...
- ...

Decision drivers
- ...
- ...

Viable options
- Option A (chosen): <description>
  - Pro: <strongest>
  - Con: <worst>
  - Fit vs principles/drivers: <how it scores>
- Option B: <description>
  - Pro: <strongest>
  - Con: <worst>
  - Why not chosen: <which driver flipped it>
- Option C (if real): ...
- Rejected without full evaluation: <name> — <invalidation rationale>

Steelman antithesis
- Argument: <strongest credible counter>
- Answer: <concession + mitigation, or identified flaw in the assumption>
```

## How it connects to other gates

- ⑤ Design produces RALPLAN-DR for each non-obvious choice.
- ⑥ Critic directly reads RALPLAN-DR: the **Principles and
  drivers**, **Fair alternatives**, and **Steelman antithesis** review checks
  in `design-critic.md` correspond one-to-one to the fields above.
- ⑩ Retrospect can reuse the rationale to recognize when a previous decision
  is now invalid because a driver or principle changed.

The rationale is durable on purpose — it is meant to be re-read months later,
not consumed once and discarded.
