---
description: Install or update the leaf CLI for the .leaf/ workspace
---

The user wants the `leaf` CLI installed. Do it now, then verify.

1. If the current repo is the leaf source (a `Cargo.toml` at the repo root
   whose `[package]` has `name = "leaf"`), run `cargo install --path .` and
   skip to step 3.
2. Otherwise pick by operating system and run it:
   - **macOS / Linux** — prefer Homebrew: `brew install hoetaek/tap/leaf`.
     If brew is unavailable, fall back to
     `cargo install --git https://github.com/hoetaek/leaf`.
   - **Windows** — run the PowerShell installer:
     `powershell -ExecutionPolicy ByPass -c "irm https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.ps1 | iex"`.
     If it fails and Rust is present, fall back to
     `cargo install --git https://github.com/hoetaek/leaf`.
3. Verify with `leaf --version` and confirm it is ≥ 0.8.0.
4. Report the installed version. If it failed, show the error and suggest the
   next fallback — do not claim success without the version output.
