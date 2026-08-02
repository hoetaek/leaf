---
name: using-leaf
description: Use when starting LEAF work or deciding which leaf skill applies — establishes the LEAF loop, routes to the right gate skill, and points conduct at soul. Injected at session start.
---

# Using LEAF

**Leaf before tree.** Validate one cheap, inspectable instance before scaling —
growing the whole artifact up front produces confident-looking slop.

LEAF closes four kinds of uncertainty in order:

| Phase | What it makes you able to do |
|---|---|
| **Learn** | Judge what the work needs (① Intent · ② Unknowns & Context) |
| **Example** | Prove one cheap instance before scaling (③ Criteria · ④ Wireframe) |
| **Architect** | Generalize it into a shippable generator (⑤ Design · ⑥ Critic · ⑦ Tasks · ⑧ Artifact) |
| **Feedback** | Confirm it holds, then settle what was established (⑨ Review · ⑩ Retrospect) |

## Invoke before you work

<EXTREMELY-IMPORTANT>
If there is even a 1% chance a leaf skill applies, invoke it BEFORE you respond
or act. Knowing the loop is not running it — only the skill carries the gates.
</EXTREMELY-IMPORTANT>

It is LEAF work to produce or substantially revise a document; capture, triage,
or park an idea; research, benchmark, or map a topic; build or prototype
something large enough to need design; decide whether one work item should split
into several (→ `split`); or carry any substantial knowledge/build work in this
repo. It is **not** LEAF work when the reply is a sentence or two, a trivial
edit, or a direct lookup — then just answer.

## Execution-ready 구현

구현을 요청받았다고 항상 Learn 문서부터 쓰지 않는다. 아래 조건을 **모두** 짧은
저장소·요구 확인으로 확인할 수 있으면 execution-ready다.

- 재현 또는 현재 상태를 관찰할 수 있다.
- 수정 후 성공 조건을 관찰할 수 있다.
- 범위와 제외 범위를 구분할 수 있다.
- 작고 되돌릴 수 있는 첫 실험을 정할 수 있다.
- 데이터·보안·공개 계약·대규모 구조를 좌우하는 미결정 사항이 없다.

execution-ready에서는 기존 요구와 저장소 상태를 한 번 확인한 뒤, 가능한 가장
작은 **최초 실행 증거**(실패 테스트, 재현, 측정, 최소 prototype)를 먼저 만든다.
그 뒤 작은 구현·검증을 반복하고, 실제로 생긴 결정·위험·부채만 기록한다. 최초
실행 증거 전에는 phase gate 파일, 누적 polish, 독립 문서 검토, live UI가
선행조건이 아니다.

하나라도 충족하지 않거나 결과물이 문서 자체·리서치·설계라면 discovery-heavy
경로다. 기존 `learn` → `work`와 hard stop을 그대로 사용한다. 실행 중 보안,
권한, 비가역성, 외부 공유·비용·배포, 공개 계약, 대규모 구조의 미결정이 드러나면
즉시 discovery-heavy로 돌아간다.

## Which skill to use

| Skill | Use it for |
|---|---|
| `soul` | **First, always.** Conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn` | Capture/triage an idea and run Learn (① Intent, ② Unknowns & Context) |
| `work` execution-first lane | execution-ready 구현에서 최초 실행 증거를 먼저 만들고 사후에 필요한 gate 기록을 남긴다 |
| `split` | Decide whether/how to split one work item into separate leaves |
| `autopilot` | Run the gates automatically after the human-reviewed why/what/wireframe triple |
| `work` | Carry a sprout from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩ |
| `polish` | Make the cumulative document read as one connected report at each phase boundary |
| `press` | Press a reference-worthy leaf into a citable digest |
| `tend` | Sweep the pressed knowledge graph and reconcile drift with current code (banner/supersede) |
| `profile` | Read/update the machine-global and repo-local LEAF profiles |

Process skills first (decide *how*), then domain skills.

## Ending a leaf

After ⑩, `polish` the cumulative whole, then decide the end and let the user
confirm:

- **keep** — useful but not citable; note it in `00-status.md`.
- **press** — reference-worthy (reusable decision, pattern, lesson); invoke `press`.
- **fall** — stop carrying it: `leaf fall <slug> --reason "<abandoned|superseded|parked|split|invalidated|archived|completed-not-reference-worthy>"`.

Don't keep or press just because effort was spent.

## The CLI is the body

The `leaf` CLI gives the workflow a repo-local `.leaf/` body (`leaf init`,
`leaf new`, `leaf next`, `leaf doctor`); requires `leaf` ≥ 0.12.0.
discovery-heavy 작업과 최초 실행 증거 뒤의 LEAF 기록은 이 몸을 사용한다.
execution-ready 작업은 증거를 만들기 위해 scaffold를 먼저 만들 필요가 없다.
`leaf next` crosses a formal phase boundary, pausing (멈칫) if the phase you are
leaving still carries its `<!-- leaf:polish-pending -->` marker — polish removes
it. If `leaf` is not on PATH,
tell the user to run the install entry (`$leaf:install` in Codex, `/leaf:install`
in Claude) before creating or advancing LEAF records. An execution-ready first
evidence command may proceed without the CLI because it does not use the CLI.

For early/idea work start with `learn`; to build a sprout that passed Learn use `work`.
