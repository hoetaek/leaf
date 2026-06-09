# LEAF 시뮬레이션 시나리오 — 사람 × 에이전트

> 한 주제가 `leaf-idea`(씨앗에서 Learn)와 `leaf-work`(잎에서 Example→Feedback)를
> 거쳐 흘러가는 과정을, **추측이 아니라 스킬 본문에 근거해** 재구성한 대화 시나리오입니다.
> 각 에이전트 행동 끝의 `— 근거:`는 그 행동이 어느 스킬 문장에서 나왔는지를 가리킵니다.

**쉽게 한 줄로:** 씨앗(`leaf-idea`)에서 "무엇을·왜"를 배우고 → `leaf promote`로 잎으로
옮긴 뒤 → 잎(`leaf-work`)에서 "한 사례로 증명 → 일반화 → 검증"한다.

- **주제:** 기존 분석 대시보드에 **데이터 내보내기(Export) 기능** 추가
- **등장:** 🧑 사람(제품 담당) · 🤖 에이전트
- **슬러그:** `dashboard-export`

**협업 컨셉 (사장님 ↔ 담당자):** 사용자는 보고를 받고 *결정*하는 사장님, 에이전트는 LEAF
프로세스를 밟는 담당자다. 보고 원칙:

- **보고는 요약이다.** 쉬운 말로 된 개요(몇 초 안에 큰 그림)를 먼저, 그다음 디테일을 펼친다.
- **봐야 할 포인트를 추려 준다.** 원자료는 보기 좋게 정리하되, 진짜 검토·결정할 것은 맨 위
  **Verify / Decide** 목록으로 따로 뺀다.
- **HTML은 직접 열어 보여준다.** Chrome DevTools로 열어 스크린샷 + 확인 포인트를 붙인다 —
  클릭·스크롤을 강요하지 않는다.
- 인지적 게으름·자료 미분류·묻힌 결론은 실패다. 보고 품질이 일의 품질이다.
— 근거: `leaf-soul` **Posture / Voice / Reporting / Show the work** (LEAF 패밀리 공용 품행; `leaf-idea`·`leaf-work`가 참조).

---

## Part 1 — `leaf-idea` (씨앗 / Learn 단계)

### 1-0. 첫 확인과 씨앗 생성

🧑 "대시보드에 export 버튼 하나 넣자."

🤖 먼저 로컬 상태부터 본다. `git status --short --branch`와
`find .leaf/01-seeds .leaf/02-leaves ...`로 비슷한 씨앗이 이미 있는지 확인 →
없으니 중복 생성 안 함.
— 근거: `leaf-idea` **First Read** ("Inspect local truth before asking").

🤖 `.leaf/`가 없어 `leaf init` 후 `leaf new dashboard-export` 실행. 씨앗 골격이
생긴다(`00-status.md` + 네 phase 폴더). 이 골격은 doctor가 요구하는 상태 5필드를
이미 채운 채 만들어진다(state/current phase/current gate/first missing gate/next action).
— 근거: `leaf-idea` **Boundary** ("Create or resume a seed with the `leaf` CLI … `leaf new <slug>`"); `scaffold.rs` 상태 템플릿.

### 1-1. Capture — 분류용 스냅샷

🤖 분류에 필요한 만큼만 짧게 기록한다(아직 full Learn 아님):
- 원문 그대로: "대시보드에 export 버튼 하나 넣자"
- 현재 가설(잠정): "사용자가 데이터를 외부로 빼낼 수단이 필요한 듯"
- 떠오른 이유: 문제 / 의무 / 호기심 / 불편 중 무엇인가?

원문·가설은 `01-Learn/01-intent.md`, 맥락·열린 질문은 `01-Learn/02-unknowns.md`에 쓴다.
— 근거: `leaf-idea` **Capture** ("Record a compact idea snapshot first — enough to triage").

### 1-2. Split Check — 하나의 일인가?

🤖 "export"가 한 덩어리인지 본다. CSV 다운로드 / 이메일 예약 발송 / 외부 API 연동이
섞여 있다면 독립 산출물·성공기준·리뷰어가 갈리므로 `split now` 대상.
지금은 사람이 "그냥 화면에서 내려받기"만 말했으니 **keep grouped**로 둔다.
— 근거: `leaf-idea` **Split Check** (split now / keep grouped / ask first 기준).

### 1-3. Gate ① Intent — 추측한 사실을 먼저 묻는다

