---
name: tend
description: |
  Use to keep pressed leaf documents true to current code — sweep the pressed
  knowledge graph and reconcile drift. Trigger on `leaf tend`, "pressed 문서
  최신화", "tend the leaves", "leaf graph 결과가 코드랑 맞는지", "인용 문서가
  옛날 거 아닌지", "press된 문서 점검", "knowledge graph 정합성". Reads pressed
  nodes via `leaf graph`, verifies each claim against the repo, and proposes
  🟢keep / 🟡correction-banner / 🔴supersede — never rewriting the frozen pressed
  body, never falling without user confirmation. Do not use for prose flow
  cleanup (→ `polish`), first-time pressing (→ `press`), human review handoff
  (→ `review`), editing gate source documents, or reconciling fallen items.
---

# LEAF Tend

pressed 문서는 인용 박제다 — press한 그 순간을 굳혀 나중에 인용하라고 만든
표본이다. 코드가 진화하면 그 안의 검증 가능한 주장(파일 경로, CLI 플래그/동작,
개수·버전, "X 없음" 같은 단언)이 조용히 어긋나고, 어긋난 박제는 인용될 때
사람을 오도한다. `tend`는 **박제는 한 글자도 건드리지 않은 채**, 어긋난 코퍼스를
다시 믿을 수 있게 되돌린다.

박물관 표본에 새 페인트를 덧칠하지 않듯, tend는 pressed 본문을 재작성하지 않는다.
표면이 어긋났으면 위에 **정정 배너**를 붙이고, 핵심 결정 자체가 뒤집혔으면
**supersede**(옛 것을 fallen으로 떨구고 새 leaf로 잇기)를 제안한다. 어느 쪽이든
비가역 행동은 **사용자 승인 후에만** 실행한다.

## Boundary

- `.leaf/` 안에서만 작업한다. `.wt/`·실행 산출물은 건드리지 않는다.
- **본문 불가침**: 게이트 원문(①–⑩)과 pressed 8섹션 원문을 재작성·삭제하지
  않는다. 🟡는 배너를 **추가만**, 🔴는 fall+supersede만.
- **검증 없이 행동 금지(review-chasing 방지)**: "낡아 보임"만으로 배너/fall 하지
  않는다. 코드에서 확인한 증거가 있을 때만 행동한다.
- **비가역은 승인 후**: fall은 triage 표로 제안하고 사용자 승인을 받은 뒤 실행한다.
- 대상은 **pressed 노드만**(`leaf graph`). fallen 재점검, 산문 polish, press/keep
  판단, graph edge 정합성은 tend의 일이 아니다.
- 시작·끝에 `leaf doctor`를 돌려 워크스페이스가 usable한지 확인한다.

## First Read

```bash
git status --short --branch
leaf doctor
leaf graph --json      # pressed 노드 = 점검 대상 전체
```

`leaf`가 PATH에 없으면 중단하고 설치를 안내한다(`install`). 추측으로 진행하지
않는다. 인자로 slug 하나를 받으면(`tend <slug>` — 스킬 호출 인자이지 leaf CLI
서브커맨드가 아님) 그 노드만, 없으면 전수.

## Sweep

```
1. Inventory : leaf graph --json 의 pressed 노드 전수 (N=0이면 종료 보고)
2. 검증      : 각 pressed.md '원문'의 주장을 추출 → repo에서 독립 확인
3. 분류      : 🟢유지 / 🟡정정배너 / 🔴supersede
4. 제안      : 개요 먼저 + triage 표 (비가역 행동 없음)
5. 승인      : 사용자 확인
6. 실행      : 🟡 배너 추가/갱신 / 🔴 fall+supersede
7. 재확인    : leaf doctor + leaf graph 로 코퍼스 일관성
```

**검증 대상은 pressed.md 원문 자체다** — 요약·메모리·다른 에이전트의 보고가
아니라. (실전 교훈: 한 보고가 "이 pressed에 X 경로가 있다"고 했지만 직접 grep하니
없었다. 문서를 믿지 말고 직접 읽어라.)

### 주장 타입별 검증 (무엇을 어떻게 확인하나)

| 타입 | 확인 방법 | 판정 |
|---|---|---|
| (a) 경로·심볼·플래그 | `ls <path>` · `grep -rn <sym> src/` · `leaf <cmd> --help` | 존재↔부재 |
| (b) 개수·버전 | 실제 산출: `ls dir \| wc -l` · Cargo.toml | 값 일치↔불일치 |
| (c) "X 없음/못 함" 부정 단언 | 부재 증명은 난해 | **검증 불가** — 틀렸다 단정 금지 |
| (d) 외부 사실(외부 버전·PR·커밋) | repo 내부에서 확인 불가 | **검증 불가** — "그대로 인용 주의"만 |

주장마다 `확인된 사실` / `어긋남(증거)` / `검증 불가`로 라벨한다. (c)·(d)만으로는
절대 fall하지 않는다.

### 분류 (인용 가능성 축)

기준 질문: **"이 pressed를 그대로 인용하면 독자가 틀린 결정을 내리는가?"**

