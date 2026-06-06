# leaf

Domain-neutral human-agent collaboration CLI.

`leaf` keeps idea-to-delivery knowledge work inside the repository it belongs
to. It bootstraps a repo-local `.leaf/` workspace (kept out of git via
`.git/info/exclude`) and scaffolds structured idea seeds that walk an idea
through Learn, Example, Architect, and Feedback phases.

- `leaf init` initializes `.leaf/` storage in the current git repository.
- `leaf new <slug>` creates a new idea seed under `.leaf/seeds/<slug>/` with
  the phase-gated file layout.

## Install

Homebrew is the recommended install path:

```bash
brew install hoetaek/tap/leaf
leaf --version
```

Update with:

```bash
brew update
brew upgrade hoetaek/tap/leaf
```

Install from the latest GitHub Release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.sh | sh
```

Install from source:

```bash
cargo install --git https://github.com/hoetaek/leaf
```

The crate is not published to crates.io.

## Usage

```bash
cd your-git-repo
leaf init
leaf new my-first-idea
```

`leaf init` creates `.leaf/seeds/` and `.leaf/leaves/` and adds `/.leaf` to
`.git/info/exclude` so the workspace never shows up in `git status`.

`leaf new <slug>` scaffolds a seed with a status dashboard and phase files:

```text
.leaf/seeds/my-first-idea/
├── 00-status.md
├── 01-Learn/
│   ├── 01-intent.md
│   ├── 02-unknowns.md
│   └── 02-references/
├── 02-Example/
│   ├── 03-criteria.md
│   └── 04-wireframe.md
├── 03-Architect/
│   ├── 05-design.md
│   └── 07-tasks.md
└── 04-Feedback/
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
