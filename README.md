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

Install the Agent Skills:

```bash
npx skills@latest add https://github.com/hoetaek/leaf
```

For a global skills install:

```bash
npx skills@latest add https://github.com/hoetaek/leaf -g
```

For a global Claude Code skills install:

```bash
npx skills@latest add https://github.com/hoetaek/leaf -g -a claude-code
```

Install the `leaf` CLI that gives those skills a repo-local body. Homebrew is
the recommended install path:

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

Ask your agent to use `leaf-idea` to capture an idea and run the Learn phase on a
sprout (lock ① Intent, resolve ② Unknowns & Context); use `leaf-work` to carry a
sprout from ③ Example through to a shipped result. Learn and post-Learn work stay
in the same sprout until completion.

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
[`skills/leaf-idea`](skills/leaf-idea/SKILL.md); Example onward lives in
[`skills/leaf-work`](skills/leaf-work/SKILL.md).

## Concepts

`leaf` keeps work inside a repo-local `.leaf/` workspace:

```text
.leaf/
├── 01-sprouts/
├── 02-leaves/
└── 03-fallen/
```

`01-sprouts/` holds incomplete work: Learn, Example, Architect, execution, and
review. `02-leaves/` holds completed, reference-worthy LEAF folders.
`03-fallen/` holds discarded or archived work, including completed work that is
not useful enough to keep as a reference. Pressed digests live inside the source
leaf as `pressed.md`, not in a shared top-level pressed folder. When a pressed
leaf cites or is cited by other leaves, cross-leaf citation metadata lives next
to the digest as `linked.md`. `.leaf/PROFILE.md` is the repo-local acquired
profile: `leaf init` scaffolds it, completed leaves consolidate working-style
traits into it at ⑩ Retrospect, and `leaf-soul` reads it at the start of LEAF
work.

`leaf init` adds `/.leaf` to `.git/info/exclude` so local collaboration notes do
not appear in normal `git status` output.

## Commands

```bash
leaf init
leaf new <slug>
leaf fall <slug> --reason <reason>
leaf list [--json]
leaf review <slug>
leaf doctor [--json]
```

`leaf init` initializes `.leaf/` storage in the current git repository.

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
`STAGE` table; `leaf list --json` outputs top-level `stages`.

`leaf review <slug>` opens the same source-faithful review reader for one
leaf-work item directly. In non-TTY output it writes the review document as
plain text.

`leaf doctor` checks whether `.leaf/` is ready for `leaf list` and reports old
layout leftovers, missing status fields, and stage/status mismatches.

## Agent Skills

This repository ships Agent Skills:

| Skill | Use it for |
|---|---|
| [`leaf-idea`](skills/leaf-idea/SKILL.md) | Capturing and triaging ideas, and running the Learn phase (① Intent, ② Unknowns & Context) on a sprout |
| [`leaf-work`](skills/leaf-work/SKILL.md) | Carrying a sprout after Learn from ③ Example to a shipped result |
| [`leaf-clean`](skills/leaf-clean/SKILL.md) | Tending the `.leaf/` workspace: pressing citable digests, moving non-reference-worthy work into fallen, and migrating old layouts reported by `leaf doctor` |
| [`leaf-soul`](skills/leaf-soul/SKILL.md) | Shared conduct, voice, and reporting standard for LEAF reporting and review handoff |

Install the LEAF skills together as a family — they are not independent.
`leaf-idea`, `leaf-work`, and `leaf-clean` read `leaf-soul` through
the sibling path `../leaf-soul/SKILL.md`; `leaf-idea` and `leaf-work` also read
the gate references under `leaf-work` through `../leaf-work/references/`. The
Quick Start command above installs the whole family; installing a single skill
with `--skill` would leave those cross-skill references broken.

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
