# Changelog

All notable changes to this project will be documented in this file.

This project follows pre-1.0 SemVer. Until the CLI and persisted state model
are stable enough for 1.0, breaking user-facing changes bump the `0.x.0`
minor version instead of moving to `x.0.0`.

## 0.13.1 - 2026-06-24

- Fixed `leaf review --json` and the web review API hiding broken reference
  files as empty markdown; reference read failures now surface the failing path.
- Fixed `leaf serve --port 0` printing `127.0.0.1:0` instead of the actual
  ephemeral port assigned by the OS.
- Fixed `leaf update` panicking on Windows (`uri scheme is https, provider is
  Rustls but feature is not enabled: rustls`). The Windows build compiles ureq
  with only the `native-tls` feature, but ureq still defaults its TLS provider to
  Rustls, so the first https request aborted. The self-update agent now selects
  `TlsProvider::NativeTls` explicitly on Windows. (Windows users on 0.12.0 /
  0.13.0 must reinstall once via the installer, since their `leaf update` cannot
  reach the network.)

## 0.6.0 - 2026-06-23

- Added the `leaf:tend` skill — keeps the pressed knowledge graph true to current
  code. It sweeps pressed nodes via `leaf graph`, verifies each `pressed.md`
  claim independently against the repo, and proposes 🟢 keep / 🟡 correction-banner
  / 🔴 supersede per leaf. The frozen pressed body is never rewritten: surface
  drift gets an append-only correction banner, a reversed core decision is handed
  off to `leaf fall --reason superseded` + a new superseding leaf. Irreversible
  actions require user confirmation. Registered in `using-leaf`; distinct from
  `polish` (prose), `press` (first pressing), and `review` (human handoff).

## 0.12.0 - 2026-06-23

- Added `leaf next <slug>` — crosses a phase boundary as a real machine event.
  Before advancing it checks the phase being left for a `<!-- leaf:polish-pending -->`
  marker; if present it **pauses (멈칫)** and asks for `leaf:polish` instead of
  silently advancing. Polishing removes the marker, then `leaf next` scaffolds the
  next phase and updates the `current phase` / `current gate` lines in
  `00-status.md`.
- `leaf new` now scaffolds **only the Learn phase**; later phases are grown
  lazily by `leaf next`. (Breaking: a fresh sprout no longer contains
  `02-Example/`, `03-Architect/`, `04-Feedback/` until you advance into them.)
- Expanded `leaf doctor` with `boundary_unpolished` — a non-blocking WARN for any
  already-passed phase whose polish marker is still present, so a skipped
  boundary polish is visible instead of silent. Detection is line-anchored on the
  machine token, so prose that merely mentions it does not false-positive.

## 0.5.0 - 2026-06-23

- `polish` skill: removing a phase's `<!-- leaf:polish-pending -->` marker is now
  the defined final step of a boundary polish — the single signal `leaf next` and
  `leaf doctor` read for "this phase is polished."
- `work` / `autopilot` skills: phase boundaries are now crossed with `leaf next`,
  which pauses when the phase is unpolished; `using-leaf` documents the command
  and raises the required CLI to `leaf` ≥ 0.12.0.

## 0.11.0 - 2026-06-22

- Added `leaf graph` — exports the pressed-leaf knowledge graph, with nodes for
  each pressed leaf and edges derived from `linked.md` predicates (`cites`,
  `refines`, `supersedes`, `depends_on`, `derived_from`, `related_to`).
- Expanded `leaf doctor` to validate citable-knowledge structure for `leaf list`
  and graph readiness: `pressed.md` must carry OKF-compatible YAML frontmatter,
  `linked.md` must hold parseable `predicate -> target` graph edges, and both
  belong only in `.leaf/02-leaves`.
- Added a scaffold placeholder reflecting the polish phase-boundary model.
- Aligned the LEAF lifecycle validation contract across `doctor` and `review`.

## 0.4.0 - 2026-06-22

- Added the `leaf:split` skill — a judgment framework for splitting one work
  item into separate leaves. It works in three layers: (a) whether/when to
  split, (b) which single dominant grain to cut along (an open cut-axis menu
  with a reject gate, recursive for a load-bearing second axis), and (c) how the
  resulting pieces order and link. The whether/when layer reuses `learn`'s Split
  Check rather than inventing new criteria. This first increment carries the
  judgment only; execution automation (parent fall, child new, link wiring) and
  a machine-readable dependency-edge model are out of scope.
- Wired `split` into the `using-leaf` router, added a `learn` Split Check
  pointer, and mirrored the skill into the `leaf-codex` plugin.

## 0.3.1 - 2026-06-21

- Removed the Codex SessionStart hook registration so `leaf:using-leaf` is no
  longer injected into every Codex session as additional context. Codex still
  gets the LEAF skills through the plugin manifest; agents load `using-leaf` on
  demand instead.
- Added `$leaf:install` as a Codex-compatible skill so the CLI installer appears
  through Codex's supported skill/slash surface. The Claude-style
  `/leaf:install` command remains available from `plugins/leaf/commands/`.
