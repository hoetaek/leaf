---
name: sidecar
description: Use when the user invokes /leaf:sidecar or $leaf:sidecar, or asks to create, refresh, document, or re-verify a leaf SRP "single responsibility" sidecar contract for a code artifact; also when a `srp_sidecar_stale` doctor warning appears, when someone is hand-writing a `*.leaf.toml`, or when a load-bearing file's responsibility risks being copied or broadened.
---

# LEAF Sidecar

A leaf SRP sidecar is one machine-checkable sentence — "this artifact exists to …" —
kept in `<artifact>.leaf.toml` next to a load-bearing file, so future work cannot
quietly broaden what the file is responsible for. This skill walks the user through
authoring a new sidecar or refreshing an existing one. `leaf sidecar` writes the
file and `leaf doctor` validates it; your job is the judgment — is the responsibility
honest, and does this file even deserve a sidecar — and the flow.

## When to use

- The user invokes `/leaf:sidecar <artifact>` (or `$leaf:sidecar <artifact>`).
- A `srp_sidecar_stale` warning appears, or a documented artifact has changed.
- A load-bearing artifact's single responsibility is worth pinning so future edits
  do not broaden it.

## When NOT to use

- Ordinary files whose responsibility no one is likely to copy or broaden — gate
  prose is enough, and a sidecar on every file is just noise.
- Authoring `.leaf/` gate documents, pressing a leaf, or knowledge-graph upkeep —
  those are `work`, `press`, and `tend`.

## Procedure

A sidecar does **not** need `leaf init` or a `.leaf/` workspace — `leaf sidecar new`
ensures its own `*.leaf.toml` git-exclude line. Do not run `leaf init` just to make
a sidecar.

1. **Detect** the path you are on:

   ```bash
   leaf sidecar list
   ```

   If `<artifact>.leaf.toml` already exists → **Refresh** (step 3). Otherwise →
   **Author** (step 2).

2. **Author** a new sidecar:

   1. Read the artifact, then state its responsibility as ONE sentence:
      "This file exists to …".
   2. **Responsibility Statement Test.** If that sentence needs several clauses,
      strained wording, or a list of unrelated `and` / `or` jobs, the artifact is
      mixing responsibilities. Do not paper over it with a tidy sentence: surface
      the mix to the user and recommend splitting or renaming, or agree on the one
      responsibility the file *should* hold. The sidecar records the honest single
      responsibility, never a wish.
   3. Write it:

      ```bash
      leaf sidecar new <artifact> --responsibility "This file exists to …"
      ```

      The artifact must already exist. `status` is always `advisory` (the only
      valid value).
   4. If the responsibility is at real risk of broadening, record the optional
      arrays `does_not_own` / `contracts` / `split_signals` — what stays out, what
      the file must honor, and what should trigger a split. `new` has no flag for
      these, so add them by editing the scaffolded `<artifact>.leaf.toml`. Editing
      a CLI-scaffolded file to add a documented v1 field is fine; only inventing the
      whole contract by hand is the mistake. Skip this for simple cases.
   5. Confirm (step 4).

3. **Refresh** an existing sidecar:

   1. Re-read the artifact and confirm the recorded `responsibility` still matches
      the code. If the file's job actually changed, edit `responsibility` first
      (then re-run the Responsibility Statement Test).
   2. Clear the stale mark:

      ```bash
      leaf sidecar verify <artifact>
      ```

      This re-records `last_verified` to today, which clears `srp_sidecar_stale`.

4. **Confirm and report:**

   ```bash
   leaf sidecar list    # primary check — the artifact shows `fresh`
   leaf doctor          # deeper check — zero srp_sidecar_* findings = sidecar clean
   ```

   Lead with `leaf sidecar list`: `fresh` is positive proof the file parses and
   pairs with its artifact. `leaf doctor` emits `srp_sidecar_*` findings ONLY when
   something is wrong, so none means clean. **Caveat:** in a repo with no `.leaf/`
   workspace, `leaf doctor` also reports `leaf_root_missing` and exits non-zero with
   a "not ready" banner — that is about the workspace, which a sidecar does not need,
   not about your sidecar. Read only the `srp_sidecar_*` lines. The sidecar is
   git-local (kept in `.git/info/exclude`), so there is nothing to commit.

## Common mistakes

| Mistake | Reality |
|---|---|
| Running `leaf init` first | Not needed. Sidecars are independent of the `.leaf/` workspace; `leaf sidecar new` manages its own exclude line. |
| "doctor showed no sidecar finding, so it didn't validate it" | Backwards. `srp_sidecar_*` findings appear only on problems; zero findings means valid. `leaf sidecar list → fresh` is the positive confirmation. |
| Reading `leaf doctor`'s `leaf_root_missing` / non-zero exit as the sidecar failing | That error is about a missing `.leaf/` workspace, which a sidecar does not need. Only `srp_sidecar_*` findings concern the sidecar. |
| A tidy one-liner over a file that does three things | The sidecar then lies. Surface the responsibility mix and propose a split — do not let the sidecar rubber-stamp broadening. |
| `status` set to anything but `advisory` | `advisory` is the only valid value; doctor rejects others (`srp_sidecar_invalid_status`). |
| Inventing the whole contract by hand | Always scaffold with `leaf sidecar new` (it gets the schema, field set, and `<artifact>.leaf.toml` name right). Editing that scaffolded file afterward to add an optional array is fine. |
| Pointing at a not-yet-created file | `new` requires the artifact to exist; create the file first. |

## Quick reference

| Need | Command |
|---|---|
| See every sidecar + freshness | `leaf sidecar list [--json]` |
| Create one | `leaf sidecar new <artifact> --responsibility "…"` |
| Clear stale after re-confirming | `leaf sidecar verify <artifact>` |
| Validate | `leaf doctor` (zero `srp_sidecar_*` = clean) |

Fields in `<artifact>.leaf.toml` (v1): required `schema` (`leaf.srp-sidecar.v1`),
`artifact`, `status` (`advisory`), `last_verified`, `responsibility`; optional string
arrays `does_not_own`, `contracts`, `split_signals`. No other keys — working context
belongs in gate files, not the sidecar.
