# LEAF Loop Contract

Use this reference when a LEAF gate needs repeated passes. A LEAF project is a
lifecycle; a loop is a bounded pass inside a gate. Do not rename the whole LEAF
lifecycle as a loop, and do not treat a loop as permission for endless autonomy.

## Admission test

Before calling repeated work a loop, check whether fresh feedback can change the
next action.

Use a loop only when all of these are true:

- a fresh observation can change what the next pass chooses;
- each pass can choose one bounded, reversible or reviewable action;
- the result can be checked under stable conditions;
- evidence and remaining work have a clear record location;
- the pass can stop in a named terminal state.

If any item is false, use a one-shot workflow instead. Repeating a checklist
without new feedback is not a loop.

## Pass contract

Every admitted gate loop follows this shape:

1. **Observe**: reread fresh state and collect the agreed evidence.
2. **Choose**: select one highest-value in-scope action from explicit criteria.
3. **Act**: make one bounded change, run one probe, or produce one candidate.
4. **Verify**: run the same acceptance check under recorded conditions.
5. **Record**: save the action, evidence, outcome, remaining work, and next input.
6. **Stop or repeat**: repeat only while progress is measurable and the current
   authorization still applies; otherwise stop in a named terminal state.

Re-read current state before consequential actions. Do not carry stale state from
an earlier pass into a new decision.

## Terminal states

Use named stops instead of reporting every stop as success.

| State | Meaning |
|---|---|
| `success` | The stated acceptance check passed. |
| `clean no-op` | Fresh state shows no change is needed. |
| `blocked` | The next useful action is impossible with the current inputs. |
| `approval-required` | The next action crosses the current authorization boundary. |
| `exhausted` | The user-supplied limit is spent. |
| `no-progress` | Another pass is unlikely to improve the result measurably. |
| `stagnated` | Repeated attempts are not changing the evidence. |

Never present a failed check, exhausted budget, or blocker as success.

## Gate mappings

| Gate loop | Observe | Choose | Act | Verify | Record | Stop examples |
|---|---|---|---|---|---|---|
| ② Unknowns Loop | Raw request, current unknowns, prior facts, references | The guess that would most damage later work if wrong | One cheap independent world probe | Source, command output, benchmark, user answer, or independent check | `01-Learn/02-unknowns.md` and, when needed, `01-Learn/02-experiments/<name>.md` | fact established, explicit assumption, user review needed, blocked |
| ④ Wireframe Loop | Locked why/what/wireframe, ③ Criteria, one candidate instance | The decisive concrete case or placeholder contract to test | Text-first wireframe revision or one artifact-specific instance | Reader/operator walkthrough, cold-reader check, criteria check | `02-Example/04-wireframe.md` or `02-Example/04-wireframe/` | pass, return to ③, return to ②, user review needed |
| ⑧ Execution Loop | Task graph, design, current artifact, diff, tests, `leaf doctor` | The most load-bearing uncertain chunk | One implementation or drafting chunk | Tests, criteria review, artifact inspection, `leaf doctor` | `03-Architect/08-execution.md` | chunk passes, blocked, approval-required, no-progress, return to earlier gate |
| ⑨ Review Loop | Finished artifact, criteria, claims, feedback | One actionable review issue | Fix, defer, contest, or return to ⑧ | Reviewer decision, tests, criteria/design sync | `04-Feedback/09-review.md` and any synced gate file | review pass, returned to ⑧, deferred with reason, invalid finding |

## Experiment vs tuning

Do not call every run-and-measure cycle an experiment.

- **Experiment** establishes a fact. In LEAF this is the ② world probe ("is this
  true?") or the ④ answer probe ("does this concrete answer hold?"). Keep the
  fact/guess boundary visible and record the established fact back in the owning
  gate.
- **Tuning** improves an answer. It may repeat a change-and-measure pass in ⑧,
  but it aims to improve a metric or artifact, not establish a fact. Log tuning
  in `03-Architect/08-execution.md`.

When optimizing a prompt, ranking, model, or other artifact that can overfit its
own metric, separate the working signal from a fresh acceptance check.

## Approval boundary

Designing or using a loop does not authorize schedules, production changes,
destructive actions, external messages, spending, private-data exposure, or
credential use. Stop with `approval-required` unless the current user request
already authorized that exact action.

Autopilot may draft, review, and execute inside the human-reviewed
why/what/wireframe lane, but it still stops at the same approval boundary.

## Record shape

When a gate loop runs, leave enough state for the next pass or another agent:

```md
## Loop pass

- Observe:
- Choose:
- Act:
- Verify:
- Record:
- Stop state:
- Next:
```

Keep the record short. It should explain the next decision, not preserve every
scratch thought.
