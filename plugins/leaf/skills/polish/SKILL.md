---
name: polish
description: |
  Use at each phase boundary (and before user review or gate close) to make the
  cumulative LEAF document — all phases so far read as one connected report — read
  simply and flow naturally, not as draft notes. Preserve source truth while
  removing stale options, process chatter, duplicate reasoning, status drift, or
  doctor-reported legacy layout/status fields. Do not use to create the artifact,
  decide press/fall close-out, or maintain execution artifacts.
---

# LEAF Polish

Polish LEAF documents until they read like a finished report, not a transcript.

The standard is Feynman-like: if the writer cannot explain the essence simply,
they probably do not understand it yet. A good LEAF file keeps the needed why,
what, evidence, decision, risk, and next input. It removes stale options,
discarded branches, duplicate reasoning, template residue, and status chatter
that no longer helps the next agent.

> 완전함이란 더 이상 보탤 것이 없는 상태가 아니라 더 이상 뺄 것이 없는
> 상태를 말한다.

Use that sentence as the quality bar. More words are not higher quality. The
best document is the one that leaves nothing important hidden and nothing
unnecessary in the way.

## Polish Unit — phase 경계의 누적 전체

polish의 기본 단위는 단일 파일이 아니라 **하나의 phase 경계에서 그때까지 쌓인 전체
LEAF 문서**다. phase 경계(Learn·Example·Architect·Feedback의 끝, 그리고 close-out)에
도달하면, 직전까지의 모든 phase를 *하나의 이어지는 보고서*로 읽고 — phase 사이가
자연스럽게 연결되는지, 중복·드리프트·끊김이 없는지를 본다. 이 누적 전체 흐름 점검은
사용자가 요청할 때만 하는 모드가 아니라 **기본 동작**이다. `leaf review <slug>` 출력이
그 누적 전체를 한눈에 보는 입구다.

단일 gate 파일이나 `00-status.md`만 다듬는 국소 청소는 작은 drift를 빠르게 고칠 때 쓰는
**보조 모드**로 남는다 — 기본을 대체하지 않는다.

## Phase 마커 제거 — polish됨의 유일한 신호

한 phase의 경계 polish를 마치면, 그 phase 첫 게이트 파일에 scaffold가 심어둔 미polish 마커
줄(`<!-- leaf:polish-pending -->`)을 **제거한다.** 이 토큰의 유무가 `leaf next`와 `leaf doctor`가
읽는, 그 phase가 polish됐다는 **유일한 신호**다:

- 토큰이 남아 있으면 `leaf next`는 다음 phase로 넘어가지 않고 멈칫하며 polish를 요구하고,
  `leaf doctor`는 그 phase를 `boundary_unpolished`로 표시한다.
- 따라서 마커 제거는 polish의 마지막 단계다 — 다듬지 않고 토큰만 지우는 것은 이 신호의
  의미를 비우는 행위이니 하지 않는다(누적 전체를 실제로 한 보고서로 만든 뒤 지운다).

## Boundary

- Work on LEAF document quality only.
- Preserve source truth. Do not delete facts, evidence, decisions, caveats, or
  user wording that affects meaning.
- Do not execute artifact work, create wt artifacts, decide press/fall close-out,
  or maintain external systems.
- Run `leaf doctor` before and after substantial cleanup. If it reports a
  workspace issue, fix the diagnosed issue before trusting inventory output.
- Keep parser-facing fields, paths, command names, code identifiers, and quoted
  source text unchanged unless the cleanup explicitly targets that field.

## First Read

```bash
git status --short --branch
leaf doctor
leaf review <slug>
```

Then choose the surface. The default is the cumulative whole; single-file passes
are the auxiliary small-drift mode:

- **default — the whole `leaf review <slug>` report** at a phase boundary or
  close-out: read all phases so far as one connected report and check it flows
  end to end;
- one gate file before it is shown to the user or closed (small, local drift);
- `00-status.md` when overview, current gate, or next action drifted.

Before editing a gate document, run `leaf checkpoint <slug> --<gate>`.

If `leaf doctor` reports layout findings (`old_stage_dir_present`,
`pressed_stage_dir_present`, `legacy_state_field`, `legacy_fall_reason_field`),
run **Migrate** first, even when the user asked for something else.

## Migrate

This skill is the migration operator that `leaf doctor` routes old-layout
findings to.