🤖 한 문장 intent를 **잠그기 전에**, 내가 *추측하고 있는* 사실을 드러내고 묻는다:

> 다음은 제가 *아는 게 아니라 추측 중인* 것들입니다 (각각 `ASSUMPTION:`):
> - `ASSUMPTION:` 형식은 CSV다 (Excel/JSON이 아니라)
> - `ASSUMPTION:` 현재 화면에 보이는 행만 내보낸다 (전체 데이터셋이 아니라)
> - `ASSUMPTION:` 즉시 다운로드다 (비동기 생성/이메일 발송이 아니라)
> - `ASSUMPTION:` 마감/규정 제약은 없다
>
> 이 중 틀린 게 있으면 잡아주세요. 여기서 잘못 잠그면 뒤 게이트 전부가 어긋납니다.

🧑 "CSV 맞고, 전체 데이터셋이어야 해. 다운로드는… 데이터가 크면 시간이 좀 걸릴 텐데."

🤖 표면 요청("export 버튼")과 더 깊은 why를 가른다 → why는 *"사용자가 지표를 매번 수동
복사해서 보고서에 붙이느라 시간을 버린다"*. 표면 요청은 그 수단일 뿐. 둘을 따로 기록하고,
파생된 한 문장 intent를 원문과 **분리해** 적는다. 핵심 명사(core noun) = "Export 기능"으로 고정.
"데이터가 크면 시간이 걸린다"는 felt sense는 **②로 넘길 unknown**으로 던진다.
— 근거: `leaf-idea` **Gate ① Intent** ("Surface your guessed facts and ask before locking" / "the why followed to where it lands" / "stated separately from the raw wording" / "a felt sense deferred to ②"); `gates.md ①`.

🤖 확인이 필요한 가정은 파일에 `USER REVIEW NEEDED:` / `ASSUMPTION:`로 표시하고
사용자 선호 에디터로 열어 함께 본다.
— 근거: `leaf-idea` **Review Handoff**.

### 1-4. Gate ② Unknowns & Context — 컨텍스트를 *짓고* 핵심을 빼낸다

🤖 unknown을 **종류**(도메인 개념 / 표준·관례 / 선택 기준)와 **출처**(외부 / 내부)로 나눈다.
— 근거: `leaf-idea` **Gate ② Unknowns** ("Categorize unknowns by kind … and by source — external … and internal").

🤖 **외부 사실을 주도한다.** 검색 대상을 먼저 *이름 붙여* 사람에게 묻는다:

> 이런 외부 사실을 찾으려 합니다 — 어떤 걸 제가 찾고, 어떤 건 이미 답을 아시고,
> 제가 안 짚은 건 뭘까요?
> - 비교 도구(Stripe·Linear 등)가 대용량 export를 **동기/비동기** 중 어떻게 처리하나
> - export 진행/완료/실패를 UI로 어떻게 보여주나 (토스트? 패널? 이메일?)
> - CSV 인코딩·대용량 한계의 통념

🧑 "동기/비동기는 네가 찾아봐. 우리 데이터 규모는 내가 줄게 — 큰 계정은 50만 행이야."

🤖 **나만의 컨텍스트 파일을 짓는다 (lazy 아님).** `01-Learn/02-references/`를 *항상* 채운다:
- **외부**: 비교 도구의 export UI를 **렌더된 HTML(+스크린샷)** 자족 파일로 저장해 눈으로 본다.
- **내부**: 현재 대시보드의 렌더된 마크업, 현재 데이터 스키마·행수 같은 *우리가 이미 가진 것*도
  컨텍스트 파일로 끌어온다.
소스별 한 폴더/파일. 베끼기용이 아니라 **판별용**(copy/adapt/avoid/reject).
두 극을 일부러 모은다 — **모범**(벤치마킹·배울 사례, 잘된 export UX)과 **반면교사**(대용량에서
브라우저가 멈추거나 무한 로딩으로 망한 사례), 각각 라벨링. 피할 실패를 짚을 수 있는 게
종종 더 날카롭다.
— 근거: `leaf-idea` **Gate ②** ("Build your own context files … gather both poles: models … and anti-models").

