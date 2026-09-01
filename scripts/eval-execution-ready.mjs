#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
let failures = 0;

function requireText(path, text, needle) {
  if (!text.includes(needle)) {
    failures += 1;
    console.error(`${path}: missing ${JSON.stringify(needle)}`);
  }
}

function forbidText(path, text, needle) {
  if (text.includes(needle)) {
    failures += 1;
    console.error(`${path}: forbidden ${JSON.stringify(needle)}`);
  }
}

function requireOrder(path, text, needles) {
  let previous = -1;
  for (const needle of needles) {
    const index = text.indexOf(needle, previous + 1);
    if (index === -1 || index <= previous) {
      failures += 1;
      console.error(`${path}: expected ordered text ${needles.map(JSON.stringify).join(" -> ")}`);
      return;
    }
    previous = index;
  }
}

const fixturePath = "tests/fixtures/execution-ready-regression.md";
const fixture = read(fixturePath);
requireOrder(fixturePath, fixture, [
  "기존 요구와 저장소 상태",
  "실패 회귀 테스트",
  "작은 구현과 검증",
  "결정·위험·부채",
  "최종 검증",
]);
for (const forbiddenBeforeEvidence of ["phase gate 파일 작성", "누적 polish", "독립 문서 검토", "live UI 열기"]) {
  requireText(fixturePath, fixture, forbiddenBeforeEvidence);
}
for (const zeroBudget of [
  "leaf scaffold: 0",
  "phase transitions: 0",
  "cumulative polish: 0",
  "independent document reviews: 0",
  "live UI opens: 0",
]) {
  requireText(fixturePath, fixture, zeroBudget);
}
requireText(fixturePath, fixture, "최초 실행 증거 뒤에도");
requireText(fixturePath, fixture, "이 경우에만");
requireText(fixturePath, fixture, "durable LEAF record를 명시적으로");
requireText(fixturePath, fixture, "보존해야 할 설계 결정이나 미해결 위험이 없다");
requireText(fixturePath, fixture, "locked `what` 없이 이 상태로 Learn만 재개");
requireText(fixturePath, fixture, "fast-track procedure budget을 유지하지만 autopilot/fold delegation");
forbidText(fixturePath, fixture, "procedure budget, autopilot, fold 권한을 주지 않는다");
requireOrder(fixturePath, fixture, [
  "## Bounded maintenance routing",
  "보존해야 할 설계 결정이나 미해결 위험이 없다",
  "durable LEAF record를 명시적으로 요청하지 않았다",
  "bounded maintenance로 분류하고 LEAF를 시작하지 않는다",
]);
requireOrder(fixturePath, fixture, [
  "direct execution",
  "fast-track LEAF",
  "discovery-heavy LEAF",
]);
for (const fastTrackBudget of [
  "scouts: 0 unless a bounded unknown requires one",
  "quiz: 0 unless the user needs outside knowledge to judge the triple",
  "live UI opens: 0 unless the user requests it or a rendered artifact needs review",
  "independent polish reviews: 0 unless document-quality risk requires one",
  "triple approvals: 1 bundled approval",
  "gates ③/⑥/⑧/⑨/⑩: always run",
]) {
  requireText(fixturePath, fixture, fastTrackBudget);
}

const usingLeafPath = "plugins/leaf/skills/using-leaf/SKILL.md";
const usingLeaf = read(usingLeafPath);
for (const readinessCondition of [
  "## Execution-ready 구현",
  "재현 또는 현재 상태를 관찰",
  "성공 조건을 관찰",
  "범위와 제외 범위를 구분",
  "작고 되돌릴 수 있는 첫 실험",
  "데이터·보안·공개 계약·대규모 구조",
  "discovery-heavy",
]) {
  requireText(usingLeafPath, usingLeaf, readinessCondition);
}
requireText(usingLeafPath, usingLeaf, "bounded maintenance");
requireText(usingLeafPath, usingLeaf, "작업 크기만으로 LEAF를 시작하지 않는다");
requireText(usingLeafPath, usingLeaf, "`.leaf/` 기록 없이 종료");
requireText(usingLeafPath, usingLeaf, "| no LEAF skill |");
requireText(usingLeafPath, usingLeaf, "canonical router가 direct로 판정한");
requireText(usingLeafPath, usingLeaf, "When the canonical router selects direct execution");
requireText(usingLeafPath, usingLeaf, "## Fast-track LEAF");
requireText(usingLeafPath, usingLeaf, "references/fast-track.md");
requireText(usingLeafPath, usingLeaf, "durable LEAF 기록을 명시적으로 요청하지");
requireText(usingLeafPath, usingLeaf, "user explicitly asks for a durable LEAF record");
requireOrder(usingLeafPath, usingLeaf, [
  "After ⑩",
  "`polish` the cumulative whole",
  "exact active `route: fast-track`",
]);
requireOrder(usingLeafPath, usingLeaf, [
  "execution-ready 조건을 판정",
  "fast-track 조건을 만족하면 fast-track",
  "아니면 discovery-heavy",
]);

