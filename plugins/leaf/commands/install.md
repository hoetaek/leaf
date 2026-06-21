---
description: Install or update the leaf CLI for the .leaf/ workspace
---

The user wants the `leaf` CLI installed. Do it now, then verify.

1. Detect the right install method:
   - If the current repo is the leaf source (a `Cargo.toml` at the repo root
     whose `[package]` has `name = "leaf"`), run: `cargo install --path .`
   - Otherwise, prefer Homebrew: `brew install hoetaek/tap/leaf`
   - If brew is unavailable, fall back to:
     `cargo install --git https://github.com/hoetaek/leaf`
2. Run the chosen command.
3. Verify with `leaf --version` and confirm it is ≥ 0.8.0.
4. Report the installed version. If it failed, show the error and suggest the
   next fallback — do not claim success without the version output.
