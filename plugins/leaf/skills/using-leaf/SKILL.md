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
Route the request before invoking another LEAF skill. Invoke LEAF only when the
work matches a positive case below and no direct-work exclusion applies.
Uncertainty about routing is not by itself a reason to start the lifecycle.
</EXTREMELY-IMPORTANT>

It is LEAF work to produce or substantially revise a document; capture, triage,
or park an idea; research, benchmark, or map a topic; build or prototype
something large enough to need design; decide whether one work item should split
into several (→ `split`); or carry any substantial knowledge/build work in this
repo. It is **not** LEAF work when the reply is a sentence or two, a trivial
edit, a direct lookup, or **bounded maintenance** whose current state, desired
state, scope, and verification are already observable — then work directly.
Merge-conflict resolution, dependency updates, localized bug fixes, and similar
maintenance stay direct unless they expose a durable unresolved decision.
**작업 크기만으로 LEAF를 시작하지 않는다.**

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
그 뒤 작은 구현·검증을 반복하고, 실제로 생긴 결정·위험·부채만 기존 issue, PR,
commit, 또는 최종 handoff처럼 가장 가까운 전달 표면에 기록한다. 보존할 항목이
없으면 최종 검증 뒤 **`.leaf/` 기록 없이 종료**한다. 최초 실행 증거 전후 모두
phase gate 파일, 누적 polish, 독립 문서 검토, live UI는 기본 경로가 아니다.

execution-ready는 일반 LEAF lifecycle의 축약판이 아니라 direct execution
경로다. 완료된 구현을 사후 설명하기 위해 ③–⑩을 만들지 않는다. 실행 중 미결정
사항이 생겨 추가 발견·설계가 필요하면 `learn`부터 discovery-heavy lifecycle로
승격한다. 사용자가 durable LEAF 기록 자체를 명시적으로 요청하면 먼저 위
execution-ready 조건을 판정한 뒤, 아래 fast-track 조건을 만족하면 fast-track
LEAF로, 아니면 discovery-heavy lifecycle로 간다.

## Fast-track LEAF

direct execution을 먼저 제외한 뒤, 사용자가 **현재 요청에 fast track을 명시**했고
실제 LEAF lifecycle이 필요하며 why / what / wireframe·안전·권한을 바꿀 core
unknown이 없으면 요청 단위 fast-track LEAF를 쓴다. `soul` 다음 `learn`으로
sprout을 만들거나 이어가되, `references/fast-track.md`의 절차 예산을 적용한다.

`fast track`이라는 말은 route 선호일 뿐 자동 진행·fold 승인이 아니다. Learn이
triple과 함께 이후 autopilot 및 적격 ④/⑤/⑦ fold를 제안하고 사용자가 승인해야
그 위임이 생긴다. 승인 뒤 `00-status.md` preamble에 `route`, `autopilot approval`,
`fold approval`을 명시적으로 기록한다. core unknown이 있거나 fast track을
명시하지 않았으면 기존 discovery-heavy `learn` → `work`와 hard stop을 그대로
사용한다.

기존 fast-track을 같은 요청에서 재개할 때도 작업이 locked `what`과 일치해야 한다.
새 follow-up이나 scope 변경이면 routing 전에 기존 autopilot/fold approval을
`expired`로 바꾸고 이 요청을 새로 route한다.

## Which skill to use

| Skill | Use it for |
|---|---|
| no LEAF skill | bounded maintenance와 execution-ready 구현 — direct execution으로 완료 |
| `soul` | **First for actual LEAF work.** Conduct: plain explanation, fact-vs-guess, user-language, review handoff |
| `learn` | Capture/triage an idea and run fast-track 또는 discovery-heavy Learn |
| `split` | Decide whether/how to split one work item into separate leaves |
| `autopilot` | Run the gates automatically after the human-reviewed why/what/wireframe triple |
| `work` | Carry a sprout from ③ Example through a shipped ⑧ Artifact, then ⑨/⑩ |
| `polish` | Make the cumulative document read as one connected report at each phase boundary |
| `press` | Press a reference-worthy leaf into a citable digest |
| `tend` | Sweep the pressed knowledge graph and reconcile drift with current code (banner/supersede) |
| `profile` | Read/update the machine-global and repo-local LEAF profiles |

Process skills first (decide *how*), then domain skills.

## Ending a leaf

After ⑩, expire any fast-track `autopilot approval` and `fold approval`,
`polish` the cumulative whole, then decide the end and let the user confirm:

- **keep** — useful but not citable; note it in `00-status.md`.
- **press** — reference-worthy (reusable decision, pattern, lesson); invoke `press`.
- **fall** — stop carrying it: `leaf fall <slug> --reason "<abandoned|superseded|parked|split|invalidated|archived|completed-not-reference-worthy>"`.

Don't keep or press just because effort was spent.

## The CLI is the body

The `leaf` CLI gives discovery-heavy work a repo-local `.leaf/` body (`leaf
init`, `leaf new`, `leaf next`, `leaf doctor`); requires `leaf` ≥ 0.12.0.
execution-ready direct work does not need the CLI. If it escalates, start the
normal lifecycle at Learn; do not synthesize a scaffold after implementation.
`leaf next` crosses a formal phase boundary, pausing (멈칫) if the phase you are
leaving still carries its `<!-- leaf:polish-pending -->` marker — polish removes
it. If `leaf` is not on PATH,
tell the user to run the install entry (`$leaf:install` in Codex, `/leaf:install`
in Claude) before creating or advancing LEAF records.

For execution-ready work, exit LEAF routing and implement directly. For
early/idea work start with `learn`; to build a sprout that passed Learn use
`work`.