🤖 통념을 못 믿을 때는 독립 실험으로 굳힌다 — 50만 행 mock으로 브라우저 클라이언트
CSV 생성 시간을 직접 재본다(프로젝트 코드 재실행이 아니라 독립 확인). 과정은
`01-Learn/02-experiments/csv-size-probe.md`에, 결론만 밖으로 뺀다.
— 근거: `leaf-idea` **Gate ②** ("use the experiment machine … keep the process in 02-experiments"); `experiment-log.md`.

🤖 **핵심을 밖으로 증류한다.** 컨텍스트 파일에서 *정말 필요한 것*만 출처와 함께
`02-unknowns.md`로 뺀다:
- (확정 사실) 50만 행 클라이언트 CSV 생성 ≈ 수 초 블로킹 → UX 위험 → **비동기 필요할 수 있음**
- (확정 사실) 비교 도구 다수가 "생성 중 → 준비됨 → 다운로드"의 비동기 패턴
- (가정·플래그) 우리 백엔드에 비동기 작업 큐가 있는지는 미확인 → 다운스트림 위험
*안쪽 = `02-references/`(raw 컨텍스트) / 바깥쪽 = `02-unknowns.md`(정제된 사실).*
— 근거: `leaf-idea` **Gate ②** ("Then extract the essentials out … inside … outside").

🤖 게이트 닫힘 판정: 블로킹 unknown에 출처 있는 답/가정이 있고, 사실·가정 경계가 보이며,
사람이 *"무엇을 두고 무엇을 기준으로 고를지"* 말할 수 있다 → ③이 소비할 지점 도달.
— 근거: `leaf-idea` **Gate ②** 닫힘 조건; `gates.md ②`.

### 1-5. Triage & 상태

🤖 매 패스 끝에 한 가지 권고로 닫는다. ①·② 통과 + 사람이 본 작업을 약속 → **promote**.
`00-status.md` 라벨을 `ready-for-leaf-work`로.
— 근거: `leaf-idea` **Triage** 표 / **status labels** ("ready-for-leaf-work: ① … ② … passed").

🤖 짧게 보고: 씨앗 경로 / 상태 / 확인한 근거 / 무엇을 담았나 / 검토 연 파일 / 권고와 이유 / 다음 행동.
— 근거: `leaf-idea` **Response Shape**.

---

## Part 2 — 경계: `leaf promote`

🧑 "좋아, 이걸로 가자."

🤖 사람의 명시적 승인 후에만 `leaf promote dashboard-export` 실행 → 씨앗이
`.leaf/01-seeds/`에서 `.leaf/02-leaves/`로 이동, 상태가 `current phase: Example`로 바뀌고
이전 씨앗 상태는 보존된다. 여기서부터 스킬이 `leaf-work`로 넘어간다.
— 근거: `leaf-idea` **Triage**("Run `leaf promote <slug>` after explicit user approval"); README의 promote 동작.

---

## Part 3 — `leaf-work` (잎 / Example → Architect → Feedback)

### 3-0. Learn을 상속한다 (다시 하지 않는다)

🤖 promote된 잎은 ① intent 잠금 + ② unknowns 해소를 안고 도착한다. `01-Learn/`을
먼저 읽어 한 문장 intent와 why, 출처 있는 사실과 플래그된 가정, 모은 레퍼런스,
"사람이 이미 고를 수 있는 것"을 흡수한다. 그 작업을 *신뢰*하고 다시 하지 않는다.
— 근거: `leaf-work` Always-on **"Inherit Learn; do not redo it."**

🤖 오프닝 프리뷰를 *이 intent에 맞춰* 네 phase의 역량으로 제시(라벨 나열 아님):
Learn(이미 끝남)·Example(싼 한 사례로 증명)·Architect(일반화)·Feedback(유지 확인).
— 근거: `leaf-work` **Response shape — Opening preview**.

### 3-1. ③ Criteria — 테스트를 먼저 쓴다

🤖 답을 만들기 전에 통과 조건을 쓴다. Purpose(의도된 효과 한 문장) + Requirements(관측 가능한
합격 체크). Clarity Ledger 다섯 행(Intent/Topology/Success/Constraints/Output form)을
**먼저 점수 매겨** 잠그거나 명시적으로 보류한다.

