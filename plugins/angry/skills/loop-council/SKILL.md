---
name: loop-council
description: "Bounded review/fix loop using council as the engine — convene the panel, fix the chair's top verdict, verify, repeat within budget (default 2). Stops on cross-axis tradeoffs needing a human. Triggers: council loop, loop-council, panel review loop."
---

# Loop Angry Council

Run a bounded panel-review/fix loop. This skill owns the loop mechanics only:
convene the panel, take the chair's single highest-priority verdict, fix it,
verify it, and repeat until no material issue remains within the agreed budget.

Use `council` as the review engine. Before the first pass, read
`../council/SKILL.md` if available and follow its panel triage, independent
parallel review, and chair synthesis rules. If that file is unavailable, say so
and continue with the loop rules here without pretending to run the full council.

## Cost note

Each iteration runs a whole council pass — triage, then 3-5 persona subagents in
parallel. That is several times the cost of a single-persona loop. Keep the
budget tight, and re-triage every pass: the relevant panel changes as the scope
changes after a fix.

## Scope

Loop only on the chair's highest-priority verdict when it is a **material,
fixable, in-axis** finding — high or critical severity, concrete evidence, and a
root-cause fix that does not need a human decision.

Do not loop on:

- a chair verdict that is a cross-axis **tradeoff** (e.g. `jobs` "cut it"
  vs `ramsay` "not done yet") rather than a true contradiction — that needs
  a human call. Stop and surface it, do not pick a side and "fix" it.
- taste, formatting, low-severity findings, or a persona's "this is actually
  fine" note.
- findings outside the summoned axes — report them, do not fix them.

## Budget

If the user does not provide a budget, default to **2 iterations max** — lower
than a single-persona loop because each pass spawns a full panel. Stop after the
budget and report the remaining panel findings. Continue only when the user
explicitly asks for more.

## Loop

1. Fix the scope: what is under review + Context Pack (Known Facts, Fixed
   Constraints), per the `council` protocol.
2. Run one `council` pass: triage the 3-5 relevant personas, have them
   review independently and in parallel, then synthesize the chair's ranked
   verdict.
3. Take the chair's `고치는 1순위`. Continue only if it is in Scope (material,
   fixable, in-axis, not a tradeoff). Otherwise stop and report it.
4. Prove the selected issue with evidence (file/line or quoted artifact) and state
   whether it is verified fact or inference.
5. Apply the smallest root-cause fix for that one issue only. Do not
   opportunistically fix lower-ranked findings in the same pass.
6. Run the cheapest verification that would fail if the fix regressed — tests,
   build, lint, or the persona's own falsifiable test when one was given.
7. Re-run the council against the new state, re-triaging because the relevant
   panel may have changed. Continue only if the new #1 is in Scope and the
   iteration budget remains.

## Termination

Stop the loop when any of these is true:

- no material in-scope issue remains in the chair's verdict
- the next #1 is a cross-axis tradeoff needing a human decision, taste, or
  low-value cleanup
- the iteration budget is exhausted
- the fix is destructive, broadly changes public behavior, alters
  deploy/release state, or needs external secrets or approval
- the same unresolved blocker repeats and cannot be progressed locally

When stopping, report remaining in-scope findings and unresolved tradeoffs
separately as not fixed.

## Output Per Iteration

Keep each iteration short:

- `회차:` loop number
- `소집 패널:` the personas summoned this pass (+ why); note any re-triage change
- `고치는 1순위:` the chair's selected root-cause verdict, or `없음`
- `증거:` two to four pointers
- `수정:` what changed, or why it was not fixed
- `검증:` commands run and result
- `다음:` continue or stop, with the exact reason

## Final Output

End with:

- total iterations
- commits or files changed, if any
- verification commands
- remaining in-scope findings left by budget or approval boundary
- unresolved cross-axis tradeoffs needing a human decision
- out-of-axis findings skipped by design
