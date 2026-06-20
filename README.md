# leaf

**Leaf before tree.**

## Why

Let AI grow the whole tree too early and you will come back to the beginning.

The hard part will not be editing the output. It will be rediscovering what you
wanted, what the work needed, which paths were available, and why one direction
was better than another.

`leaf` exists to keep that discovery visible. Start with one cheap, inspectable
leaf. Learn what must be true. Prove one instance. Then grow the tree.

The Agent Skills guide the workflow; the `leaf` CLI gives that workflow a
repo-local body.

## Quick Start

Install the Agent Skills as a plugin. The skills ship as one plugin (`leaf`)
served from this repo as a marketplace for both Claude Code and Codex.

**Claude Code:**

```bash
/plugin marketplace add hoetaek/leaf
/plugin install leaf@leaf
```

The skills then appear namespaced as `/leaf:learn` … `/leaf:work`.
Update later with `/plugin marketplace update leaf`.

**Codex** (CLI 0.125+):

```bash
codex plugin marketplace add hoetaek/leaf
codex plugin add leaf@leaf
```

Or enable it in `~/.codex/config.toml`:

```toml
[plugins."leaf@leaf"]
enabled = true
```

Update later with `codex plugin marketplace upgrade leaf`.

> **Deprecated:** the previous `npx skills@latest add https://github.com/hoetaek/leaf`
> install path is superseded by the plugin marketplace above. If you installed
> the skills that way (a symlink under `~/.agents/skills/`), remove it and
> re-install via the plugin.

Install the `leaf` CLI that gives those skills a repo-local body. The plugin
versions independently of the CLI and **requires `leaf` CLI ≥ 0.8.0**. The
plugin does not install the CLI; it only checks for it on session start and
points you here. Homebrew is the recommended install path:

```bash
brew install hoetaek/tap/leaf
leaf --version
```

Start inside a git repository:

```bash
cd your-git-repo
leaf init
leaf new my-first-idea
```

Ask your agent to use `learn` to capture an idea and run the Learn phase on a
sprout (lock ① Intent, resolve ② Unknowns & Context); use `work` to carry a
sprout from ③ Example through ⑧ Artifact / Execution. After ⑧ passes,
`work` moves the sprout into leaves before Feedback. Immediately after ⑩
Retrospect, follow `using-leaf` ("Ending a leaf") to keep, press, or fall.