관측 가능한 동작이라 EARS 형식 선호:
```
WHEN 사용자가 Export를 누르면, THE SYSTEM SHALL 현재 필터가 적용된 전체 데이터셋을 CSV로 제공한다.
GIVEN 50만 행, WHEN Export를 누르면, THE SYSTEM SHALL 화면을 블로킹하지 않는다.   (성능)
THE SYSTEM SHALL CONTINUE TO 기존 대시보드 필터·정렬을 동일하게 유지한다.        (회귀 민감)
THE SYSTEM SHALL NOT 권한 없는 컬럼을 내보낸다.                                  (non-goal)
```
"비동기 작업 큐 존재 여부" 미확정은 요구사항 안에 **명시적 위험/가정**으로 올린다(모호한 문구로 숨기지 않음).
— 근거: `gates.md ③` (Purpose/Requirements, Clarity Ledger 먼저 점수, EARS, `SHALL CONTINUE TO`, "explicit risk or assumption, not hidden").

### 3-2. ④ Wireframe — 텍스트 → HTML → 상태별

🤖 **텍스트 먼저가 게이트다.** 산문/아스키 아트로 화면 스케치를 그리고 실제 사용자와
워크스루한다. 이게 통과해야 픽셀로 간다.
```
[ 대시보드 헤더 ............................. (Export ▾) ]
 (Export ▾) 클릭 →
 ┌── Export 패널 ─────────────────────────┐
 │ 범위: ( ) 현재 페이지  (•) 전체 데이터셋  │
 │ 형식: [CSV ▾]                           │
 │ [ 내보내기 시작 ]                        │
 └─────────────────────────────────────────┘
```
mock 데이터는 happy path만이 아니라 edge(빈/에러/대용량/다국어)를 드러낼 만큼 현실적으로.
콜드 리더 체크: 화면·라벨·순서만 보고 행위자·목적·다음 행동을 추론할 수 있나?
— 근거: `gates.md ④` ("The text-first wireframe passed before any artifact-specific wireframe", mock data edge cases, cold reader); `gates.md ④` Form("interactive → text-first screen sketch (prose or ASCII-art layout)").

🤖 텍스트가 통과하면 **brownfield edit 레시피로 HTML 뷰를 렌더**해 사람이 눈으로 본다.
이건 brownfield 변경이므로: 실제 대시보드 페이지를 브라우저로 캡처해 **locked context**(손대지
않는 검증된 현실)로 두고, 바뀌는 영역(**variation point**)에만 Export 패널을 mock 데이터로 주입,
점선+배지로 diff 표시, 자족 `.html` 한 파일로 저장 후 offline 새로고침으로 자족성 확인.
— 근거: `gates.md ④` ("render an HTML view … using the brownfield edit recipe … capture the real rendered page as locked context"); `brownfield-html-capture.md` Recipe.

🤖 **결정적 상태들을 별도 뷰로 렌더한다.** 한 화면은 조건값에 따라 여러 화면이다 — 패널을
상태별로 복제해 `02-Example/04-wireframe/` 갤러리로 저장하고 각 파일에 축·값을 제목으로 단다:
`empty`(데이터 0행) / `generating`(비동기 진행) / `ready`(다운로드 가능) / `error`(생성 실패) /
`large`(50만 행) / `denied`(권한 없는 컬럼). 각 상태는 ③ 기준이나 선언된 계약 축으로 추적된다.
— 근거: `gates.md ④` ("Render the decisive states … a small gallery in 02-Example/04-wireframe/ … trace it to a ③ criterion"); `brownfield-html-capture.md` **Render the decisive states**.

🤖 **사람에게 직접 열어 보여준다(클릭 강요 금지).** 저장만 하지 않는다 — Chrome DevTools로
각 상태 뷰를 직접 열어 스크린샷을 찍고, 뷰마다 *확인할 한 가지*를 붙여 보고한다:

> | 상태 | 스크린샷 | 확인 포인트 |
> |---|---|---|
> | empty | 〔shot〕 | 데이터 0행일 때 Export 버튼을 비활성/숨김 처리했나? |
> | generating | 〔shot〕 | 진행 표시가 화면을 블로킹하지 않나? (③ 성능 기준) |
> | error | 〔shot〕 | 실패 사유와 재시도 경로가 보이나? |
> | denied | 〔shot〕 | 권한 없는 컬럼이 빠졌나? (③ non-goal) |

검증은 *흘끗 보면 끝*이어야지, 사용자가 파일을 열거나 UI를 눌러 상태에 도달하게 하지 않는다.
— 근거: `leaf-soul` **Show the work**; `gates.md ④` **"Open the renders for the user"**; `brownfield-html-capture.md` 갤러리 오픈.

