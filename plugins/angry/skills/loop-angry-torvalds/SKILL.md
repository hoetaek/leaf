---
name: loop-angry-torvalds
description: "Bounded review/fix loop using angry-torvalds as the engine — find the highest-priority issue, fix it, verify, repeat within budget. For a diff, PR, branch, or repo. Triggers: angry-torvalds loop, loop-angry-torvalds, maintainer review loop."
---

# Loop Angry Torvalds

Run a bounded best-practice review/fix loop. This skill owns the loop mechanics
only: select one root-cause issue, fix it, verify it, and repeat until no
material issue remains within the agreed budget.

Use `angry-torvalds` as the review engine. Before the first review pass, read
`../angry-torvalds/SKILL.md` if available and follow its voice, candidate
generation, evidence, and priority rules. If that file is unavailable, say so
and continue with the loop rules here without pretending to be that skill.

## Scope

Loop on the highest-priority issue in these classes, in this order:

- correctness bug
- data loss or corrupt migration
- security hole or trust-boundary failure
- broken public/API/CLI/config/data contract
- unsafe concurrency
- unsafe deploy, release, rollback, or irreversible operation
- tests that fail to protect changed behavior or make safe change impossible
- tangled ownership, mixed responsibilities, or hidden coupling
- bad precedent: names, file placement, layer boundaries, APIs, config shape, or
  examples future work will likely copy incorrectly
- excessive complexity where a smaller existing/local/stdlib pattern works
- naming or interface shape that creates real maintenance ambiguity

Do not loop on pure taste, formatting, speculative architecture, performance
micro-optimizations, or "nice to have" refactors without a concrete maintenance
or correctness cost.

## Budget

If the user does not provide a budget, default to **3 iterations max**. Stop
after the budget and report the remaining candidates. Continue only when the
user explicitly asks for more.

## Loop

1. Read before judging: `git status`, current diff, touched entry points, core
   flow, and existing tests/checks for the touched behavior.
2. Run one `angry-torvalds` review pass to produce serious candidates and pick
   the highest-priority complaint.
3. Continue only if that selected complaint is in Scope. If it is not, stop and
   report it as a skipped out-of-scope issue.
4. Prove the selected issue from code with file/line evidence and state whether
   it is verified fact or inference.
5. Apply the smallest root-cause fix for that one issue only. Do not opportunistically
   fix lower-priority smells in the same pass.
6. Run the cheapest relevant verification that would fail if the fix regressed.
   Expand tests only when the touched contract needs it.
7. Re-run the angry review against the new state. Continue only if the new
   highest-priority issue is still in Scope and the iteration budget remains.

## Termination

Stop the loop when any of these is true:

- no material best-practice issue remains in Scope
- the next issue is only taste, formatting, speculative design, or low-value
  cleanup
- the iteration budget is exhausted
- the fix requires user approval because it is destructive, changes public
  behavior broadly, alters deployment/release state, or needs external secrets
- the same unresolved blocker repeats and cannot be progressed locally

When stopping, report remaining in-scope candidates separately as not fixed.

## Output Per Iteration

Keep each iteration short:

- `회차:` loop number
- `욕먹을 1순위:` the one selected root-cause issue, or `없음`
- `증거:` two to four file/line references
- `수정:` what changed, or why it was not fixed
- `검증:` commands run and result
- `다음:` continue or stop, with the exact reason

## Final Output

End with:

- total iterations
- commits or files changed, if any
- verification commands
- remaining in-scope issues left by budget or approval boundary
- out-of-scope issues skipped by design
