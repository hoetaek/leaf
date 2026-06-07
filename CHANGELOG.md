# Changelog

All notable changes to this project will be documented in this file.

This project follows pre-1.0 SemVer. Until the CLI and persisted state model
are stable enough for 1.0, breaking user-facing changes bump the `0.x.0`
minor version instead of moving to `x.0.0`.

## Unreleased

- Added `leaf promote <slug>` to move Learn-complete seeds into active leaves
  before Example work starts.
- Updated LEAF skill guidance so seeds are for rough ideas and Learn-phase work,
  while active leaves carry Example and later phases.

## 0.1.3 - 2026-06-07

- Reworked the README around the LEAF-first positioning, Agent Skills install
  path, LEAF loop, repo-local concepts, command reference, and project status.

## 0.1.2 - 2026-06-07

- Added LEAF agent skills for idea capture, work planning, pressing, and
  falling archived work.
- Added `leaf fall <slug> --reason <reason>` to move active leaves into the
  fallen archive with closure metadata.
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
