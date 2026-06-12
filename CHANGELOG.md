# Changelog

All notable changes to this project will be documented in this file.

This project follows pre-1.0 SemVer. Until the CLI and persisted state model
are stable enough for 1.0, breaking user-facing changes bump the `0.x.0`
minor version instead of moving to `x.0.0`.

## Unreleased

- Added a machine-global profile at `~/.config/leaf/profile.md`
  (`LEAF_CONFIG_DIR` → `$XDG_CONFIG_HOME/leaf` → `~/.config/leaf`); `leaf init`
  scaffolds it idempotently for machine-wide facts such as the user's working
  language.
- Added `leaf profile`, which prints the effective profile: the global profile
  layered with the repo-local `.leaf/PROFILE.md` behind source markers, local
  winning on conflict. It also works outside a git repository with the global
  layer alone.

## 0.5.1 - 2026-06-12

- Added `leaf tree --demo` as a public growth preview that stacks full-size
  recursive tree renders for 0, 3, 10, 20, 50, and 100 synthetic leaves; the
  demo keeps color by default, supports `--plain`, and deliberately excludes
  the compressed left-to-right strip because it loses the approved tree shape.
- Made `leaf tree` responsive to terminal width: it scales tree art within a
  112-column maximum, wraps headers and legends, and falls back to a compact
  semantic summary below 32 columns instead of overflowing.
- Tuned the `leaf tree` foliage model so larger leaf counts visibly fill in the
  green crown through the 100-leaf saturated stage while keeping growth bounded.
- `leaf init` now scaffolds `.leaf/PROFILE.md`; leaf-work ⑩ Retrospect
  consolidates behavior traits and recurring facts into it with
  strengthen/replace/evict rules and a 30-line cap; and `leaf-soul` recalls it
  at the start of LEAF work.
- The `leaf-idea` Learn-close question now shows the `01-Learn/02-references/`
  file tree first, with a one-line note per file, so the user judges whether
  enough material was gathered from the files rather than from memory.

## 0.5.0 - 2026-06-11

- Added `leaf review <slug>` to open the review reader for one leaf-work item
  directly, with plain text output on non-TTY stdout.
- Merged the `leaf-press` and `leaf-fall` skills into a single `leaf-clean`
  skill that tends the `.leaf/` workspace: pressing citable digests, moving
  non-reference-worthy work into fallen, and acting as the migration operator
  for old layouts reported by `leaf doctor` (old stage dirs, top-level pressed
  dir, legacy `state` / `fall reason` status fields).
- Press now also records cross-leaf citations in a per-leaf `linked.md` —
  outgoing `Cites` with evidence, plus a `Cited By` snapshot scanned from other
  leaves' `linked.md` at press time.

## 0.4.1 - 2026-06-10

- Improved `leaf list` review and preview Markdown rendering with
  pulldown-cmark parsing, Codex-style syntax highlighting, quote gutters,
  vertical rhythm, heading markers, and wrapped table/link handling.
- Added review reader layout polish including cached rendered body lines,
  content padding, an end-of-document marker, and restored 3-line mouse wheel
  scrolling.
- Clarified LEAF skill guidance around `leaf-soul`, Learn close checks, preview
  gate boundaries, and review voice.
- Fixed review text wrapping, Markdown table rendering, `.leaf` symlink
  directory handling, and inventory row ordering by gate.
- Updated the transitive `time` dependency to address `RUSTSEC-2026-0009` and
  raised the declared Rust version to 1.88.

## 0.4.0 - 2026-06-10

- `leaf-press` now writes a paper-style `## Press Abstract` into the source
  `00-status.md` while also creating the citable `.leaf/04-pressed/{slug}.md`
  digest.
- `leaf list` can open a read-only full review reader for a selected active
  leaf, combining the current gate source files from the originals without
  creating a persistent derived review file.
- The review reader renders lightweight Markdown, supports scrolling,
  half-page movement with `d`/`u`, `r` refresh, live source refresh while open,
  prominent file separators, and `Esc`/`q` back to the inventory list.
