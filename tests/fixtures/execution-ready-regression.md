# Execution-ready regression scenarios

## Bounded maintenance routing

### Given

- merge conflict, dependency update, or localized regression fix처럼 현재 상태와 완료 조건이 관찰 가능하다.
- 보존해야 할 설계 결정이나 미해결 위험이 없다.

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
