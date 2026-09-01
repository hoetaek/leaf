# Fast-track LEAF

Fast-track은 별도 skill·CLI·세션 모드가 아니라 **현재 요청 단위**의 LEAF
절차 예산이다. `using-leaf`가 direct execution을 먼저 제외한 뒤, 사용자가 fast
track을 명시했고 실제 LEAF 기록이 필요하며 core unknown이 없을 때만 선택한다.

## Unknown 구분

- **bounded unknown** — 답이 달라도 승인할 why / what / wireframe이 바뀌지 않는다.
  필요한 scout나 확인 하나만 실행한다.
- **core unknown** — 답에 따라 triple, 안전 경계, 권한, 공개 계약, 핵심 범위가
  바뀔 수 있다. `route: discovery-heavy`로 바꾸고 두 approval을 `expired`로 만든
  뒤 discovery-heavy LEAF로 간다.

`fast track`이라는 말만으로 자동 진행이나 gate fold가 승인되지는 않는다. Learn의
triple 제안에 `승인 후 자동 진행`과 `적격 ④/⑤/⑦ fold`를 함께 적고 사용자가
승인한 경우에만 autopilot이 그 위임을 소비한다.

## Durable status contract

Fast-track을 선택하면 승인 대기 전 `00-status.md` preamble의 기존 operational
fields와 `## Overview` 사이에 `route: fast-track`과 두 approval의 `not approved`
값을 즉시 기록한다. 묶음 승인 뒤 승인 결과로 덮어쓴다. 승인하지 않은 위임은
생략하지 않는다. `autopilot approval: not approved`이면 `fold approval`도 `not
approved`여야 한다.

```markdown
- route: fast-track | fast-track (expired) | discovery-heavy
- autopilot approval: approved | not approved | expired
- fold approval: eligible ④/⑤/⑦ approved | not approved | expired
```

이 필드들은 요청 단위 route와 위임의 재개 근거다. 대화 기록이나 `fast track`
문구만으로 추론하지 않는다. 기존 status parser는 unknown preamble key를
무시하므로 별도 scaffold나 parser 변경은 필요하지 않다.

`approved`는 **같은 요청을 재개하고 현재 작업이 user-approved locked `what`과
계속 일치할 때만** 유효하다. 새 follow-up이나 scope 변경이 들어오면 routing 전에
route를 `fast-track (expired)`로, 두 approval을 `expired`로 바꾼다. ⑩ close-out에서도
route-appropriate cumulative polish를 먼저 마친 뒤 셋을 같은 방식으로 만료한다.
Exact `route: fast-track`만 현재
활성 route이고 `fast-track (expired)`은 감사용 이력일 뿐 권한을 부여하지 않는다.
이후 요청은 새 묶음 승인을 받아 exact active 값을 다시 기록해야 한다.

실행 중 core unknown이 드러나면 active route를 `discovery-heavy`로 교체하고 두
approval을 `expired`로 바꾼 뒤 승격한다. 이 전환도 대화에만 남기지 않는다.

## 기본 절차 예산

| 절차 | 기본값 | 늘리는 조건 |
|---|---:|---|
| sprout·CLI body | 유지 | — |
| triple 승인 | 1회 묶음 | 항목이 모호하거나 서로 충돌함 |
| scout | 0 | bounded unknown 하나를 확인해야 함 |
| quiz | 0 | 사용자가 triple을 판단할 외부 지식이 필요함 |
| 경계 누적 self-polish | 유지 | — |
| full polish·독립 polish reviewer | 0 | 문서가 길거나 stale·모순·사용자 검토 품질 위험이 있음 |
| live UI | 0 | 사용자가 요청했거나 rendered artifact를 봐야 판단 가능함 |

③ · ⑥ · ⑧ · ⑨ · ⑩은 항상 실행하되 증거에 맞게 깊이를 줄일 수 있다.
④ · ⑤ · ⑦은 `../../work/references/gates.md`의 기존 조건을 만족하고 triple에서
autopilot과 fold를 함께 승인받았을 때만 저장된 fold delegation으로 접는다.
Manual fast-track은 저장된 fold 값을 소비하지 않고 ③ 끝의 canonical interactive
approval을 따른다. 구체적인 이유와 ⑨ audit은 남긴다.

## 기존 규칙의 주인

- route와 적용 조건: `../SKILL.md`
- Learn의 scout·quiz·triple 방식: `../../learn/SKILL.md`와
  `../../learn/references/gate-02-unknowns-context.md`
- gate 실행·fold 조건: `../../work/SKILL.md`와
  `../../work/references/gates.md`
- 자동 진행·fold 위임: `../../autopilot/references/approval-policy.md`
- self-polish와 full polish 선택: `../../polish/SKILL.md`
- live UI 조건: `../../soul/SKILL.md`

Fast-track은 검증이나 안전을 생략하지 않으며 **권한을 추가하지 않는다**. 삭제,
배포, 외부 공유, 비용, credential, security/privacy/legal/permission boundary는
기존 승인을 그대로 요구한다. 실행 중 core unknown 승격은 위 durable status
contract를 따른다. 이 선택은 다음 요청으로 이어지지 않는다.
