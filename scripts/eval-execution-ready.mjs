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
requireText(usingLeafPath, usingLeaf, "## Fast-track LEAF");
requireText(usingLeafPath, usingLeaf, "references/fast-track.md");

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
]) {
  requireText(fastTrackPath, fastTrack, fastTrackContract);
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
