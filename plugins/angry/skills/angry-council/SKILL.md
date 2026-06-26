---
name: angry-council
description: Convene the relevant angry-* personas to review one scope, each through its own lens, then a chair synthesizes one ranked verdict. For multi-angle review of a diff, PR, design, doc, or plan. Triggers on "angry council", "angry 패널로 리뷰", "분노 패널". Spawns subagents only when asked; profane only on request.
user-invocable: true
argument-hint: "[리뷰할 범위 — diff/PR/브랜치/파일/디렉토리/설계/문서/계획] [+ 거친 어조 원하면 명시]"
---

# Angry Council

`council`의 분노 버전이다. 차이는 멤버가 즉석에서 만든 역할이 아니라 **미리
정의된 angry-* 페르소나**이고, 각 페르소나가 자기 SKILL을 로드해 **하나의 리뷰
축**만 맡는다는 것이다.

원칙 셋:

- **페르소나 = 차원.** 같은 범위를 각 페르소나가 자기 렌즈로 통째로 본다 (파일을
  나눠 갖는 게 아니라 보는 각도를 나눈다).
- **전원 소집 금지.** 범위에 실제로 존재하는 축의 페르소나만 triage해서 부른다.
  무관한 페르소나는 신호가 아니라 소음이다.
- **의장이 합성.** 독립 병렬 리뷰 → 중복 제거 → 심각도순 → 축이 다른 충돌 조정
  → 고치는 1순위 한 방.

## 패널 (페르소나 = 축)

| 페르소나 | 축 | 언제 부르나 |
|---|---|---|
| `angry-torvalds` | 코드 craftsmanship | 거의 모든 코드 변경 |
| `angry-dijkstra` | correctness 엄밀성 | 알고리즘·상태기계·동시성·자료구조 |
| `angry-feynman` | 이해·추론 | 설계 근거, "왜 이렇게 했나" |
| `angry-pauli` | 검증 가능성 | 주장·지표·가설·성공 기준 |
| `angry-theo` | 보안·무타협 | input·auth·crypto·권한·네트워크 |
| `angry-ramsay` | 출시 준비도 | "다 됐다 / 머지 가능"이라는 주장 |
| `angry-orwell` | 글·완곡어법 | 문서·PR 본문·주석·카피 |
| `angry-rams` | 디자인 미니멀·정직 | UI·컴포넌트·레이아웃·토큰 |
| `angry-jobs` | 제품/UX 취향 | 기능·플로우·제품 결정 |
| `angry-ego` | 진짜 품질 | 완성·릴리즈됐다고 제시된 결과물 |
| `angry-fletcher` | 기준선·안주 거부 | "다 됐다 / 이만하면 됐다"로 안주한 핵심 산출물 |

## 언제 쓰나 / 안 쓰나

사용한다:
- 한 변경/산출물을 여러 각도로 강하게 리뷰해야 하고 blind spot이 비싼 경우
- 사용자가 명시적으로 angry council(분노 패널)을 요청한 경우

쓰지 않는다:
- 한 축이면 충분할 때 — 그 페르소나 하나만 직접 호출한다 (패널은 과하다)
- 단순 사실 조회, 이미 결정된 방향 재표결, 비용이 정당화되지 않는 작업

## 프로토콜

### 1. 범위 + Context Pack 고정

- 무엇을 리뷰하나: diff/PR/브랜치/파일/디렉토리/설계/문서/계획 + 실제 내용
- Known Facts / Fixed Constraints — 각 페르소나가 임의로 바꿀 수 없다
- 어조: 거친 어조는 사용자가 요청했을 때만 (각 페르소나의 어조 가드를 상속)
- 사용자가 지정한 강제 포함/제외 페르소나 반영

### 2. 패널 선택 (triage)

범위에 실제로 존재하는 축만 추려, 얼마나 중심 축인지 순으로 정렬해 **3~5명**을
고른다.

- 관련 축 **1~2개** → council을 부르지 않는다. 그 페르소나를 직접 호출한다
  (의장 합성·병렬 spawn 비용이 정당화되지 않는다).
