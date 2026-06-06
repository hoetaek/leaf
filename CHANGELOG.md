# Changelog

All notable changes to this project will be documented in this file.

This project follows pre-1.0 SemVer. Until the CLI and persisted state model
are stable enough for 1.0, breaking user-facing changes bump the `0.x.0`
minor version instead of moving to `x.0.0`.

## 0.1.1 - 2026-06-06

- Added `leaf --version` support for installed-binary smoke checks.

## 0.1.0 - 2026-06-06

- Added `leaf init` to initialize repo-local `.leaf/` storage and exclude it
  from git via `.git/info/exclude`.
- Added `leaf new <slug>` to scaffold phase-gated idea seeds under
  `.leaf/seeds/`.
- Added cargo-dist release configuration with shell and Homebrew installers
  publishing to `hoetaek/homebrew-tap`.