const fastTrackPath = "plugins/leaf/skills/using-leaf/references/fast-track.md";
const fastTrack = existsSync(resolve(root, fastTrackPath)) ? read(fastTrackPath) : "";
if (!fastTrack) {
  failures += 1;
  console.error(`${fastTrackPath}: missing fast-track contract`);
}
for (const fastTrackContract of [
  "요청 단위",
  "## 기본 절차 예산",
  "scout | 0",
  "quiz | 0",
  "live UI | 0",
  "독립 polish reviewer | 0",
  "③ · ⑥ · ⑧ · ⑨ · ⑩",
  "core unknown",
  "권한을 추가하지 않는다",
  "route: fast-track | fast-track (expired) | discovery-heavy",
  "autopilot approval: approved | not approved | expired",
  "fold approval: eligible ④/⑤/⑦ approved | not approved | expired",
  "locked `what`",
  "routing 전에",
  "⑩ close-out",
  "autopilot approval: not approved`이면 `fold approval`도",
  "canonical interactive",
  "승인 대기 전",
  "route: discovery-heavy",
  "승인 전 Learn 재개 표지",
  "locked `what`이 없어도",
  "fast-track procedure budget으로 이어간다",
  "Approved delegation",
]) {
  requireText(fastTrackPath, fastTrack, fastTrackContract);
}
for (const statusField of [
  "route: fast-track",
  "autopilot approval: approved",
  "fold approval: eligible ④/⑤/⑦ approved",
  "autopilot approval: expired",
  "fold approval: expired",
  "route: fast-track (expired)",
  "autopilot approval: not approved",
  "fold approval: not approved",
  "route: discovery-heavy",
]) {
  requireText(fixturePath, fixture, statusField);
}

const workPath = "plugins/leaf/skills/work/SKILL.md";
const work = read(workPath);
requireText(workPath, work, "## Execution-first lane");
requireText(workPath, work, "## Fast terminal");
requireText(workPath, work, "최초 실행 증거 뒤에도");
requireText(workPath, work, "`.leaf/` scaffold를 만들지 않는다");
requireOrder(workPath, work, [
  "저장소 상태 확인",
  "최초 실행 증거",
  "작고 되돌릴 수 있는 구현과 검증",
  "실제로 생긴 결정·위험·부채",
]);
forbidText(workPath, work, "then create the concise ③–⑦ records");
forbidText(workPath, work, "The first evidence is not a skipped gate");

const autopilotPath = "plugins/leaf/skills/autopilot/SKILL.md";
const autopilot = read(autopilotPath);
requireText(autopilotPath, autopilot, "execution-ready 분기");
requireText(autopilotPath, autopilot, "execution-ready direct path를 LEAF lifecycle로 바꾸지 않는다");
requireText(autopilotPath, autopilot, "fast-track 절차 예산");
requireText(autopilotPath, autopilot, "autopilot approval: approved");
requireText(autopilotPath, autopilot, "fold approval: eligible ④/⑤/⑦ approved");
requireText(autopilotPath, autopilot, "same request");
requireText(autopilotPath, autopilot, "approval: expired");
requireText(autopilotPath, autopilot, "fold approval: expired");
requireText(autopilotPath, autopilot, "history, not an active fast-track route");
requireText(autopilotPath, autopilot, "only if");
requireText(autopilotPath, autopilot, "exact `route: fast-track`");
requireOrder(autopilotPath, autopilot, [
  "Complete route-appropriate cumulative polish first",
  "fast-track (expired)",
]);
forbidText(autopilotPath, autopilot, "④·⑤·⑦을 별도 사람 승인 없이 자동 fold");

const polishPath = "plugins/leaf/skills/polish/SKILL.md";
const polish = read(polishPath);
requireText(polishPath, polish, "최초 실행 증거 뒤에도 polish 대상이 아니다");
requireText(polishPath, polish, "독립 문서 검토");
requireText(polishPath, polish, "실행을 막지 않는다");
requireText(polishPath, polish, "fast-track");

const learnPath = "plugins/leaf/skills/learn/SKILL.md";
const learn = read(learnPath);
requireText(learnPath, learn, "fast-track");
requireText(learnPath, learn, "한 번의 묶음 승인");
requireText(learnPath, learn, "autopilot approval: approved |");
requireText(learnPath, learn, "fold approval: eligible ④/⑤/⑦ approved | not approved");
requireText(learnPath, learn, "manual fast-track requests canonical fold approval at ③");