| Finding | Repair |
|---------|--------|
| `old_stage_dir_present` | Map the old dir to its own canonical stage dir: `seeds`/`01-seeds` → `01-sprouts`, `leaves` → `02-leaves`, `fallen` → `03-fallen`. If the canonical dir is missing or empty, rename the old dir to the canonical name. If both hold items, move item folders one by one into the canonical dir; on a slug collision, stop and ask. |
| `pressed_stage_dir_present` (top-level `.leaf/04-pressed/` or `.leaf/pressed/`) | Move each `{slug}.md` digest into the matching item folder as `pressed.md`, then remove the emptied pressed dir — doctor warns as long as the dir exists. If a digest has no matching folder, report it, leave it in place, and tell the user the warning will persist until it is resolved. |
| `legacy_state_field` | Rewrite the status `state` field as the canonical `stage` field and translate the value: `seed`/`active` → `sprout`; `complete`/`completed` → `leaf`; `fallen` stays `fallen`. For an unrecognized value, use the stage matching the directory the item lives in; if that is ambiguous, stop and ask. |
| `legacy_fall_reason_field` | Rewrite `fall reason` as `fallen reason`. |

- Never merge folders by overwriting; a collision means stop.
- Migration changes locations and field names, never meaning: do not rewrite
  prose while migrating.
- Re-run `leaf doctor` after migrating and confirm the findings are gone.

## Polish Pass

Read the whole target before editing. Rewrite only after you can answer:

- 이 문서의 현재 결론은 무엇인가?
- 왜 이 작업이 필요한가?
- 무엇이 실제로 결정되었고 무엇이 폐기되었는가?
- 어떤 evidence가 결론을 지탱하는가?
- 다음 agent가 소비해야 할 입력은 무엇인가?
- 사용자가 검토해야 하는 결정이나 질문은 무엇인가?
- 남은 질문은 왜 이 gate를 막지 않는가, 아니면 무엇을 막는가?

Then edit toward the current report:

- lead with the current conclusion;
- keep the strongest evidence close to the claim it supports;
- collapse old alternatives into one sentence when only the decision matters;
- remove stale TODOs, dead options, repeated caveats, and template headings;
- present user review needs as decision points, not process history;
- state assumptions and unresolved limits plainly;
- keep Korean prose when the user is working in Korean.

For prose — not tables, code, parser fields, paths, or quoted source — apply
Orwell's plain-prose rules:

- name the actor in the active voice; do not bury who acted in a passive or a
  nominalization ("X로 닫힘" → "사용자가 X를 닫았다");
- call one thing by one name; never two names for the same concept;
- replace dead metaphor, cliché, and the long or jargon word with the short
  plain one;
- cut the hedge that blurs a commitment ("거의 그대로", "mostly") — if it is
  true, assert it;
- never dress an empty or unpleasant truth in euphemism.

Extra checks: drift, surface, archive, fallen.

- drift: `00-status.md` must match the existing gate files.
- surface: separate canonical report, checkpoint history, pressed digest, and
  fallen closure.
- archive: checkpoint files are history; do not rewrite them unless asked.
- fallen: fallen items should lead with fallen reason, closure, lesson, and
  limits.

## Subagent Review

Before calling a polish complete, delegate an independent review to a subagent.
The reviewer judges only document quality, not implementation truth. Give it the
target file or `leaf review` output and this rubric:

```text
Judge whether this LEAF document is a simple complete current report.
Use the standard: "완전함이란 더 이상 보탤 것이 없는 상태가 아니라 더 이상 뺄 것이 없는 상태를 말한다."

Return:
- keep: the sentences or sections that carry the essence
- cut: stale, duplicated, decorative, or non-load-bearing text
- unclear: why/what/evidence/decision gaps
- drift/surface: status/file mismatch, archive confusion, or fallen closure gaps
- prose: passive voice hiding the actor, one concept under two names, dead
  metaphor, jargon or long words, euphemism, hedging that blurs a commitment
- verdict: pass or revise
```

For the prose dimension, check whether the runtime exposes the `orwell`
(blunt plain-prose review) skill: inspect the available skills list, and if it
is listed, read its `SKILL.md` and use it as the prose lens. It is an
accelerator, not a blocker — if it is not exposed, apply the prose rules from
the Polish Pass directly. Use its harsh tone only if the user asks for it.

## Pass Criteria

A polished LEAF document passes when:

- a new agent can state the why and what after one read;
- the current decision is easier to find than the history of how it was reached;
- current stage, gate, decision, and next action match the existing artifacts;
- every remaining section either carries meaning, evidence, a risk, or a next
  input;
- checkpoint/archive and fallen closure do not confuse the current report;
- no removed text changes the source truth;
- `leaf doctor` still reports the workspace as usable, or remaining findings are
  named precisely.

## Report

Report:

- target files polished;
- what was removed or compressed;
- what source truth was preserved;
- subagent reviewer verdict;
- `leaf doctor` result;
- confirmation that no `.wt/` or execution artifacts were created.
