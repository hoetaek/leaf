# Global Profile Design

날짜: 2026-06-12 / 상태: 승인됨

## 목적

repo마다 있는 `.leaf/PROFILE.md`(repo-local 기억) 외에, 기기 단위로 적용되는
기억(사용자 언어 등)을 `~/.config/leaf/profile.md`에 두고, `leaf profile`
명령으로 두 파일을 겹쳐 읽은 effective profile을 보여준다.

## 결정 사항

- **포맷**: 글로벌도 마크다운(`profile.md`). PROFILE은 에이전트가 읽는 prose
  기억이므로 TOML 구조화는 하지 않는다.
- **병합**: 단순 레이어링(글로벌 → 로컬 순 이어붙이기). 섹션 단위 deep-merge
  없음. 충돌 시 로컬 우선, leaf-soul이 최우선 — 해석은 에이전트가 한다.
- **effective profile 보기**: 새 명령 `leaf profile`이 출처 마커
  (`<!-- global: ... -->`, `<!-- local: ... -->`)와 함께 출력. 병합 파일을
  물질화하지 않는다 (stale 문제 없음).

## 동작

### 경로 해석

`LEAF_CONFIG_DIR` → `$XDG_CONFIG_HOME/leaf` → `~/.config/leaf` 순서.
빈 환경변수는 미설정으로 취급한다(XDG 규약).

### `leaf init`

기존 동작에 더해 글로벌 `profile.md`가 없으면 템플릿을 생성한다. 멱등:
있으면 보존, 없으면 생성, 경로가 디렉터리면 에러 — 로컬 `ensure_profile_file`
과 동일한 패턴.

### `leaf profile`

- 글로벌 → 로컬 순으로 각 블록 앞에 출처 마커를 붙여 출력.
- 머리말에 우선순위 규칙 한 줄.
- 글로벌 파일 없음 → `(missing; run `leaf init`)` 마커만 출력.
- git repo 밖에서도 동작: 로컬 블록은 `(not in a git repository)` 마커.

## 범위에서 뺀 것 (YAGNI)

`--json`, doctor 연동, 병합 파일 물질화, profile.toml, 섹션 단위 병합.

## 스킬 변경

`skills/leaf-profile/SKILL.md`:

- 읽기: `leaf profile` 출력으로 effective profile을 읽는다.
- 쓰기 라우팅: 사용자 언어·기기/사용자 전반 사실 → 글로벌, repo 한정 내용 →
  로컬.
- leaf-soul 우선 규칙은 유지.

## 테스트

- 단위: 경로 해석(순수 함수), 글로벌 ensure 멱등성, 렌더링 4분면
  (global×local 존재/부재).
- 통합(tests/cli.rs): `leaf profile` 출력, repo 밖 동작, init의 글로벌 생성·
  보존·멱등. 테스트는 `LEAF_CONFIG_DIR`로 실제 홈을 오염시키지 않게 격리.