const gate02Path = "plugins/leaf/skills/learn/references/gate-02-unknowns-context.md";
const gate02 = read(gate02Path);
requireText(gate02Path, gate02, "route: fast-track");
requireText(gate02Path, gate02, "missing fields grant no delegation");
requireText(gate02Path, gate02, "canonical ③ interactive approval");

const approvalPolicyPath = "plugins/leaf/skills/autopilot/references/approval-policy.md";
const approvalPolicy = read(approvalPolicyPath);
requireText(approvalPolicyPath, approvalPolicy, "autopilot approval: approved");
requireText(approvalPolicyPath, approvalPolicy, "Missing fields grant no delegation");
requireText(approvalPolicyPath, approvalPolicy, "routing a new");
requireText(approvalPolicyPath, approvalPolicy, "follow-up or scope change");
requireText(approvalPolicyPath, approvalPolicy, "⑩ close-out");
requireText(approvalPolicyPath, approvalPolicy, "ordinary autopilot");
requireText(approvalPolicyPath, approvalPolicy, "manual work uses ③'s interactive approval");
requireText(approvalPolicyPath, approvalPolicy, "route-appropriate cumulative polish before");

const usingLeafAgentPath = "plugins/leaf/skills/using-leaf/agents/openai.yaml";
const usingLeafAgent = read(usingLeafAgentPath);
requireText(usingLeafAgentPath, usingLeafAgent, "SKILL.md canonical direct exclusions and route order");
requireText(usingLeafAgentPath, usingLeafAgent, "canonical status handoff");
forbidText(usingLeafAgentPath, usingLeafAgent, "trivial reply/edit");
forbidText(usingLeafAgentPath, usingLeafAgent, "no durable LEAF record");

const workAgentPath = "plugins/leaf/skills/work/agents/openai.yaml";
const workAgent = read(workAgentPath);
requireText(workAgentPath, workAgent, "$leaf:using-leaf provides the canonical route and status handoff");
forbidText(workAgentPath, workAgent, "Keep execution-ready implementation direct");

const autopilotAgentPath = "plugins/leaf/skills/autopilot/agents/openai.yaml";
const autopilotAgent = read(autopilotAgentPath);
requireText(autopilotAgentPath, autopilotAgent, "canonical start checks and recorded delegation");
forbidText(autopilotAgentPath, autopilotAgent, "otherwise run the ordinary full loop");
forbidText(autopilotAgentPath, autopilotAgent, "exact route: fast-track");

const helpPath = "plugins/leaf/skills/help/SKILL.md";
const help = read(helpPath);
requireText(helpPath, help, "`using-leaf` owns the exact routing predicates");
requireText(helpPath, help, "replies/edits and direct lookups are unconditional direct exclusions");
requireText(helpPath, help, "no durable unresolved decision remains");
requireText(helpPath, help, "no durable LEAF record was explicitly requested");
requireText(helpPath, help, "Execution-ready implementation");
requireText(helpPath, help, "a durable LEAF record is explicitly requested");
requireText(helpPath, help, "eligible, otherwise discovery-heavy Learn");
requireText(helpPath, help, "Routes direct → request-scoped fast-track → discovery-heavy");
requireText(helpPath, help, "approved active fast-track budget");

const soulPath = "plugins/leaf/skills/soul/SKILL.md";
requireText(soulPath, read(soulPath), "fast-track");

const gatesPath = "plugins/leaf/skills/work/references/gates.md";
const gates = read(gatesPath);
requireText(gatesPath, gates, "unfold");
forbidText(gatesPath, gates, "## Execution-ready folding");

const directPathContracts = [
  ["plugins/leaf/skills/help/SKILL.md", "creates no LEAF document"],
  ["plugins/leaf/skills/learn/SKILL.md", "exits without this document flow"],
  ["plugins/leaf/skills/work/references/layout.md", "Execution-ready direct work does not run these commands"],
  ["plugins/leaf/skills/work/references/loop-contract.md", "Existing issue, PR, commit, or final handoff"],
  ["plugins/leaf/skills/work/references/engine.md", "these gates do not run"],
  ["plugins/leaf/skills/autopilot/references/approval-policy.md", "Execution-ready direct work does not use autopilot"],
];
for (const [path, needle] of directPathContracts) {
  requireText(path, read(path), needle);
}

for (const [path, needle] of [
  ["plugins/leaf/skills/work/references/layout.md", "create the project folder immediately afterward"],
  ["plugins/leaf/skills/work/references/loop-contract.md", "then concise ③–⑦ records"],
  ["plugins/leaf/skills/work/references/engine.md", "create that separate record after the first execution evidence"],
  ["plugins/leaf/skills/autopilot/references/approval-policy.md", "five-condition routing judgment replaces"],
]) {
  forbidText(path, read(path), needle);
}

if (failures > 0) {
  console.error(`execution-ready contract failed: ${failures} assertion(s)`);
  process.exit(1);
}

console.log("execution-ready contract passed");