- Simplified the install skill and command to choose installers by current OS
  only, without special-casing source checkouts via `cargo install --path .`.
- Removed the unsupported Codex plugin `commands` manifest field and updated
  manifest validation to require the install skill instead.
- Split the Codex marketplace entry onto a hook-free plugin root so Claude keeps
  its SessionStart hook while Codex no longer discovers `hooks/hooks.json`.

## 0.2.1 - 2026-06-21

- Added a `/leaf:install` command that installs the `leaf` CLI, auto-detecting
  the source repo (`cargo install --path .`) versus a normal checkout
  (`brew install hoetaek/tap/leaf`, falling back to `cargo install --git`), then
  verifying `leaf --version`.
- Strengthened the SessionStart CLI-missing notice: instead of a passive install
  hint, it now mandates that the agent surface `/leaf:install` to the user
  (without installing the binary itself), mirroring the imperative tone of the
  `using-superpowers` entry skill. The `using-leaf` skill body carries the same
  nudge with the raw `brew`/`cargo` commands as a fallback for platforms without
  the slash command.

## 0.2.0 - 2026-06-21

- Renamed the `clean` skill to `polish` (clearer that it polishes a LEAF
  document into a current report, not code/working-tree cleanup), and renamed
  `done` to `press`, narrowing it to the single press action — writing a citable
  `pressed.md` digest. The keep / press / fall **decision** and the lightweight
  fall (`leaf fall`) and keep actions moved into `using-leaf` under a new
  "Ending a leaf" section, so the always-injected router owns the close-out
  decision while `press` stays a focused on-demand skill. **Breaking:** reinstall
  and reload to pick up the new names.

## 0.1.0 - 2026-06-20

- The LEAF skills now ship as a Claude Code + Codex **plugin marketplace**
  self-hosted from this repo. Install with `/plugin marketplace add hoetaek/leaf`
  then `/plugin install leaf@leaf` (Claude Code), or `codex plugin marketplace
  add hoetaek/leaf` then `codex plugin add leaf@leaf` (Codex 0.125+). The skills
  moved under `plugins/leaf/skills/` (bodies unchanged); the `npx skills add`
  path is deprecated.
- Added a `using-leaf` skill that routes to the right LEAF gate skill, and a
  SessionStart hook that injects it (platform-aware: Claude Code / Cursor /
  Codex / SDK) and notes when the `leaf` CLI is missing without blocking the
  session.
- Added `scripts/validate-manifests.mjs` (with `--audit`) and
  `scripts/lint-shell.sh`, both run in CI, to keep the dual-runtime manifests
  version-synced with `Cargo.toml` and the hook scripts lint-clean.
- Removed the `leaf-reversed` skill (no longer used). The plugin now ships seven
  skills; `using-leaf` and the README skills list no longer reference it.
- Dropped the redundant `leaf-` prefix from the six gate skills now that both
  Claude Code and Codex namespace plugin skills (`/leaf:clean`, `$leaf:clean`):
  `leaf-clean`→`clean`, `leaf-done`→`done`, `leaf-learn`→`learn`,
  `leaf-profile`→`profile`, `leaf-soul`→`soul`, `leaf-work`→`work`. The entry
  skill `using-leaf` keeps its name (matching the `using-superpowers`
  convention). **Breaking:** reinstall and reload to pick up the new names.
- The plugin now **versions independently of the `leaf` CLI**, starting at
  `0.1.0` (previously pinned to the CLI's `0.8.0`), so plugin-only changes ship
  to users without waiting for a CLI release. The plugin declares a
  compatibility floor of **`leaf` CLI ≥ 0.8.0** (documented in the README and
  `using-leaf`). `validate-manifests.mjs` now checks the four manifests agree
  with each other rather than matching `Cargo.toml`.

## 0.10.0 - 2026-06-21

- Added `leaf update`: the installed binary fetches the latest stable GitHub
  release, verifies its sha256, and atomically self-replaces (a `claude install`
  analog). Direct implementation (`ureq` + `self-replace`, no async runtime); the
  asset is resolved from the release's own `dist-manifest.json` rather than a
  hardcoded name, so it tracks whatever targets are distributed. Never downgrades;
  "already up to date" is a no-op. A Homebrew-managed binary is detected and left
  untouched with guidance to use `brew upgrade` instead.
- After a successful `leaf update`, a best-effort notice reports when the
  installed leaf **plugin** is behind the latest, with the Claude Code and Codex
  update commands. The plugin is owned by the marketplace, so this only advises;
  any detection failure is silent and never affects the update.

## 0.9.1 - 2026-06-21

- Added a native Windows ARM64 build (`aarch64-pc-windows-msvc`). Windows on ARM
  no longer relies on x64 emulation — the PowerShell installer now fetches a
  native ARM64 binary on those machines. (`onig`/`oniguruma` cross-compiles
  cleanly for the target; verified in CI before release.)

## 0.9.0 - 2026-06-21

- Added a Windows native install path. `dist` now builds the
  `x86_64-pc-windows-msvc` target and emits a `powershell` installer, so each
  release publishes `leaf-x86_64-pc-windows-msvc.zip` and `leaf-installer.ps1`
  alongside the macOS/Linux artifacts. Windows users install with
  `powershell -ExecutionPolicy ByPass -c "irm https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.ps1 | iex"`.
- Restructured the README install section to present each OS's best path as a
  peer — macOS/Linux (Homebrew + shell installer), Windows (PowerShell
  installer), and from source — each with a `leaf --version` check, instead of
  burying the shell installer under "Other CLI install paths".

## 0.8.0 - 2026-06-18

- `leaf review` now surfaces a leaf's Learn references. The references live in
  `01-Learn/02-references/*.md` and are listed separately from the canonical
  11-source body so conclusions are not buried. In non-TTY output they are
  appended as a trailing `REFERENCES` section (filename-ascending; a missing
  folder yields none, and subfolders/non-`.md` files are ignored).
- The review reader's TUI gains a references modal picker. Press `R` to open a
  centered list of the leaf's references; navigate with `j`/`k` (`g`/`G` for
  top/bottom), press `/` for an incremental case-insensitive search, and `Enter`
  to read a reference full-screen (`Esc` to step back to the modal, `Esc` again
  to the review). Empty folders show `no references`. Clearing the search keeps
  the read reference selected. sprout/leaf/fallen rows behave identically.

