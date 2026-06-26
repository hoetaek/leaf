---
name: using-angry
description: Use when starting Angry Review work or when the user invokes `$angry`, asks which angry review persona to use, requests an angry review/council/panel, or needs routing among council, single-axis personas, and review/fix loops.
---

# Using Angry

`angry` is the plugin id. `Angry Review` is the plugin display label.
`council` is one skill inside the plugin, not the plugin name.

Use this as the entry/router before review work when the right angry skill is
not already obvious.

## Route

| Need | Use |
|---|---|
| Multi-angle review of one diff, PR, design, doc, plan, or artifact | `council` |
| Code craft, contracts, ownership, complexity, weak tests | `torvalds` |
| Correctness, invariants, state machines, concurrency, algorithms | `dijkstra` |
| Security, trust boundaries, auth, crypto, privileges, network | `theo` |
| Production readiness, missing edge cases, rollback, migrations | `ramsay` |
| Product taste, feature bloat, choices dumped on the user | `jobs` |
| Visual/UI design, clutter, dishonest interface, removable decoration | `rams` |
| Prose, docs, PR text, comments, copy | `orwell` |
| Understanding, first principles, hand-waving, cargo-culted plans | `feynman` |
| Falsifiability, success criteria, metrics, claims that cannot fail | `pauli` |
| Debugging diagnosis, evidence vs theory, symptom-only fixes | `sherlock` |
| Finished-work judgment: remarkable vs merely competent | `ego` |
| Good-enough ceiling, missed standard, pushing the work further | `fletcher` |
| Review/fix/review with panel | `loop-council` |
| Review/fix/review with maintainer code lens | `loop-torvalds` |

## Rules

- If the user says only `$angry`, explain the plugin is `angry` and choose
  `council` only when the scope needs 3-5 review axes.
- For one clear axis, invoke that persona directly. Do not route everything
  through `council`.
- Use `loop-*` only when the user asks to fix and repeat, or clearly wants a
  bounded review/fix loop.
- Profane or insulting tone is off unless the user explicitly asks for it.
- Keep each persona on one axis. Do not make `theo` do general code review or
  `torvalds` do security theater.

## Examples

- `$angry:council review this PR`
- `$angry:torvalds review this diff`
- `$angry:theo review this auth change`
- `$angry:orwell clean up this PR body`
- `$angry:loop-council review, fix the top issue, and repeat twice`