> 🔎 이 시점에서 `generating` 상태를 *렌더하려다* "비동기 진행 표시"가 백엔드 작업 큐를
> 전제한다는 게 분명해진다 → **return 발생**(아래 3-7 참조).

### 3-3. ⑤ Design — 사례가 아니라 생성기

🤖 또 하나의 인스턴스가 아니라 *모든 유효 인스턴스를 만드는 규칙*을 쓴다: 컴포넌트 경계
(Export 트리거 / 패널 / 상태 머신), state model(idle→generating→ready→error), 상호작용 규칙,
접근성·포커스 규칙. 데이터/상태 계약은 ④에서 *소비*하지 여기서 정하지 않는다.
brownfield라 기존 시스템의 Static/Dynamic Model로 연다(어느 컴포넌트를 확장 vs 교체하는지를
실제 코드/렌더 동작으로 싸게 확인 — 기억으로 설계하지 않음).
비자명한 선택(동기 vs 비동기)은 RALPLAN-DR 근거(원칙·결정 동인·≥2 옵션·반론과 답)를 남긴다.
— 근거: `gates.md ⑤` ("Build the generator, not another instance", Brownfield Static/Dynamic Model, RALPLAN-DR, "③→④→⑤ never merge across produce/consume edge").

### 3-4. ⑥ Critic — 일반화를 반증한다

🤖 ⑤는 검증된 한 사례를 규칙으로 *귀납 비약*했다 — 태스킹 전에 반증한다. 이 게이트는
**항상 돈다**(깊이만 다름). Export는 권한 경계·대용량/되돌릴 수 없는 작업·공개 인터페이스를
건드리므로 **deep pass**(외부 리뷰어·여러 렌즈·근거 기록). 리뷰어는 사람/다른 에이전트/서브에이전트 가능.
판정 APPROVE / ITERATE / REJECT. ITERATE면 고쳐 재리뷰, REJECT면 ⑤(때로 ②)로 복귀.
— 근거: `gates.md ⑥` ("Falsify the generator before tasking it … It always runs", deep-pass 트리거, 판정/복귀); `design-critic.md`.

### 3-5. ⑦ Task Graph + 실행 전 승인

🤖 일을 리뷰 가능한 단위로 쪼개고 의존성을 그린다. 각 슬라이스에 크기 라벨(small/medium/
large-justified)과 "도움 됐음을 증명하는 체크"를 단다.
— 근거: `gates.md ⑦` ("reviewable chunks", 크기 트립와이어); `task-pr-size-guidance.md`.

🤖 **여기서 명시적 승인 게이트.** ⑧ 실행을 *기본적으로 시작하지 않는다*. 승격된 Architect
스냅샷 — ⑤ Design, ⑥ Critic 판정, ⑦ Task Graph, 실행 범위, 위험, 첫 실행 청크 — 을 사람이
명시적으로 승인해야 한다. (사람이 이 잎에 auto-execution을 미리 허가한 경우만 생략.)

🧑 "스냅샷 봤고, 비동기 방향으로 승인. 첫 청크부터 가자."
— 근거: `leaf-work` Always-on(Architect 예외, 승인 항목 열거); `gates.md ⑧` ("Do not start this gate by default until the user explicitly approves the promoted Architect snapshot").

### 3-6. ⑧ Artifact / Execution

🤖 가장 하중이 큰 불확실 청크부터(첫 섹션이 아니라) — 비동기 생성·상태 폴링 코어. 매끈한
근거 없는 산출보다 못생겨도 확인 가능한 작업을 우선, 플레이스홀더는 정직하게 표시.
세션마다 `03-Architect/08-execution.md`에 "한 일/결과/다음" 기록. "되는가/통과하는가"는
experiment machine로 굳힌다. **사람이 명시적으로 통과라고 말할 때만** 08에 passed로 기록.
— 근거: `gates.md ⑧` (load-bearing 청크 먼저, 세션 로그, "Record … passed … only after the user explicitly says so").

### 3-7. 🔁 Return — 게이트는 파이프라인이 아니라 루프

🤖 (3-2에서 드러난 것 + ⑧에서 확정) 비동기 작업 큐가 우리 스택에 없다는 사실이 가정을
뒤집었다. 발견이 뒤엎는 *가장 이른* 게이트로 돌아간다:
1. **② Unknowns**로 복귀 — "작업 큐 부재"를 출처 있는 사실로 기록(잎의 `01-Learn/`에서
   제자리, 공유 `gates.md ②` 기준).
