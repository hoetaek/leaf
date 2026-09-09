---
name: using-leaf
description: Route explicit LEAF work or work needing structured discovery and design; keep clear ordinary tasks direct. Injected at session start.
---

# Using LEAF

Choose the route before loading another LEAF skill. This file is the canonical
router, also read by the session hook. Routing does not expand user or project
authorization; preserve explicit requests for step-by-step approval.

For a known sprout continuation, inspect its `00-status.md`. If it records
`route: fast-track`, or active fast-track is already known, read
[references/fast-track.md](references/fast-track.md) **before routing**. Preserve
same-request pre-lock/approved resume; expire route and approvals before a new
follow-up or scope change. Do not search `.leaf/` for ordinary direct requests.

## Direct execution

When actual LEAF work is not requested (see below), work directly when the
request and success conditions are clear enough to act.
Research, comparison, explanation, or document output alone does not start LEAF.
Neither task size nor uncertainty about routing is a reason to start it.
Bounded maintenance and clear new implementation both qualify.

For implementation, check execution readiness from the request and local state:

- 재현 또는 현재 상태를 관찰할 수 있다 (new work may start from an absent artifact).
- 성공 조건을 관찰할 수 있다.
- 범위와 제외 범위를 구분할 수 있다.
- 작고 되돌릴 수 있는 첫 실험을 정할 수 있다.
- No core unresolved decision controls data, security, privacy, permissions,
  legal judgment, irreversible effects, external sharing, cost, deployment,
  public contracts, or major structure.

Use the relevant specialist skills and project checks. Implementation starts
with the smallest useful evidence (test, reproduction, measurement, or prototype),
then small changes and verification. Before handoff, perform final verification
and a route-appropriate lightweight implementation review/retrospect. Record
actual decisions, risks, and debt in the nearest existing delivery surface.

Direct work needs no LEAF CLI, lifecycle references, phase files, cumulative
polish, independent document reviewer, or live UI. Finish **`.leaf/` 기록 없이 종료**;
do not reconstruct ③–⑩ after execution. If a core unresolved decision emerges,
continue through Learn only when structured discovery or design is needed.

## Actual LEAF work

Use LEAF when the user explicitly requests LEAF, a durable learning/work record,
a learning session, or collaborative design; or when core unresolved decisions
require structured discovery and design before execution. A clear ordinary task
without that intent stays direct, including ordinary research and writing.
Existing authorization remains valid for the same scope; do not ask the user to
approve again merely to enter a phase or transcribe their decision.

Only after selecting LEAF, read [references/lifecycle.md](references/lifecycle.md)
for phase routing, CLI setup, and close-out. Use `soul`, then `learn` for discovery
or `work` to resume approved Learn. For a new LEAF request, when the user explicitly
asks for fast track and no core unknown remains, use
[references/fast-track.md](references/fast-track.md); otherwise use discovery-heavy
LEAF. Fast-track approval, resume, and expiration rules remain in that reference.