Other CLI install paths are available from the latest GitHub Release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.sh | sh
```

Or from the current source checkout:

```bash
cargo install --git https://github.com/hoetaek/leaf
```

Update the Homebrew install with:

```bash
brew update
brew upgrade hoetaek/tap/leaf
```

## The LEAF Loop

LEAF closes uncertainty in order:

| Phase | What it proves |
|---|---|
| Learn | What the work needs, learned instead of guessed |
| Example | One cheap instance can pass inspection |
| Architect | The passed instance can become reusable structure |
| Feedback | The result still holds, and the lessons carry forward |

The Learn gate contract lives in
[`learn`](plugins/leaf/skills/learn/SKILL.md); Example onward lives in
[`work`](plugins/leaf/skills/work/SKILL.md).

## Concepts

`leaf` keeps work inside a repo-local `.leaf/` workspace:

```text
.leaf/
├── 01-sprouts/
├── 02-leaves/
└── 03-fallen/
```

`01-sprouts/` holds incomplete work through Learn, Example, Architect, and
⑧ Artifact / Execution. `02-leaves/` holds ⑧-passed work while it goes through
⑨ Review and ⑩ Retrospect, then remains as completed, reference-worthy LEAF
folders.
`03-fallen/` holds discarded or archived work, including completed work that is
not useful enough to keep as a reference. Pressed digests live inside the source
leaf as `pressed.md`, not in a shared top-level pressed folder. When a pressed
leaf cites or is cited by other leaves, cross-leaf citation metadata lives next
to the digest as `linked.md`. `.leaf/PROFILE.md` is the repo-local acquired
profile: `leaf init` scaffolds it, completed leaves consolidate working-style
traits into it at ⑩ Retrospect, and `soul` reads it at the start of LEAF
work. A machine-global profile at `~/.config/leaf/profile.md` layers underneath
it for facts that apply to every repo on the machine, such as the user's
working language; `leaf profile` prints the merged view.

`leaf init` adds `/.leaf` to `.git/info/exclude` so local collaboration notes do
not appear in normal `git status` output.

## Commands

```bash
leaf init
leaf new <slug>
leaf fall <slug> --reason <reason>
leaf list [--json]
leaf tree [--plain] [--demo]
leaf review <slug>
leaf profile
leaf checkpoint <slug> --<gate>
leaf doctor [--json]
```

`leaf init` initializes `.leaf/` storage in the current git repository and
scaffolds the machine-global profile at `~/.config/leaf/profile.md` if it does
not exist yet. Both are idempotent: existing files are never overwritten.

`leaf new <slug>` creates a new sprout under `.leaf/01-sprouts/<slug>/`:

```text
.leaf/01-sprouts/my-first-idea/
├── 00-status.md
├── 01-Learn/
│   ├── 01-intent.md
│   ├── 02-unknowns.md
│   └── 02-references/
│       └── README.md
├── 02-Example/
│   ├── 03-criteria.md
│   └── 04-wireframe.md
├── 03-Architect/
│   ├── 05-design.md
│   └── 07-tasks.md
└── 04-Feedback/
```

Slug values must be path-safe ASCII strings using letters, digits, `-`, and
`_`. Existing sprouts are not overwritten.

`leaf fall <slug> --reason <reason>` moves a sprout or leaf to
`.leaf/03-fallen/<slug>/` and writes `fallen reason` plus closure fields into
`00-status.md`. The reason is free text, so an agent or human can use canonical
reasons such as `abandoned`, `superseded`, `parked`, `split`, `invalidated`, or
`completed-not-reference-worthy`, while still recording project-specific detail.

`leaf list` shows the current stage inventory. Non-TTY output uses a deterministic
`STAGE` table; `leaf list --json` outputs top-level `stages`. In an interactive
terminal it opens a browser for navigating, filtering, previewing, and
multi-selecting rows; press `F` to fall the marked sprouts/leaves (or the current
row) — a centered prompt collects one shared reason, `Enter` confirms and `Esc`
cancels.

`leaf tree` renders the current `.leaf/` workspace as a bounded terminal tree:
completed leaves fill the green crown, per-leaf `pressed.md` digests appear as
gold fruit, active sprouts appear in an `active sprouts:` row, and fallen items
stay below the living tree. It emits ANSI color by default even when redirected;
use `leaf tree --plain` for clean text output. `leaf tree --demo` renders the
same tree renderer repeatedly with synthetic 0, 3, 10, 20, 50, and 100 leaf
folders, stacked from small to saturated, without requiring an initialized `.leaf/`
workspace. In an interactive terminal, `leaf tree` uses the current terminal
width up to 112 columns; below 32 columns it falls back to a compact summary
instead of forcing broken tree art.

`leaf review <slug>` opens the same source-faithful review reader for one
work item directly. In non-TTY output it writes the review document as
plain text.

`leaf profile` prints the effective profile: the machine-global
`~/.config/leaf/profile.md` followed by the repo-local `.leaf/PROFILE.md`, each
behind a source marker. On conflict the local layer wins. The global location
honors `LEAF_CONFIG_DIR`, then `$XDG_CONFIG_HOME/leaf`, then `~/.config/leaf`.
Outside a git repository it still prints the global layer.

`leaf checkpoint <slug> --<gate>` copies each existing canonical gate source
next to its original with a UTC `YYMMDD-HHMM` prefix, for example
`260612-1430 03-criteria.md`. Folder-based gate sources such as
`02-Example/04-wireframe/`, `04-Feedback/09-reviews/`, and
`04-Feedback/10-retrospective/` are copied recursively, for example
`260612-1430 04-wireframe/`. Gate flags accept names such as `--criteria` and
numbers such as `--3`; `--gate <gate>` is also accepted.

`leaf doctor` checks whether `.leaf/` is ready for `leaf list` and reports old
layout leftovers, missing status fields, and stage/status mismatches.

## Agent Skills

This repository ships Agent Skills bundled as the `leaf` plugin (see Quick Start):

| Skill | Use it for |
|---|---|
| [`using-leaf`](plugins/leaf/skills/using-leaf/SKILL.md) | Entry/router: the LEAF loop and which leaf skill to use; injected at session start |
| [`learn`](plugins/leaf/skills/learn/SKILL.md) | Capturing and triaging ideas, and running the Learn phase (① Intent, ② Unknowns & Context) on a sprout |
| [`work`](plugins/leaf/skills/work/SKILL.md) | Carrying a sprout after Learn from ③ Example to a shipped result |
| [`polish`](plugins/leaf/skills/polish/SKILL.md) | Polishing LEAF documents into simple, complete current reports |
| [`press`](plugins/leaf/skills/press/SKILL.md) | Pressing a reference-worthy leaf into a citable digest once press is the chosen close-out |
| [`profile`](plugins/leaf/skills/profile/SKILL.md) | Reading and updating the machine-global and repo-local LEAF profiles |
| [`soul`](plugins/leaf/skills/soul/SKILL.md) | Shared conduct, voice, and reporting standard for LEAF reporting and review handoff |

Install the LEAF skills together as a family — they are not independent.
`learn`, `work`, `polish`, and `press` read
`soul` through the sibling path `../soul/SKILL.md`; `learn` and
`work` also read the gate references under `work` through
`../work/references/`. Installing the `leaf` plugin ships the whole family
together, so these cross-skill references resolve.

## Status

`leaf` is currently an early Rust CLI. The current slice initializes repo-local
LEAF storage, scaffolds sprouts, lists stage inventory, diagnoses list readiness,
opens review readers, and moves non-reference-worthy work into fallen.

The crate is not published to crates.io.

## Development

```bash
cargo fmt --all --check
cargo check --locked --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```

Release artifacts are generated by cargo-dist from `dist-workspace.toml`.
The release workflow publishes GitHub Release artifacts and updates
`hoetaek/homebrew-tap` when a SemVer tag points at a commit contained in the
protected `main` release branch.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
