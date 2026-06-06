# Layout

How a project's persistent files are named and organized. Read this before you
write files.

**The leaf-work folder records the thinking process.** Each file says what you
did, learned, and decided at that gate; the artifact itself — essay, video,
code — lives wherever you keep your work, and leaf-work only records what was
done. Keep one folder per project. The top level is split by the four LEAF
phases, each with a two-digit phase prefix so `ls` sort shows the work in order.
Inside each phase, files keep their two-digit gate prefix.

## Recommended structure

```text
<project-name>/
├── 00-status.md                              dashboard: current gate, progress, next action
├── README.md                                 project description only (optional)
│
├── 01-Learn/
│   ├── 01-intent.md                          ① Intent
│   ├── 02-unknowns.md                        ② Unknowns, sourced answers, context
│   └── 02-references/                        ② collected materials & search notes, by topic
│
├── 02-Example/
│   ├── 03-criteria.md                        ③ Criteria (purpose + requirements)
│   └── 04-wireframe/                         ④ Instance + contract (folder when interactive)
│       ├── index.html                        the instance (variation rendered)
│       ├── mock-data.json
│       └── contracts.md                      declared contract behind each placeholder
│
├── 03-Architect/
│   ├── 05-design-<artifact>.md               ⑤ Design — generator (consumes ④ contract)
│   ├── 06-critic-<artifact>.md               ⑥ Critic — design falsification (depth scales with risk)
│   ├── 07-tasks.md                           ⑦ Task graph
│   └── 08-execution.md                       ⑧ what was done — one entry per work session
│
└── 04-Feedback/
    ├── 09-review-<artifact>-v1.md            ⑨ Review/sync (file when one or two)
    └── 10-retrospect-<topic>.md              ⑩ Retrospect (file when one or two)
```

## Status dashboard

`00-status.md` is the first file to read when resuming a project. It is an
overview, not the source of truth: each gate's own file/folder remains
authoritative. Update it whenever a gate starts, becomes ready for approval,
is approved, returns, is blocked/deferred, or the next action changes materially.
Returns are historical events, not gate states; summarize them in the dashboard
and record them in the Return Log.

Use coarse progress values to avoid fake precision:

```text
0    not started
25   started / notes exist
50   core artifact drafted
75   ready for user approval
100  approved by user
```

Use these state values:

```text
not-started      gate work has not begun
active           gate work is currently being worked
needs-approval   gate work is complete enough to request explicit user approval
approved         user explicitly approved the current gate version
```

`approved` means approved for the current gate version. A later return may
invalidate or reopen it with explicit user approval. If work is blocked or
intentionally deferred, keep the gate state as `active` or `not-started` and
write the reason in `Next / Waiting on` (`blocked: <reason>` or
`deferred: <resume condition>`).

Recommended template:

```markdown
# Status

- Current phase: Learn
- Current gate: ② Unknowns & Context
- First missing gate: ②
- Next action: resolve blocking unknowns and ask whether to start ③
- Latest return: -
- Return count: 0
- Last updated: YYYY-MM-DD

| Gate | State | Progress | Artifact | Next / Waiting on |
|---|---:|---:|---|---|
| ① Intent | approved | 100 | 01-Learn/01-intent.md | - |
| ② Unknowns & Context | active | 50 | 01-Learn/02-unknowns.md | resolve blocking unknowns |
| ③ Criteria | not-started | 0 | 02-Example/03-criteria.md | start after ② approval |
| ④ Wireframe | not-started | 0 | 02-Example/04-wireframe/ | - |
| ⑤ Design | not-started | 0 | 03-Architect/05-design-<artifact>.md | - |
| ⑥ Critic | not-started | 0 | 03-Architect/06-critic-<artifact>.md | - |
| ⑦ Tasks | not-started | 0 | 03-Architect/07-tasks.md | - |
| ⑧ Artifact | not-started | 0 | 03-Architect/08-execution.md | - |
| ⑨ Review | not-started | 0 | 04-Feedback/09-review-<artifact>-v1.md | - |
| ⑩ Retrospect | not-started | 0 | 04-Feedback/10-retrospect-<topic>.md | - |

## Return Log

| Date | From | To | Trigger | Reason | Affected gates | Next approval |
|---|---|---|---|---|---|---|
| - | - | - | - | - | - | - |
```

