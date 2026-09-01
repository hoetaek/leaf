# Execution-ready regression scenarios

## Routing precedence

Evaluate routes in this order and stop at the first match:

1. direct execution — execution-ready이고 durable LEAF record를 명시적으로
   요청하지 않았을 때만 match
2. fast-track LEAF
3. discovery-heavy LEAF

## Bounded maintenance routing

### Given

- merge conflict, dependency update, or localized regression fix처럼 현재 상태와 완료 조건이 관찰 가능하다.
- 보존해야 할 설계 결정이나 미해결 위험이 없다.
- durable LEAF record를 명시적으로 요청하지 않았다.

### Expected trace

1. bounded maintenance로 분류하고 LEAF를 시작하지 않는다.
2. 저장소 상태를 확인하고 수정·검증한다.
3. 결과와 검증 증거를 최종 handoff에 남긴다.

### LEAF operation budget

- leaf scaffold: 0
- phase transitions: 0
- cumulative polish: 0
- independent document reviews: 0
- live UI opens: 0

---

## Execution-ready implementation

## Given

- 기존 UI 회귀를 재현할 수 있다.
- 수정 뒤 성공 조건을 테스트로 관찰할 수 있다.
- 수정 범위와 제외 범위가 정해져 있다.
- 첫 행동은 작고 되돌릴 수 있는 실패 회귀 테스트다.
- 데이터·보안·권한·공개 계약·대규모 구조에 영향을 주는 미결정 사항이 없다.
- durable LEAF record를 명시적으로 요청하지 않았다.

## Expected trace

1. 기존 요구와 저장소 상태를 한 번 확인한다.
2. 문서 gate보다 먼저 실패 회귀 테스트를 실행해 최초 실행 증거를 만든다.
3. 작은 구현과 검증을 반복한다.
4. 구현 중 실제로 생긴 결정·위험·부채만 압축 기록한다.
5. 최종 검증과 필요한 review/retrospect를 수행한다.

## Fast terminal after evidence

보존할 결정·위험·부채가 없으면 최종 검증과 handoff로 종료한다. 최초 실행 증거 뒤에도
이 작업만을 위해 phase gate, scaffold, checkpoint, cumulative polish,
독립 문서 검토, live UI를 만들거나 실행하지 않는다.

## Before the first execution evidence, do not require

- phase gate 파일 작성
- 누적 polish
- 독립 문서 검토
- live UI 열기

## Safety boundary

보안·권한·비가역 변경·외부 공유·비용·배포·공개 계약·대규모 구조 미결정이 드러나면
execution-ready 경로를 멈추고 기존 Learn/Work hard stop으로 돌아간다.

---

## Discovery-heavy escalation

실행 중 보안·공개 계약·대규모 구조를 좌우하는 미결정 사항이 드러나면 direct
execution을 멈추고 Learn부터 일반 LEAF lifecycle을 시작한다. 이 경우에만
phase gate, polish, review UI가 적용된다.

---

## Fast-track LEAF lifecycle

### Given

- 사용자가 이 요청에 fast track을 명시했다.
- 결과물이나 기록 특성상 LEAF lifecycle 자체는 필요하다.
- why / what / wireframe, 안전, 권한을 바꿀 core unknown은 없다.
- triple에 이후 자동 진행과 ④/⑤/⑦의 적격 fold 승인을 함께 제안할 수 있다.

### Expected trace

1. direct execution 조건을 먼저 확인하고 제외한다.
2. 요청 단위 fast-track LEAF로 분류하고 승인 전에 다음 상태를 기록한다.

   - `route: fast-track`
   - `autopilot approval: not approved`
   - `fold approval: not approved`

3. 승인 전에 중단되면 locked `what` 없이 이 상태로 Learn만 재개한다. 이 상태는
   fast-track procedure budget을 유지하지만 autopilot/fold delegation은 주지 않는다.
4. why / what / wireframe triple을 한 묶음으로 제안하고 사용자의 승인을 받는다.
5. 승인 결과로 `00-status.md` preamble을 덮어쓴다.

   - `route: fast-track`
   - `autopilot approval: approved`
   - `fold approval: eligible ④/⑤/⑦ approved`

6. 승인된 절차 예산 안에서 gate를 실행하거나 기존 fold 규칙으로 접는다.
7. ⑧ 실행, ⑨ 검토, ⑩ 회고와 최종 검증은 생략하지 않는다.
8. ⑩ close-out의 route-appropriate cumulative polish를 마친 뒤 다음 만료 상태를
   기록한다.

   - `route: fast-track (expired)`
   - `autopilot approval: expired`
   - `fold approval: expired`

### Default procedure budget

- scouts: 0 unless a bounded unknown requires one
- quiz: 0 unless the user needs outside knowledge to judge the triple
- live UI opens: 0 unless the user requests it or a rendered artifact needs review
- independent polish reviews: 0 unless document-quality risk requires one
- triple approvals: 1 bundled approval
- gates ③/⑥/⑧/⑨/⑩: always run

Bundled fold delegation은 `autopilot approval: approved`일 때만 소비한다.
Autopilot이 `not approved`이면 fold도 `not approved`로 기록하고 manual
fast-track은 ③ 끝에서 interactive fold approval을 받는다.

### Escalation and scope

fast-track은 현재 요청에만 적용한다. core unknown이나 safety/authority boundary가
드러나면 `route: discovery-heavy`와 만료된 approval을 기록한 뒤 승격한다.
fast-track이나 autopilot 승인은 배포,
삭제, 외부 공유, 비용 발생 같은 별도 권한을 만들지 않는다. 같은 요청을 재개하며
locked `what`이 유지될 때만 승인이 유효하다. 새 follow-up이나 scope 변경은
routing 전에 기존 승인을 만료하고 새로 승인받는다.
