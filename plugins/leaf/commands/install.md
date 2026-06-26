---
description: Install or update the leaf CLI for the .leaf/ workspace
---

The user wants the `leaf` CLI installed or updated. Detect the current OS,
run the matching installer, then verify.

1. Check the current version:

   ```bash
   leaf --version
   ```

   If it is `0.12.0` or newer, report that version and stop. If `leaf` is
   missing or older, continue.

2. Determine the current OS. Do not use the current repo, `Cargo.toml`, or
   source-checkout status to choose the installer.

   - On Unix shells, run `uname -s`: `Darwin` means macOS, `Linux` means Linux.
   - On Windows/PowerShell, treat the OS as Windows.
   - If the OS cannot be determined, ask the user before installing.

3. Run the installer for that OS:

   - **macOS** — if Homebrew is available:

     Missing `leaf`:

     ```bash
     brew install hoetaek/tap/leaf
     ```

     Older Homebrew-managed `leaf`:

     ```bash
     brew update && brew upgrade hoetaek/tap/leaf
     ```

     If Homebrew is unavailable or does not manage the current `leaf`, run:

     ```bash
     curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.sh | sh
     ```

   - **Linux** — run the shell installer:

     ```bash
     curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.sh | sh
     ```

   - **Windows** — run the PowerShell installer:

     ```powershell
     powershell -ExecutionPolicy ByPass -c "irm https://github.com/hoetaek/leaf/releases/latest/download/leaf-installer.ps1 | iex"
     ```

4. Verify with `leaf --version` and confirm it is `0.12.0` or newer.
5. Report the installed version. If installation failed, show the failing
   command and error summary, then suggest the next OS-appropriate fallback.
   Do not claim success without version output.
