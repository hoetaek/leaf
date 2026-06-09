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
seed (lock ① Intent, resolve ② Unknowns & Context); use `leaf-work` to carry a
promoted leaf from ③ Example through to a shipped result. When Learn is complete,
move the work into active leaf storage:

```bash
leaf promote my-first-idea
```

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

The detailed gate model lives in [`skills/leaf-work`](skills/leaf-work/SKILL.md).

## Concepts

`leaf` keeps work inside a repo-local `.leaf/` workspace:

```text
.leaf/
├── seeds/
├── leaves/
├── fallen/
└── pressed/
```

`seeds/` are rough ideas and exploratory Learn-phase starts. `leaves/` are
committed active LEAF work from Example onward. `fallen/` preserves active
leaves after they close. `pressed/` stores citable Markdown digests of important
LEAF work, such as intent, method, what was done, limits, and lessons learned.

`leaf init` adds `/.leaf` to `.git/info/exclude` so local collaboration notes do
not appear in normal `git status` output.

## Commands

```bash
leaf init
leaf new <slug>
leaf promote <slug>
leaf fall <slug> --reason <reason>
```

`leaf init` initializes `.leaf/` storage in the current git repository.

`leaf new <slug>` creates a new seed under `.leaf/01-seeds/<slug>/`:

```text
.leaf/01-seeds/my-first-idea/
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
`_`. Existing seeds are not overwritten.

`leaf promote <slug>` moves a seed from `.leaf/01-seeds/<slug>/` to
`.leaf/02-leaves/<slug>/` once Learn is complete and Example should start from
active leaf storage. It updates `00-status.md` to `state: active` and
`current phase: Example`, while preserving the previous seed status.

`leaf fall <slug> --reason <reason>` moves an active leaf from
`.leaf/02-leaves/<slug>/` to `.leaf/03-fallen/<slug>/` and writes flexible closure
fields into `00-status.md`. The reason is free text, so an agent or human can
use canonical reasons such as `completed`, `abandoned`, `superseded`, `parked`,
`split`, or `invalidated`, while still preserving project-specific detail.

## Agent Skills

This repository ships Agent Skills:

| Skill | Use it for |
|---|---|
| [`leaf-idea`](skills/leaf-idea/SKILL.md) | Capturing and triaging ideas, and running the Learn phase (① Intent, ② Unknowns & Context) on a seed |
| [`leaf-work`](skills/leaf-work/SKILL.md) | Carrying a promoted leaf from ③ Example to a shipped result |
| [`leaf-press`](skills/leaf-press/SKILL.md) | Creating citable Markdown digests from LEAF work |
| [`leaf-fall`](skills/leaf-fall/SKILL.md) | Closing active leaves into the fallen archive |
| [`leaf-soul`](skills/leaf-soul/SKILL.md) | Shared conduct, voice, and reporting standard for LEAF reporting and review handoff |

Install the LEAF skills together as a family — they are not independent.
`leaf-idea` reads `leaf-soul` and the gate references under `leaf-work`, and
`leaf-work` reads `leaf-soul`, all through sibling paths (`../leaf-soul/SKILL.md`,
`../leaf-work/references/`). The Quick Start command above installs the whole
family; installing a single skill with `--skill` would leave those cross-skill
references broken.

## Status

`leaf` is currently an early Rust CLI. The current slice initializes repo-local
LEAF storage, scaffolds idea seeds, promotes seeds into active leaves after
Learn, and archives active leaves when they close.

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
