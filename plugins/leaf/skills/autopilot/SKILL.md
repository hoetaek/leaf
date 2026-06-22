---
name: autopilot
description: Use after a LEAF sprout's why / what / wireframe triple has been explicitly reviewed by the user, when the user wants the remaining LEAF gates to proceed automatically through Example, Architect, Execution, Review, and Retrospect. Trigger on "$leaf:autopilot", "LEAF autopilot", "triple is approved, continue automatically", "run the rest of LEAF without asking", or "after this lock, handle the rest". Do not use before the triple is reviewed, for unclear intent, for destructive/external/credential/cost/security/privacy-sensitive work without explicit pre-authorization, or when the user wants normal per-gate approval.
---

# LEAF Autopilot

`autopilot` carries a LEAF sprout after the human-reviewed `why / what /
wireframe` triple. It does not remove LEAF's judgment boundary; it moves the
boundary to the triple lock, then runs the remaining gates with automatic
reviews, hard stops, and evidence.

## Core Contract

- **Triple first.** Do not start unless `00-status.md` has human-reviewed `why`,
  `what`, and `wireframe` values. If they are missing, provisional, stale, or
  marked `USER REVIEW NEEDED` / `LOCK CANDIDATE`, return to `learn`.
- **Autopilot after the lock.** Once the triple is locked, proceed through
  `work` gates automatically: ③ Criteria, ④ Wireframe, ⑤ Design, ⑥ Critic,
  ⑦ Tasks, ⑧ Artifact / Execution, ⑨ Review / Sync, and ⑩ Retrospect.
- **Review still happens.** Gate reviews are automatic unless a hard stop or
  pre-authorization gap appears. Leave the review evidence in the gate file,
  `08-execution.md`, or `09-review.md`.
- **Do not duplicate LEAF.** Invoke and follow `leaf:work`, `leaf:polish`,
  `leaf:press`, `leaf:profile`, `leaf:soul`, and `leaf:using-leaf` when their
  contracts apply. This skill orchestrates them; it does not rewrite their gate
  rules.
- **Status is the dashboard.** Keep `00-status.md` current after every gate,
  return, hard stop, execution session, and sprout-to-leaf move.

## Start Checklist

Before doing work:

1. Read `../soul/SKILL.md` and apply its conduct rules.
2. Read `../using-leaf/SKILL.md` for close-out rules and routing boundaries.
3. Run `git status --short --branch` and `leaf doctor`.
4. Read the sprout's `00-status.md`, `01-Learn/01-intent.md`, and
   `01-Learn/02-unknowns.md`.
5. Verify the triple is human-reviewed and internally consistent.
6. Read `references/approval-policy.md` when the request involves execution,
   external side effects, credentials, cost, security, privacy, or ambiguity
   about what autopilot may decide.

If any start check fails, stop with the smallest needed repair or user question.

## Workflow

1. **Consume Learn.** Treat the locked triple as the current contract. If ③ or
   later reveals the triple is wrong, return to `learn`, record the return, and
   do not continue on the old contract.
2. **Run Example.** Use `leaf:work` gates ③ and ④. Write criteria that can reject
   a bad instance, then build the cheap wireframe promised by `wireframe`.
3. **Run Architect.** Use gates ⑤, ⑥, and ⑦. Design the generator, run at least a
   quick critic pass, and slice reviewable tasks.
4. **Execute only inside the delegated lane.** Gate ⑧ may start without asking
   only when the task graph, risks, first execution chunk, and hard-stop policy
   fit the locked triple and the user's pre-authorization.
5. **Audit completion.** Do not mark done because files exist. Map the locked
   triple and criteria to evidence: gate files, command output, tests, `leaf
   doctor`, review notes, and unresolved assumptions.
6. **Move to Feedback.** After ⑧ is passed or delivered, move the sprout to
   `.leaf/02-leaves/<slug>/` using the LEAF lifecycle rule, update status, and
   run `leaf doctor`.
7. **Review and retrospect.** Run ⑨ and ⑩ automatically unless a hard stop
   appears. Then follow `using-leaf` ending rules: keep, press via `leaf:press`,
   or fall.

## Hard Stops

Stop and ask for explicit user direction when any of these appear without prior
authorization:

- the triple is missing, stale, contradicted, or not human-reviewed;
- destructive or hard-to-revert changes;
- credentials, secrets, external accounts, production systems, deployment, or
  cost-incurring actions;
- public or external sharing;
- security, privacy, legal, policy, or permission-boundary decisions;
- scope expansion, split decisions, or a changed core noun;
- user taste, risk tolerance, or organizational judgment that cannot be inferred
  from the locked triple;
- the same failure repeats three times;
- `leaf doctor`, tests, review, or completion audit fails.

## Completion Audit

Before reporting completion, show:

- the locked triple consumed;
- the current LEAF path and stage;
- gates completed and evidence paths;
- commands run, including `leaf doctor` and any tests/build/lint;
- automatic review or critic verdicts;
- hard stops checked and not triggered, or the stop that remains;
- files changed outside `.leaf/`, if any.

Do not hide unresolved assumptions. If something is delegated rather than
human-approved, say where the delegation was recorded.

## Anti-Patterns

- Starting before the triple is locked.
- Treating "no human approval" as "no review".
- Consuming a stale gate because the next file already exists.
- Rewriting LEAF gate contracts inside this skill.
- Starting ⑧ execution when criteria, design, task graph, risks, or first chunk
  are missing.
- Calling the leaf complete without a completion audit.