2. **③ Criteria** 조정 — "블로킹 금지"를 "≤N행은 동기 다운로드, 초과는 비동기 이메일 발송"으로
   purpose를 중재자 삼아 다시 잠근다.
3. 바뀐 것에 의존한 ④ 상태 갤러리·⑤ 상태 머신만 다시 손댄다(전체 재실행 아님).
각 복귀는 `04-Feedback/10-retrospective/mid-process-discoveries.md`에 로그.
— 근거: `leaf-work` Always-on **"Gates loop; they are not a pipeline."**("return to the earliest gate the discovery overturns … A return into ① or ② reopens Learn … in place … against the shared `references/gates.md` ①②"); `gates.md ④`/`⑤` 복귀 조건.

### 3-8. ⑨ Review / Sync

🤖 초안·피드백 후 계획을 참으로 유지. 산출이 기준을 만족하나? 회귀(`SHALL CONTINUE TO`)
지켜졌나? 동기/비동기 분기가 ④ 와이어프레임과 여전히 맞나? 리뷰가 효과를 바꾸면 ③을,
흐름을 바꾸면 ⑤를, 작업량을 바꾸면 ⑦을 동기화한다(바뀐 초안 옆에 낡은 계획 메모를 두지 않음).
이 게이트는 ⑧과 여러 번 루프.
— 근거: `gates.md ⑨` (Review checks, Sync rule, "loops with ⑧").

### 3-9. ⑩ Retrospect — 잎을 닫는 두 축

🤖 **Limitations(내용 회고):** 무엇이 성립했고 권위가 어디서 끝나는가 — "≤N행 동기 export는
검증됨; 비동기 이메일 경로는 큐 SLA 가정 위에서만 성립" 등을 `10-retrospective/limitations.md`에.
이게 이 잎을 **인용 가능한 선행 작업**으로 만든다.
🤖 **Lessons(과정 회고):** 무엇을 너무 일찍 그렸나? 어떤 unknown(작업 큐 부재)이 ②에서
잡혔어야 했나? → 다음 프로젝트 ② 체크리스트에 "인프라 전제(큐/잡) 확인"을 추가.
— 근거: `gates.md ⑩` (Limitations/Lessons, "Which unknowns surfaced mid-work that should have been caught at ②?", ② 체크리스트 갱신).

---

## Part 4 — 닫기 (선택)

- 잎이 끝나 보존·은퇴시킬 때 → `leaf-fall`로 `.leaf/03-fallen/dashboard-export/`로 이동.
- 의도·방법·한계·교훈을 한 인용용 마크다운으로 누를 때 → `leaf-press`로 `.leaf/04-pressed/dashboard-export.md`.
- 언제든 `leaf list`로 인벤토리를, `leaf doctor`로 `.leaf/` 무결성을 점검.
— 근거: 형제 스킬 `leaf-fall` / `leaf-press`; `leaf` CLI(`list`/`doctor`).

---

## 한눈에 — 책임 경계

| | `leaf-idea` (씨앗) | `leaf-work` (잎) |
|---|---|---|
| 소유 단계 | Learn (① Intent, ② Unknowns) | Example·Architect·Feedback (③–⑩) |
| 진입 | `leaf new` | `leaf promote` 후 ③부터 |
| ① 특징 | 추측 사실을 `ASSUMPTION:`로 묻고 잠금 | 잠긴 intent를 *상속* |
| ② 특징 | 외부+내부 레퍼런스로 컨텍스트 파일을 *항상* 짓고 핵심을 `02-unknowns.md`로 증류 | ②를 다시 안 함, 상속 |
| ④ 특징 | (해당 없음) | 텍스트/아스키 통과 → HTML 렌더 → 상태별 갤러리 |
| 게이트 정의 | 공유 `../leaf-work/references/gates.md` 참조 | `references/gates.md` (10게이트 단일 원천) |
| 공용 품행·보고 | `../leaf-soul/SKILL.md` 참조 | `../leaf-soul/SKILL.md` 참조 (태도·목소리·보고·Show the work 단일 원천) |
| 승인 지점 | promote(사람 승인) | phase 전환 + ⑧ 실행 전 스냅샷(사람 승인) |
| 루프 | — | 발견이 ①②를 뒤엎으면 잎의 `01-Learn/`에서 제자리 재개 |
