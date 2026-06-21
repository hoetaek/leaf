---
name: profile
description: "Use when reading, creating, or updating LEAF profiles — the machine-global `~/.config/leaf/profile.md` and the repo-local `.leaf/PROFILE.md`: user language, recurring requirements, agent mistakes, wrong-answer notes, and cross-leaf facts. Read the merged view with `leaf profile`."
---

# LEAF Profile

PROFILE은 두 겹이다.

- **global** `~/.config/leaf/profile.md`: 이 기기의 모든 repo에 적용되는
  기억. 사용자 언어처럼 기기·사용자 단위 사실을 둔다. 경로는
  `LEAF_CONFIG_DIR` → `$XDG_CONFIG_HOME/leaf` → `~/.config/leaf` 순으로
  해석되고, `leaf init`이 없으면 만든다.
- **repo-local** `.leaf/PROFILE.md`: 이 repo에만 적용되는 기억.

읽을 때는 두 파일을 따로 읽지 말고 `leaf profile`이 출력하는 effective
profile을 읽는다. 충돌하면 local이 global을 이긴다.

처음 만들거나 비어 있으면 사용자가 쓰는 작업 언어를 먼저 적는다. 언어는
global에 적는다.

중간중간 사용자의 요구사항, 에이전트의 실수, 오답노트, 재발 방지 교훈, 반복
사실이 어떤 leaf 작업에도 적용되어야 한다면 PROFILE에 적는다. 적을 곳은
이렇게 고른다: 어느 repo에서나 참이면 global, 이 repo에서만 참이면 local,
애매하면 local.

PROFILE 항목의 description은 짧고 명확해야 한다. 한 작업에만 필요한 내용은
gate file, retrospect, pressed에 남기고 PROFILE에는 넣지 않는다.

PROFILE은 `soul`을 부정하지 않는다. `soul`과 충돌하면 `soul`이
이긴다.

PROFILE을 수정하면 어느 파일에 무엇을 추가, 수정, 삭제했는지 사용자에게
보여준다.
