# Layout

How a LEAF project's persistent files are named and organized. Read this before
writing gate files.

The LEAF folder records the thinking process: what you learned, decided, built,
reviewed, and closed. The artifact itself — document, code, video, design, or
prototype — may live elsewhere; LEAF records why it exists and how it was made.

## Stage Model

Use stage language in human-facing prose:

```text
.leaf/
├── 01-sprouts/  incomplete work: Learn through ⑧ Artifact / Execution
├── 02-leaves/   ⑧-passed work under Feedback and reference-worthy leaves
└── 03-fallen/   discarded or archived work, including completed non-reference work
```

Learn and post-Learn work stay in the same sprout through ⑧ Artifact /
Execution. After ⑧ passes, `work` moves the project folder to
`.leaf/02-leaves/<slug>/`, updates `00-status.md` for Feedback, and runs
`leaf doctor` before ⑨ Review / Sync and ⑩ Retrospect. Immediately after ⑩,
invoke `done`.

## CLI Start

Run these before writing gate files:

```bash
leaf init
leaf new <slug>
```

`leaf new <slug>` creates or resumes the sprout project folder. Use lowercase
ASCII kebab-case slugs. If a likely matching sprout already exists, resume it
instead of creating a duplicate.

## Project Structure

```text
.leaf/01-sprouts/<slug>/                         LEAF work through ⑧ Artifact / Execution
├── 00-status.md                              dashboard: stage, gate, progress, next action
│
├── 01-Learn/
│   ├── 01-intent.md                          ① Intent
│   ├── 02-unknowns.md                        ② Unknowns, sourced answers, context
│   ├── 02-references/                        ② collected materials and search notes
│   └── 02-experiments/                       ② experiment logs, summarized back to 02-unknowns.md
│
├── 02-Example/
│   ├── 03-criteria.md                        ③ Criteria
│   └── 04-wireframe/                         ④ Instance + contract when interactive
│       ├── index.html
│       ├── mock-data.json
│       └── contracts.md
│
├── 03-Architect/
│   ├── 05-design.md                          ⑤ Design
│   ├── 06-critic.md                          ⑥ Critic, lazy when depth/risk warrants
│   ├── 07-tasks.md                           ⑦ Task graph
│   └── 08-execution.md                       ⑧ execution log / handoff
│
└── 04-Feedback/
    ├── 09-review.md                          ⑨ Review/sync
    └── 10-retrospect.md                      ⑩ Retrospect
```

After ⑧ passes, `work` moves the same project folder to:

```text
.leaf/02-leaves/<slug>/                          Feedback and reusable leaf record
└── 04-Feedback/
    ├── 09-review.md                          ⑨ Review/sync
    └── 10-retrospect.md                      ⑩ Retrospect
```

Completed reference-worthy leaves may also contain after `done` presses
them:

```text
.leaf/02-leaves/<slug>/pressed.md                citable digest for future reference
```

## Status Dashboard

`00-status.md` is the first file to read when resuming. It is an overview, not
the source of truth; each gate file remains authoritative. Update it whenever
the current phase/gate, next action, review status, approval need, return, or
closure condition changes.

Use coarse progress values:

```text
0    not started
25   started / notes exist
50   core artifact drafted
75   reviewed / ready for phase review or escalated approval
100  complete; user-approved when approval was required
```

Use these gate status values:

```text
not-started      gate work has not begun
active           gate work is currently being worked
review-ready     gate work is complete enough for the gate authoring review loop
complete         gate passed inside the current phase; no explicit approval was required
needs-approval   phase boundary, escalated gate, or pre-execution Architect
                 snapshot is ready for explicit user approval
approved         user explicitly approved the phase transition, escalated gate,
                 or pre-execution Architect snapshot
```

For fallen items, use a `fallen reason` such as `abandoned`, `superseded`,
`split`, `invalidated`, `archived`, or `completed-not-reference-worthy`. Do not
use `fallen` as an ordinary gate status.

Recommended template:

```markdown
# Status

- stage: sprout
- current phase: Learn
- current gate: ② Unknowns & Context
- first missing gate: ② Unknowns & Context
- next action: resolve blocking unknowns; then continue to ③ Criteria
- next approval point: Learn phase -> Example phase
- latest return: -
- return count: 0
- last updated: YYYY-MM-DD

| Gate | Status | Progress | Artifact | Next / Waiting on |
|---|---:|---:|---|---|
| ① Intent | complete | 100 | 01-Learn/01-intent.md | - |
| ② Unknowns & Context | active | 50 | 01-Learn/02-unknowns.md | resolve blocking unknowns |
| ③ Criteria | not-started | 0 | 02-Example/03-criteria.md | start after Learn closes |
| ④ Wireframe | not-started | 0 | 02-Example/04-wireframe/ | - |
| ⑤ Design | not-started | 0 | 03-Architect/05-design.md | - |
| ⑥ Critic | not-started | 0 | 03-Architect/06-critic.md | - |
| ⑦ Tasks | not-started | 0 | 03-Architect/07-tasks.md | - |
| ⑧ Artifact | not-started | 0 | 03-Architect/08-execution.md | - |
| ⑨ Review | not-started | 0 | 04-Feedback/09-review.md | - |
| ⑩ Retrospect | not-started | 0 | 04-Feedback/10-retrospect.md | - |

## Return Log

| Date | From | To | Trigger | Reason | Affected gates | Next approval point |
|---|---|---|---|---|---|---|
| - | - | - | - | - | - | - |
```

When a return happens, update the affected gate statuses separately. The target
gate usually becomes `active`; downstream gates may become `not-started`,
`active`, `review-ready`, `complete`, or `needs-approval` depending on what the
return invalidated. Do not use `returned` as a status.

## Naming Rules

- **One project folder per LEAF work item.** Do not spread one item's process
  files across multiple sprout or leaf folders.
- **The scaffold comes first.** Invoking LEAF means using `leaf init` /
  `leaf new <slug>` and keeping the gate files in that project folder. A task too
  small for the scaffold should not invoke LEAF.
- **Top-level folders are phases.** Use exactly `01-Learn/`, `02-Example/`,
  `03-Architect/`, and `04-Feedback/` inside the project folder.
- **No nested project folders.** Split large work inside `03-Architect/07-tasks.md`;
  create a sibling sprout only when the work needs an independent LEAF cycle.
- **② is one gate.** Unknowns, reference search, sourced answers, assumptions,
  and unresolved questions stay indexed in `01-Learn/02-unknowns.md`. Put bulky
  source material in `01-Learn/02-references/` and experiments in
  `01-Learn/02-experiments/`, then summarize the useful answer back.
- **Never merge produce/consume boundaries.** Keep ③ Criteria, ④ Wireframe, and
  ⑤ Design separate so disagreement stays visible.
- **File or folder by count.** Keep gate outputs as prefix files when there are
  one or two. Convert to a folder when three or more pile up.
- **`08-execution.md` is a running log.** Append one entry per work session: what
  you did, what came of it, and what is next.
- **Reuse comes from closure.** Retrospect lessons feed the next project's ②;
  limitations and unresolved boundaries can start future ① intents.
