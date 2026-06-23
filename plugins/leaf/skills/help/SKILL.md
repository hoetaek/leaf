---
name: help
description: >
  Quick-reference card for LEAF phases, skills, CLI commands, and update flow.
  One-shot display, not a persistent mode. Trigger: /leaf:help, $leaf:help,
  "leaf help", "LEAF commands", "how do I use leaf".
---

# LEAF Help

Display this reference card when invoked. One-shot, do NOT change phase,
write gate files, run `leaf next`, or persist anything.

## Loop

| Phase | What it proves | Gates |
|-------|----------------|-------|
| **Learn** | What the work needs, learned rather than guessed. | ① Intent · ② Unknowns & Context |
| **Example** | One cheap instance is right before scaling. | ③ Criteria · ④ Wireframe |
| **Architect** | The instance can become a shippable generator. | ⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact |
| **Feedback** | The result holds, and the lesson is settled. | ⑨ Review · ⑩ Retrospect |

Core rule: leaf before tree. Validate one inspectable instance before growing
the full artifact.

## Skills

| Skill | Trigger | What it does |
|-------|---------|--------------|
| **using-leaf** | automatic session context | Routes LEAF work to the right skill. |
| **soul** | LEAF reporting or handoff | Plain explanation, fact/guess boundaries, user-language prose. |
| **learn** | `$leaf:learn`, ideas, unclear work | Captures intent and unknowns before execution. |
| **work** | approved Learn work | Carries Example → Architect → Feedback. |
| **split** | `$leaf:split`, split this work | Decides whether one item should become multiple leaves. |
| **autopilot** | `$leaf:autopilot` | Continues after the why/what/wireframe triple is reviewed. |
| **polish** | phase boundaries | Makes cumulative LEAF files read as one connected report. |
| **press** | `$leaf:press` | Turns reference-worthy work into a citable digest. |
| **profile** | LEAF preferences | Reads or updates global and repo-local LEAF profile entries. |
| **install** | `$leaf:install` | Installs or updates the `leaf` CLI, then verifies `leaf --version`. |
| **help** | `$leaf:help`, `/leaf:help` | This card. |

Codex exposes skills as `$leaf:<skill>`. Claude-style hosts may expose the same
entries as `/leaf:<skill>` when the host supports slash commands.

## CLI

| Command | Use |
|---------|-----|
| `leaf init` | Create the repo-local `.leaf/` workspace. |
| `leaf profile` | Show effective global + repo-local profile. |
| `leaf new <slug>` | Start a sprout. |
| `leaf next <slug>` | Cross a polished phase boundary. |
| `leaf checkpoint <slug> --<gate>` | Snapshot gate files before polishing. |
| `leaf doctor` | Check workspace and boundary health. |
| `leaf fall <slug> --reason "<reason>"` | Stop carrying a leaf or sprout. |

## Install Or Update

Use `$leaf:install` first when `leaf` is missing or too old. It chooses the
OS-appropriate installer and verifies with:

```bash
leaf --version
```

## More

Full docs + releases: https://github.com/hoetaek/leaf
