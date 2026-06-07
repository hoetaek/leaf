# Layout

How a project's persistent files are named and organized. Read this before you
write files.

**The leaf-work folder records the thinking process.** Each file says what you
did, learned, and decided at that gate; the artifact itself — essay, video,
code — lives wherever you keep your work, and leaf-work only records what was
done. `leaf-work` uses the `leaf` CLI for this folder: initialize `.leaf/` in
the repo and create one project folder per slug. Inside that project folder,
the top level is split by the four LEAF phases, each with a two-digit phase
prefix so `ls` sort shows the work in order. Inside each phase, files keep their
two-digit gate prefix.

## Leaf CLI workspace

Run these before writing gate files:

```bash
leaf init
leaf new <slug>
```

`leaf init` creates the repo-local workspace:

```text
.leaf/
├── seeds/
└── leaves/
```

`leaf new <slug>` creates exploratory work under `.leaf/seeds/<slug>/`. Seeds
are ideas: they can die, split, or be rewritten. When work becomes committed and
active, it belongs under `.leaf/leaves/<slug>/`; use the current lifecycle
command when one exists, otherwise move it only after explicit user approval and
record the promotion in `01-Learn/01-intent.md`.

## Recommended structure

```text
.leaf/seeds/<slug>/                         exploratory idea seed
# or .leaf/leaves/<slug>/                   committed active leaf
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
│   ├── 05-design.md                          ⑤ Design — generator (consumes ④ contract)
│   ├── 06-critic.md                          ⑥ Critic — gate always runs; file created when depth/risk warrants
│   ├── 07-tasks.md                           ⑦ Task graph
│   └── 08-execution.md                       ⑧ lazy execution log / handoff
│
└── 04-Feedback/
    ├── 09-review.md                          ⑨ Review/sync, lazy until review happens
    └── 10-retrospect.md                      ⑩ Retrospect, lazy until close
```

The current CLI scaffolds the core seed files (`00-status.md`, gates ①-⑤, ⑦,
and the phase folders). Create lazy files such as `06-critic.md`,
`08-execution.md`, `09-review.md`, and `10-retrospect.md` only when the gate
needs them.

## Status dashboard

`00-status.md` is the first file to read when resuming a project. It is an
overview, not the source of truth: each gate's own file/folder remains
authoritative. Update it whenever a gate starts, becomes ready for review,
completes, needs explicit approval, is approved, returns, is blocked/deferred,
or the next action changes materially. Returns are historical events, not gate
states; summarize them in the dashboard and record them in the Return Log.

Use coarse progress values to avoid fake precision:

```text
0    not started
25   started / notes exist
50   core artifact drafted
75   reviewed / ready for phase review or escalated approval
100  complete; user-approved when approval was required
```

Use these state values:

```text
not-started      gate work has not begun
active           gate work is currently being worked
review-ready     gate work is complete enough for the gate authoring review loop
complete         gate passed inside the current phase; no explicit approval was required
needs-approval   phase boundary or escalated gate is ready for explicit user approval
approved         user explicitly approved the phase transition or escalated gate
```

Do not mark every completed gate `approved`. Use `complete` for ordinary gates
the agent validated inside the current phase. Use `approved` only when
the user explicitly approved a phase transition, escalated gate, or passed
snapshot. A later return may invalidate or reopen a `complete` or `approved`
gate; if the return crosses a previously approved phase boundary, escalate
again. If work is blocked or intentionally deferred, keep the gate state as
`active` or `not-started` and write the reason in `Next / Waiting on`
(`blocked: <reason>` or `deferred: <resume condition>`).

Recommended template:

```markdown
# Status

- Current phase: Learn
- Current gate: ② Unknowns & Context
- First missing gate: ②
- Next action: resolve blocking unknowns; then ask whether to approve Learn and start Example
- Next approval point: Learn phase -> Example phase
- Latest return: -
- Return count: 0
- Last updated: YYYY-MM-DD

| Gate | State | Progress | Artifact | Next / Waiting on |
|---|---:|---:|---|---|
| ① Intent | complete | 100 | 01-Learn/01-intent.md | - |
| ② Unknowns & Context | active | 50 | 01-Learn/02-unknowns.md | resolve blocking unknowns |
| ③ Criteria | not-started | 0 | 02-Example/03-criteria.md | start after Learn phase approval |
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

When a return happens, update the affected gate states separately. The target
gate usually becomes `active`; downstream gates may become `not-started`,
`active`, `review-ready`, `complete`, or `needs-approval` depending on what the
return invalidated. Do not use `returned` as a state.

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

- **One leaf folder per project.** Do not spread one project's process files
  across multiple `.leaf/seeds/` or `.leaf/leaves/` folders.
- **The scaffold comes first, and `00-status.md` is part of it.** Invoking
  leaf-work means running `leaf init` / `leaf new <slug>` and using the
  resulting `.leaf/seeds/<slug>/` scaffold before working any gate — there is no
  "LEAF without a body." A task too small to deserve that scaffold should not
  invoke leaf-work at all, rather than run it while skipping the files.
- **`README.md` is not the status file.** Use it only for stable project
  description or handoff notes. Current gate, progress, and next action belong
  in `00-status.md`.
- **Top-level folders inside the leaf are phases.** Use exactly `01-Learn/`,
  `02-Example/`, `03-Architect/`, and `04-Feedback/` inside the seed/leaf
  folder. The numeric prefix preserves order; the phase name preserves meaning.
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
- **Match the file name to the gate vocabulary.** The CLI uses stable names for
  the common single-artifact case: `01-intent.md`, `02-unknowns.md`,
  `03-criteria.md`, `04-wireframe.md`, `05-design.md`, `07-tasks.md`, plus lazy
  `06-critic.md`, `08-execution.md`, `09-review.md`, and `10-retrospect.md`.
  When a gate has several artifacts, use a folder form and put named files
  inside it.
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