| 입력 상태 | 분류 |
|---|---|
| 검증 가능한 주장 모두 성립 | 🟢 유지 |
| 표면이 어긋났지만 **독자 판단을 바꾸지 않음**(무해한 카운트 변동 등) | 🟢 유지 + (선택)주석 — 배너 강제 안 함 |
| 표면 사실이 이동했고 **그게 인용 독자에게 실제로 중요**, 핵심 결정은 유효 | 🟡 정정배너 |
| **핵심 결정·주장 자체가 뒤집힘** — 그대로 인용 시 틀린 결정 | 🔴 supersede |
| 검증 불가만 존재(틀린 증거 없음) | 🟢 유지 + 배너에 "검증 불가" 주석(선택) |

### 제안 (승인 전, 비가역 행동 없음)

개요 먼저(🟢/🟡/🔴 분포), 그다음 triage 표:

```
노드 · 분류 · 어긋난 주장 · 증거(명령) · 제안 액션
```

## 정정 배너 (🟡 실행)

- **위치**: frontmatter + H1/source 블록 **뒤**, `## Citation Summary` **앞**.
  blockquote 한 덩이.
- **형식**:
  ```markdown
  > ⚠️ **TEND 정정 노트 — <YYYY-MM-DD>**
  > 이 pressed는 인용 박제다(본문 불변). press 이후 코드 변화로 어긋난 **표면
  > 사실**만 아래에 정정한다. 핵심 결정은 그대로 유효.
  > - 「<원문 표현>」 → 현재 <정정>.
  > - 「<외부 사실>」 = repo 내부 검증 불가(그대로 인용 시 주의).
  > 검증: <확인 명령들>
  ```
- **불가침**: 원문 8섹션·frontmatter 무수정(append-only). 새 `##` 헤딩을 만들지
  않는다 — press의 8섹션 구조를 보존하고 `leaf graph` 노드 파싱(맨 위 첫 `---`
  블록만 읽음)을 지키기 위함이다. (`leaf doctor`는 본문 구조를 검사하지 않으니
  doctor가 막아주리라 기대하지 말 것 — graph 파싱만 보호하면 된다.)
- **idempotency**: 이미 TEND 정정 노트가 있으면 새로 쌓지 말고 그 블록을
  **갱신**한다(날짜 교체 + 항목 병합). 여러 번 돌려도 배너는 1개.
- `00-status.md`에 `## Press Abstract`가 같은 표면 사실을 담고 있으면 동일 정정을
  미러한다(있을 때만). 이 블록은 press 소유지만 tend의 표면 정정은 허용된다.
- 삽입 후 `leaf graph`로 노드가 정상 파싱되는지 확인한다. 첫 `---` 블록이
  깨졌으면 삽입을 되돌리고(원문 보존) 보고한다.

## Supersede (🔴 실행 — 핸드오프)

핵심 결정이 뒤집힌 pressed는 고치지 않고 떨군다:

1. `leaf fall --reason superseded <old-slug>` (폴더 → `03-fallen` 이동).
2. 현재 진실을 담은 **새 leaf**를 길러 잇는다 — tend는 **제안만** 하고, 새 leaf
   작성은 사용자가 `learn`/`work`로 진행하도록 핸드오프한다(새 leaf는 그 자체로
   독립 수명·리뷰가 필요하므로).
3. 새 leaf의 `linked.md`에 `supersedes: <old citation_handle>` edge를 적는다.
- **정직한 한계 고지**: fall하면 옛 노드는 `02-leaves` 밖(`03-fallen`)으로 가고
  `leaf graph`는 `02-leaves`만 읽으므로, 옛 노드는 graph에서 사라지고 새 leaf의
  `supersedes` edge는 fallen 핸들을 가리키는 **단글링**이 된다. 추적은 `03-fallen`
  폴더로만 가능함을 사용자에게 알린다. (박제를 훼손하지 않는 대가다.)
- 사용자가 🔴 제안을 거부하면, 다음 회차 무한 재제안을 막기 위해 배너에
  "supersede 검토됨(거부) — <date>"로 idempotent 마킹한다.

## Family 경계

- `polish` — 누적 문서가 한 편으로 읽히게 **산문**을 다듬는다(의미 불변).
- `press` — 새 leaf를 인용용으로 **처음** 누른다.
- `review` — 게이트 원문을 모아 **사람 리뷰**로 넘긴다.
- `tend` — 이미 눌린 pressed를 **코드와 대조**해 정정배너/supersede(본문 불가침).

## Report

`soul`대로 — 개요 먼저, 결정점 위로, 사실/증거/추측 분리. 비가역 행동은 승인
전에 표로 제안한다. 보고에 담을 것:

- 점검한 pressed 노드 수와 🟢/🟡/🔴 분포;
- 각 어긋남의 증거(확인 명령)와 사실/검증불가 라벨;
- 제안 액션과, 승인이 필요한 비가역 결정;
- 실행 후 `leaf doctor`/`leaf graph` 재확인 결과.

> 박제는 그대로, 인용은 다시 믿을 수 있게.