- `leaf list` preview placement now follows a gh-dash-style auto layout:
  right-side preview on wide terminals, bottom preview when the table would
  become too narrow.
- `leaf list` TUI polish: clearer two-line inventory header, readable selected
  rows, no redundant `state` column, narrower phase column, title-first row
  labels when available, tab-delimited multi-row copy output, and a larger
  preview area.
- LEAF skill guidance now includes a shared `leaf-soul` conduct/reporting
  layer and clearer split between idea/work/press/fall capabilities.

## 0.3.0 - 2026-06-09

### Breaking

- `.leaf/` bucket directories now carry lifecycle-order prefixes: `seeds` →
  `01-seeds`, `leaves` → `02-leaves`, `fallen` → `03-fallen`, `pressed` →
  `04-pressed`.
- Existing workspaces migrate automatically and losslessly on the next `leaf`
  command (`fs::rename`). If both a legacy and a prefixed name exist for the
  same bucket, migration aborts without moving any files and prints guidance.
- Downgrading is not supported (older versions do not recognize the prefixed
  directory names).
- `leaf list` / JSON / TUI display labels are unchanged: `seeds`, `leaves`,
  `fallen`, `pressed`.

### Added

- Added `leaf doctor` to run workspace status diagnostics, reporting issues
  such as unreadable pressed digests.
- `leaf list` TUI: promote Learn-complete seeds into active leaves directly
  from the browser.
- `leaf list` TUI: copy rows to the system clipboard — `y` copies the current
  row, and multi-select (`space` toggle, `v` range-select, select-all over
  visible rows) copies every marked row as tab-delimited lines.
- `leaf list` TUI: mouse selection, and `r` to refresh the inventory.
- `leaf-work` skill: absorbed the experiment methodology into the Learn phase
  as a shared `experiment-log` reference — independent, cheap fact-gathering
  accumulated as a fact ladder, with a `01-Learn/02-experiments/` sidecar.
- `leaf-idea` skill: split-triage guidance, and refreshed LEAF review/split
  handoff guidance.

### Fixed

- `leaf list` TUI range-selection contract.
- Status parsing now reads only the status preamble, so fallen sections
  no longer override the parsed state.

## 0.2.0 - 2026-06-08

- Added `leaf list` to project a deterministic inventory of seeds, active
  leaves, fallen leaves, and pressed digests, with `--json` output for tooling.
- Added an interactive `leaf list` TUI browser for navigating and previewing
  LEAF items.
- Added `leaf promote <slug>` to move Learn-complete seeds into active leaves
  before Example work starts.
- Updated LEAF skill guidance so seeds are for rough ideas and Learn-phase work,
  while active leaves carry Example and later phases.
- Added a preferred-language rule to the `leaf-work` skill.

## 0.1.3 - 2026-06-07

- Reworked the README around the LEAF-first positioning, Agent Skills install
  path, LEAF loop, repo-local concepts, command reference, and project status.

## 0.1.2 - 2026-06-07

- Added LEAF agent skills for idea capture, work planning, pressing, and
  falling discarded work.
- Added `leaf fall <slug> --reason <reason>` to move active leaves into the
  fallen trash bucket with closure metadata.
- Added `.leaf/fallen/` and `.leaf/pressed/` storage buckets during
  initialization.
- Tightened `leaf-work` approval policy so ⑧ Artifact / Execution requires
  explicit approval before execution starts.

## 0.1.1 - 2026-06-06

- Added `leaf --version` support for installed-binary smoke checks.

## 0.1.0 - 2026-06-06

- Added `leaf init` to initialize repo-local `.leaf/` storage and exclude it
  from git via `.git/info/exclude`.
- Added `leaf new <slug>` to scaffold phase-gated idea seeds under
  `.leaf/seeds/`.
- Added cargo-dist release configuration with shell and Homebrew installers
  publishing to `hoetaek/homebrew-tap`.
