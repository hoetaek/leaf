---
name: leaf-profile
description: "Use when reading, creating, or updating `.leaf/PROFILE.md`: user language, recurring requirements, agent mistakes, wrong-answer notes, and cross-leaf facts."
---

# LEAF Profile

`.leaf/PROFILE.md`는 모든 leaf 작업에 적용해야 하는 repo-local 기억이다.

처음 만들거나 비어 있으면 사용자가 쓰는 작업 언어를 먼저 적는다.

중간중간 사용자의 요구사항, 에이전트의 실수, 오답노트, 재발 방지 교훈, 반복
사실이 어떤 leaf 작업에도 적용되어야 한다면 PROFILE에 적는다.

PROFILE 항목의 description은 짧고 명확해야 한다. 한 작업에만 필요한 내용은
gate file, retrospect, pressed에 남기고 PROFILE에는 넣지 않는다.

PROFILE은 `leaf-soul`을 부정하지 않는다. `leaf-soul`과 충돌하면 `leaf-soul`이
이긴다.

PROFILE을 수정하면 무엇을 추가, 수정, 삭제했는지 사용자에게 보여준다.

