#!/usr/bin/env node

import { readFileSync } from "node:fs";
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

const workPath = "plugins/leaf/skills/work/SKILL.md";
const work = read(workPath);
requireText(workPath, work, "## Execution-first lane");
requireText(workPath, work, "누적 polish, 독립 문서 검토, live UI, phase gate 파일을 요구하지 않는다");
requireOrder(workPath, work, [
  "저장소 상태 확인",
  "최초 실행 증거",
  "작고 되돌릴 수 있는 구현과 검증",
  "실제로 생긴 결정·위험·부채",
]);

const autopilotPath = "plugins/leaf/skills/autopilot/SKILL.md";
const autopilot = read(autopilotPath);
requireText(autopilotPath, autopilot, "execution-ready 분기");
requireText(autopilotPath, autopilot, "④·⑤·⑦을 별도 사람 승인 없이 자동 fold");
requireText(autopilotPath, autopilot, "최초 실행 증거 전에는");

const polishPath = "plugins/leaf/skills/polish/SKILL.md";
const polish = read(polishPath);
requireText(polishPath, polish, "최초 실행 증거 전에는");
requireText(polishPath, polish, "독립 문서 검토");
requireText(polishPath, polish, "실행을 막지 않는다");

const gatesPath = "plugins/leaf/skills/work/references/gates.md";
const gates = read(gatesPath);
requireText(gatesPath, gates, "## Execution-ready folding");
requireText(gatesPath, gates, "⑨ audit");
requireText(gatesPath, gates, "unfold");

if (failures > 0) {
  console.error(`execution-ready contract failed: ${failures} assertion(s)`);
  process.exit(1);
}

console.log("execution-ready contract passed");