## 0.7.0 - 2026-06-16

- Renamed the `leaf-idea` skill to `leaf-learn` to match what it owns: the Learn
  phase (① Intent, ② Unknowns & Context). Capture and triage stay in the same
  skill as the cheap front door to learning. Sibling skills (`leaf-work`,
  `leaf-reversed`) and the README now reference `leaf-learn`.
- `leaf-learn`'s ② Unknowns & Context now runs as a parallel fan-out: four scouts
  — Terrain (what exists), Method (how it's done), Judgment (where it forks), and
  Context (why it's this way) — search the topic concurrently and write findings
  to `01-Learn/02-references/`. The leader synthesizes them into a reading map
  (which threads to read first to find the 실마리) rather than handing over an
  answer, and quizzes the user on the core judgments before resting Learn so that
  receiving references is not mistaken for having learned them.
- `leaf list`'s interactive browser can now fall items in place: press `F` to
  move the marked sprouts/leaves (or the current row) to `.leaf/03-fallen/`. A
  centered prompt collects one shared reason for the batch — empty defaults to
  `fallen via leaf list` — with `Enter` to confirm and `Esc` to cancel. Already
  fallen or pressed rows are skipped, and per-item failures are reported without
  aborting the rest.

## 0.6.1 - 2026-06-12

- `leaf checkpoint` now snapshots folder-based gate sources
  (`02-Example/04-wireframe/`, `04-Feedback/09-reviews/`,
  `04-Feedback/10-retrospective/`) by copying the folder recursively with the
  timestamp prefix; when a gate has both a canonical file and a folder, both
  are checkpointed.
- Restored the old-layout migration procedure in `leaf-clean`, which the
  `leaf doctor` `old_stage_dir_present` warning routes to; it was dropped in
  the `leaf-done` split. The repair table now maps each old stage dir to its
  own canonical dir, covers both legacy pressed paths, and translates legacy
  `state` values while renaming the field.
- Pinned the transitive `time` dependency below the broken 0.3.48 release so
  fresh resolutions (e.g. `cargo install` without `--locked`) no longer fail
  to compile `ratatui-widgets`; remove the pin once a fixed `time` ships or
  0.3.48 is yanked.

## 0.6.0 - 2026-06-12

- Added a machine-global profile at `~/.config/leaf/profile.md`
  (`LEAF_CONFIG_DIR` → `$XDG_CONFIG_HOME/leaf` → `~/.config/leaf`); `leaf init`
  scaffolds it idempotently for machine-wide facts such as the user's working
  language.
- Added `leaf profile`, which prints the effective profile: the global profile
  layered with the repo-local `.leaf/PROFILE.md` behind source markers, local
  winning on conflict. It also works outside a git repository with the global
  layer alone.
- Added `leaf checkpoint <slug> --<gate>`, which copies one canonical gate
  document next to its source with a UTC `YYMMDD-HHMM` prefix, for example
  `260612-1430 03-criteria.md`. Gate flags accept names such as `--criteria`,
  numbers such as `--3`, and the explicit `--gate <gate>` form.
- Split the press/fall close-out decision into a new `leaf-done` skill;
  `leaf-clean` now focuses on cleaning gate documents into a simple, complete
  current report.
- `leaf new` now rejects slugs that already exist as an active leaf or fallen
  item instead of creating a duplicate sprout, and `leaf fall` refuses to move
  anything when both a sprout and an active leaf match the slug.
- Preview rendering keeps trailing sentence punctuation out of bare URL link
  targets, and the review reader resolves local links relative to each source
  file's directory.

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