- 관련 축 **3~5개** → 그대로 소집한다.
- 관련 축 **6개 이상**으로 보이면 → 축이 겹치는 페르소나를 중복 계산한 것이다.
  가장 중심 축으로 병합해 5명 이하로 줄인다 (torvalds↔dijkstra,
  ego↔fletcher↔ramsay는 자주 겹친다).

3~5는 인지적 sweet spot이다 — 시각이 다양할 만큼은 많고, findings가 중복으로
묽어지거나 의장 합성이 둔해질 만큼은 많지 않다. (`angry-torvalds`가 후보를 3~5개로
잡는 것과 같은 이유.)

범위 → 축 매핑:

```
코드/로직 diff   → torvalds · dijkstra · feynman
보안 건드림      → + theo
"완료" 주장      → + ramsay · fletcher
UI/디자인 PR     → rams · jobs · ego (+ torvalds)
문서/PR 본문     → orwell · pauli
계획/제안/근거   → feynman · pauli · ego (+ fletcher)
```

부른 이유와 뺀 축을 한 줄씩 남긴다. silent 누락 금지 — 안 부른 축은 "이 범위에
없음"이라고 명시한다. 매핑 결과가 5명을 넘기면 위 병합 규칙으로 줄인다.

### 3. 독립 병렬 리뷰

선택된 페르소나마다 subagent 하나를 병렬로 띄운다. 각 subagent에게:

- 같은 Context Pack과 같은 범위를 준다
- 지시: "Use the `$angry-<persona>` skill. 이 범위를 네 축으로만 리뷰하라."
- 서로의 답을 넘기지 않는다 (독립성이 다양성을 만든다)

각 산출물은 해당 페르소나의 출력 형식 그대로 + 공통 메타:

```text
Persona: angry-<name>
Severity: critical / high / medium / low
Findings: [issue + 증거 file:line + 고치는 방향]
Out of scope: [내 축이 이 범위에 없으면 그렇게 명시]
```

### 4. 의장 합성 (chair)

새 주장을 invent하지 않고 페르소나 결과만 재조합한다.

- 중복/겹침 병합: 여러 페르소나가 같은 줄을 지적하면 합치고 신호를 강화한다
- 심각도순 정렬
- **축 충돌 조정**: `jobs`("빼라") vs `ramsay`("덜 익었다")는 축이 달라 둘 다 옳을
  수 있다 — 둘 다 살리고 트레이드오프를 명시한다. 진짜 모순(같은 것을 A는 하라,
  B는 하지 말라)만 판정한다
- 소수 의견이라도 근거가 강하면 살린다 — 다수결이 진실이 아니다

### 5. 출력

```markdown
## Angry Council 평결

고치는 1순위: [한 줄 — 어느 페르소나가 제기했는지]
왜 1순위: [다른 후보보다 위인 이유]

소집된 패널: [페르소나 — 부른 이유]
뺀 축: [페르소나 — 이 범위에 없음]

페르소나별 핵심:
- angry-<name>: [한 줄]
  ...

교차 이슈: [여러 페르소나가 함께 짚은 것]
충돌/트레이드오프: [축이 갈린 지점 + 권고]
다음 검증: [실행·테스트로 확인할 것]
```

## Subagent 사용 규칙

- 사용자가 angry council을 명시적으로 요청했을 때만 실제 subagent를 spawn한다.
  아니면 이 프로토콜을 단일 응답으로 축약하거나, 패널을 부를 가치가 있는지 먼저
  제안한다.
- 즉시 차단되는 핵심 작업을 subagent에 넘기지 않는다. council은 판단 보조이고,
  최종 통합과 판단은 메인 agent가 한다.
- 범위가 크면 페르소나별로 파일 슬라이스를 줄 수 있으나, 기본은 각자 전 범위를
  자기 축으로 본다.

## Anti-patterns

- 범위와 무관한 페르소나까지 전원 소집 (소음, 비용 낭비)
- 축이 다른 의견을 다수결로 뭉개기
- 합성 전에 서로의 답을 노출해 early consensus 만들기
- 멀쩡한 코드에 흠을 지어내라고 압박하기 — 각 페르소나의 "멀쩡하면 멀쩡하다 하고
  멈춰라" 가드와 충돌한다 (review-chasing 금지)
- "의장이 그렇게 말했으니 맞다"로 닫기
