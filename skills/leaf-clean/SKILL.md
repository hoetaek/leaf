---
name: leaf-clean
description: |
  Use when a LEAF document needs to be cleaned into a simple, complete current
  report: closing a gate, removing stale options, cutting process chatter,
  simplifying why/what, checking whether a leaf reads as one coherent report, or
  asking an independent reviewer/subagent to judge document quality. Do not use
  for producing the artifact, deciding press/fall close-out, or maintaining
  external execution artifacts.
---

# LEAF Clean

Clean LEAF documents until they read like a finished report, not a transcript.

The standard is Feynman-like: if the writer cannot explain the essence simply,
they probably do not understand it yet. A good LEAF file keeps the needed why,
what, evidence, decision, risk, and next input. It removes stale options,
discarded branches, duplicate reasoning, template residue, and status chatter
that no longer helps the next agent.

> 완전함이란 더 이상 보탤 것이 없는 상태가 아니라 더 이상 뺄 것이 없는
> 상태를 말한다.

Use that sentence as the quality bar. More words are not higher quality. The
best document is the one that leaves nothing important hidden and nothing
unnecessary in the way.

## Boundary

- Work on LEAF document quality only.
- Preserve source truth. Do not delete facts, evidence, decisions, caveats, or
  user wording that affects meaning.
- Do not execute artifact work, create wt artifacts, decide press/fall close-out,
  or maintain external systems.
- Run `leaf doctor` before and after substantial cleanup. If it reports a
  workspace issue, fix the diagnosed issue before trusting inventory output.
- Keep parser-facing fields, paths, command names, code identifiers, and quoted
  source text unchanged unless the cleanup explicitly targets that field.

## First Read

```bash
git status --short --branch
leaf doctor
leaf review <slug>
```

Then identify the smallest document surface to clean:

- one gate file before it is closed;
- `00-status.md` when overview, current gate, or next action drifted;
- a whole `leaf review <slug>` report when the user asks whether the leaf reads
  coherently end to end.

## Clean Pass

Read the whole target before editing. Rewrite only after you can answer:

- 이 문서의 현재 결론은 무엇인가?
- 왜 이 작업이 필요한가?
- 무엇이 실제로 결정되었고 무엇이 폐기되었는가?
- 어떤 evidence가 결론을 지탱하는가?
- 다음 agent가 소비해야 할 입력은 무엇인가?
- 남은 질문은 왜 이 gate를 막지 않는가, 아니면 무엇을 막는가?

Then edit toward the current report:

- lead with the current conclusion;
- keep the strongest evidence close to the claim it supports;
- collapse old alternatives into one sentence when only the decision matters;
- remove stale TODOs, dead options, repeated caveats, and template headings;
- state assumptions and unresolved limits plainly;
- keep Korean prose when the user is working in Korean.

Extra checks: drift, surface, archive, fallen.

- drift: `00-status.md` must match the existing gate files.
- surface: separate canonical report, checkpoint history, pressed digest, and
  fallen closure.
- archive: checkpoint files are history; do not rewrite them unless asked.
- fallen: fallen items should lead with fallen reason, closure, lesson, and
  limits.

## Subagent Review

Before calling a cleanup complete, ask for an independent reviewer/subagent when
available. The reviewer judges only document quality, not implementation truth.
Give it the target file or `leaf review` output and this rubric:

```text
Judge whether this LEAF document is a simple complete current report.
Use the standard: "완전함이란 더 이상 보탤 것이 없는 상태가 아니라 더 이상 뺄 것이 없는 상태를 말한다."

Return:
- keep: the sentences or sections that carry the essence
- cut: stale, duplicated, decorative, or non-load-bearing text
- unclear: why/what/evidence/decision gaps
- drift/surface: status/file mismatch, archive confusion, or fallen closure gaps
- verdict: pass or revise
```

If no subagent is available, run the rubric yourself and label it as self-review.

## Pass Criteria

A cleaned LEAF document passes when:

- a new agent can state the why and what after one read;
- the current decision is easier to find than the history of how it was reached;
- current stage, gate, decision, and next action match the existing artifacts;
- every remaining section either carries meaning, evidence, a risk, or a next
  input;
- checkpoint/archive and fallen closure do not confuse the current report;
- no removed text changes the source truth;
- `leaf doctor` still reports the workspace as usable, or remaining findings are
  named precisely.

## Report

Report:

- target files cleaned;
- what was removed or compressed;
- what source truth was preserved;
- reviewer/subagent verdict, or self-review verdict;
- `leaf doctor` result;
- confirmation that no `.wt/` or execution artifacts were created.