When a return happens, update the affected gate states separately. The target
gate usually becomes `active`; downstream gates may become `not-started`,
`active`, or `needs-approval` depending on what the return invalidated. Do not
use `returned` as a state.

When a gate's artifacts grow to three or more, promote the file form to a folder
inside its phase folder. The folder name uses the plural gate keyword; files
inside drop the keyword:

```text
04-Feedback/09-reviews/                       (folder form)
├── 공적서-review-v1.md
├── 공적서-review-v2.md
└── 추천서-review-v1.md

04-Feedback/10-retrospective/                 (folder form)
├── mid-process-discoveries.md
├── limitations.md
└── retrospective-2026-05-30.md
```

## Naming rules

- **One folder per project.** Do not spread one project's outputs across
  multiple top-level folders.
- **The scaffold comes first, and `00-status.md` is part of it.** Invoking
  leaf-work means standing up the four phase folders and `00-status.md` at the
  root before working any gate — there is no "LEAF without a body." A task too
  small to deserve that scaffold should not invoke leaf-work at all, rather than
  run it while skipping the files.
- **`README.md` is not the status file.** Use it only for stable project
  description or handoff notes. Current gate, progress, and next action belong
  in `00-status.md`.
- **Top-level folders are phases.** Use exactly `01-Learn/`, `02-Example/`,
  `03-Architect/`, and `04-Feedback/` when persistent files are needed. The
  numeric prefix preserves order; the phase name preserves meaning.
- **No nested project folders.** Do not create `<##>-sub-<name>/` or recursive
  leaf-work children. If work is too large for one task line, split it inside
  `03-Architect/07-tasks.md`. If it truly needs an independent LEAF cycle,
  create a separate sibling project folder and reference that path from the task
  graph.
- **② is one gate, not a merge.** Unknown surfacing, reference search, sourced
  answers, context notes, assumptions, and unresolved questions stay together in
  `01-Learn/02-unknowns.md`. This is deliberate: the agent naturally records a
  question and then updates the same entry with what was found. Put bulky source
  material in `01-Learn/02-references/`, but summarize the useful answer back in
  `02-unknowns.md`.
- **③ is one gate, not a merge.** Criteria combines purpose and requirements
  because both are pre-instance judgment. Keep purpose and requirements as
  visible sections inside `02-Example/03-criteria.md`.
- **Never merge across a produce/consume boundary.** ③ Criteria vs ④ Wireframe
  must stay separate because the concrete instance must pass the criteria and can
  falsify them. ④ Wireframe vs ⑤ Design must stay separate because the locked
  contract is consumed by the generator. `03+04`, `04+05`, and `03+04+05` are
  forbidden.
- **Match the file name to the gate vocabulary.** As a file (1–2 artifacts) the
  name carries the gate keyword: `01-intent.md`, `02-unknowns.md`,
  `03-criteria.md`, `05-design-<artifact>.md`, `07-tasks.md`, `08-execution.md`,
  `09-review-<artifact>-v1.md`, `10-retrospect-<topic>.md`. Inside a folder form
  (`09-reviews/`, `10-retrospective/`) files drop the keyword because the folder
  carries it.
- **File or folder by count.** Keep gate outputs as prefix files inside their
  phase folder when there are one or two; promote to a folder when three or more
  pile up (`01-Learn/02-references/`, `04-Feedback/09-reviews/`,
  `04-Feedback/10-retrospective/`).
- **`08-execution.md` is a running log.** Append one entry per work session —
  what you did, what came of it, what's next. The artifact itself lives outside
  leaf-work; this file keeps the record of what was done.
- **Reuse**: copy the folder structure as the starting point for the next
  project. The previous retrospective then feeds the next project's ② — its
  lessons update the ② checklist, and its limitations (what stayed unresolved,
  where the conclusions stop) seed future ① intents.

