# LEAF lifecycle

Read only after `using-leaf` selects actual LEAF work. Direct work does not load
this lifecycle, require the CLI, or create a `.leaf/` record.

**Leaf before tree:** validate one cheap, inspectable instance before scaling.

LEAF closes four kinds of uncertainty in order:

| Phase         | What it makes you able to do                                                          |
| ------------- | ------------------------------------------------------------------------------------- |
| **Learn**     | Judge what the work needs (① Intent · ② Unknowns & Context)                           |
| **Example**   | Prove one cheap instance before scaling (③ Criteria · ④ Wireframe)                    |
| **Architect** | Generalize it into a shippable generator (⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact) |
| **Feedback**  | Confirm it holds, then settle what was established (⑨ Review · ⑩ Retrospect)          |

## Fast-track LEAF

For a known sprout continuation, inspect its `00-status.md`. If it records
`route: fast-track`, or active fast-track is already known, read
[fast-track.md](fast-track.md) before routing to preserve same-request resume or
expire the old delegation before a new follow-up or scope change. This does not
require searching `.leaf/` for ordinary direct requests.

For a new LEAF request that explicitly asks for fast track with no core unknown,
read [fast-track.md](fast-track.md) before creating status.
It owns the procedure budget, approval fields, escalation, and expiration.
Otherwise use discovery-heavy `learn` → `work`, with applicable approval stops.

## Which skill to use

| Skill       | Use it for                                                                                               |
| ----------- | -------------------------------------------------------------------------------------------------------- |
| `soul`      | **First for actual LEAF work.** Conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn`     | Capture/triage an idea and run fast-track 또는 discovery-heavy Learn                                     |
| `split`     | Decide whether/how to split one work item into separate leaves                                           |
| `autopilot` | Run the gates automatically after the human-reviewed why/what/wireframe triple                           |
| `work`      | Carry a sprout from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩                                     |
| `polish`    | Make the cumulative document read as one connected report at each phase boundary                         |
| `press`     | Press a reference-worthy leaf into a citable digest                                                      |
| `tend`      | Sweep the pressed knowledge graph and reconcile drift with current code (banner/supersede)               |
| `profile`   | Read/update the machine-global and repo-local LEAF profiles                                              |

Use only the references and specialist skills the current phase needs.

## Ending a leaf

After ⑩, `polish` the cumulative whole at the active route's depth. Then set an
exact active `route: fast-track` to `fast-track (expired)`, expire its approvals,
and decide the end with the user:

- **keep** — useful but not citable; note it in `00-status.md`.
- **press** — reference-worthy (reusable decision, pattern, lesson); invoke `press`.
- **fall** — stop carrying it: `leaf fall <slug> --reason "<abandoned|superseded|parked|split|invalidated|archived|completed-not-reference-worthy>"`.

Don't keep or press just because effort was spent.

## The CLI is the body

The `leaf` CLI gives actual LEAF work a repo-local `.leaf/` body (`leaf
init`, `leaf new`, `leaf next`, `leaf doctor`); requires `leaf` ≥ 0.12.0.
execution-ready direct work does not need the CLI. If it escalates, start the
normal lifecycle at Learn; do not synthesize a scaffold after implementation.
`leaf next` crosses a formal phase boundary, pausing (멈칫) if the phase you are
leaving still carries its `<!-- leaf:polish-pending -->` marker — polish removes
it. If `leaf` is not on PATH,
tell the user to run the install entry (`$leaf:install` in Codex, `/leaf:install`
in Claude) before creating or advancing LEAF records.
