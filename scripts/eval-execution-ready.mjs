#!/usr/bin/env node

import {
  existsSync,
  readFileSync,
  mkdtempSync,
  mkdirSync,
  symlinkSync,
  writeFileSync,
  chmodSync,
  rmSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
let failures = 0;

function requireText(path, text, needle) {
  if (!text.replace(/\s+/g, " ").includes(needle.replace(/\s+/g, " "))) {
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
      console.error(
        `${path}: expected ordered text ${needles.map(JSON.stringify).join(" -> ")}`,
      );
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
  "경량 구현 review/retrospect",
  "handoff",
]);
for (const forbiddenBeforeEvidence of [
  "phase gate 파일 작성",
  "누적 polish",
  "독립 문서 검토",
  "live UI 열기",
]) {
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
requireText(
  fixturePath,
  fixture,
  "보존해야 할 설계 결정이나 미해결 위험이 없다",
);
requireText(fixturePath, fixture, "locked `what` 없이 이 상태로 Learn만 재개");
requireText(
  fixturePath,
  fixture,
  "fast-track procedure budget을 유지하지만 autopilot/fold delegation",
);
forbidText(
  fixturePath,
  fixture,
  "procedure budget, autopilot, fold 권한을 주지 않는다",
);
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
  "quiz: 0 unless a learning session benefits from a knowledge check",
  "live UI opens: 0 unless the user requests it or a rendered artifact needs review",
  "independent polish reviews: 0 unless document-quality risk requires one",
  "triple approvals: 1 bundled approval",
  "gates ③/⑥/⑧/⑨/⑩: always run",
]) {
  requireText(fixturePath, fixture, fastTrackBudget);
}

const usingLeafPath = "plugins/leaf/skills/using-leaf/SKILL.md";
const usingLeaf = read(usingLeafPath);
// Static contract checks detect instruction drift; they are not model-behavior tests.
for (const readinessCondition of [
  "재현 또는 현재 상태를 관찰",
  "성공 조건을 관찰",
  "범위와 제외 범위를 구분",
  "작고 되돌릴 수 있는 첫 실험",
  "data, security, privacy, permissions",
  "legal judgment, irreversible effects, external sharing, cost, deployment",
  "public contracts, or major structure",
  "clear new implementation",
  "Research, comparison, explanation, or document output alone does not start LEAF",
  "durable learning/work record",
  "learning session",
  "collaborative design",
  "`.leaf/` 기록 없이 종료",
  "route-appropriate lightweight implementation review/retrospect",
  "Direct work needs no LEAF CLI",
  "Existing authorization remains valid",
])
  requireText(usingLeafPath, usingLeaf, readinessCondition);
requireOrder(usingLeafPath, usingLeaf, [
  "## Direct execution",
  "## Actual LEAF work",
]);
for (const legacy of [
  "It is LEAF work to produce",
  "## Ending a leaf",
  "leaf init",
])
  forbidText(usingLeafPath, usingLeaf, legacy);
if (Buffer.byteLength(usingLeaf) > 3600 || usingLeaf.split("\n").length > 80) {
  failures += 1;
  console.error(
    `${usingLeafPath}: session router exceeds 3600 bytes / 80 lines`,
  );
}
for (const match of usingLeaf.matchAll(/\[[^\]]+\]\((references\/[^)]+)\)/g)) {
  if (!existsSync(resolve(root, dirname(usingLeafPath), match[1]))) {
    failures += 1;
    console.error(`${usingLeafPath}: missing routed reference ${match[1]}`);
  }
}
requireOrder(usingLeafPath, usingLeaf, [
  "For a known sprout continuation",
  "`route: fast-track`",
  "**before routing**",
  "## Direct execution",
]);
for (const contract of [
  "active fast-track is already known",
  "same-request pre-lock/approved resume",
  "expire route and approvals before a new",
  "follow-up or scope change",
  "Do not search `.leaf/` for ordinary direct requests",
  "For a new LEAF request",
])
  requireText(usingLeafPath, usingLeaf, contract);

const lifecyclePath = "plugins/leaf/skills/using-leaf/references/lifecycle.md";
const lifecycle = read(lifecyclePath);
for (const contract of [
  "Read only after",
  "## Fast-track LEAF",
  "fast-track.md",
  "## Which skill to use",
  "## The CLI is the body",
  "leaf fall",
  "leaf:install",
  "0.12.0",
])
  requireText(lifecyclePath, lifecycle, contract);
for (const contract of [
  "For a known sprout continuation",
  "`route: fast-track`",
  "before routing",
  "same-request resume",
  "before a new follow-up or scope change",
  "does not require searching `.leaf/` for ordinary direct requests",
])
  requireText(lifecyclePath, lifecycle, contract);
for (const scenario of [
  "## Known fast-track continuation without repeating the route name",
  "pre-lock continuation",
  "approved same-request continuation",
  "new follow-up",
  "ordinary direct request",
  "no workspace search",
])
  requireText(fixturePath, fixture, scenario);
requireOrder(lifecyclePath, lifecycle, [
  "After ⑩",
  "`polish` the cumulative whole",
  "exact active `route: fast-track`",
]);

// Run the actual hook with an isolated PATH: no user's installed CLI is invoked.
const hookDir = mkdtempSync(resolve(tmpdir(), "leaf-hook-contract-"));
try {
  const bin = resolve(hookDir, "bin");
  mkdirSync(bin);
  for (const command of ["cat", "dirname"]) {
    const source = [`/usr/bin/${command}`, `/bin/${command}`].find(existsSync);
    if (!source) throw new Error(`Missing hook dependency: ${command}`);
    symlinkSync(source, resolve(bin, command));
  }
  const platforms = [
    ["sdk", {}, "additionalContext"],
    ["cursor", { CURSOR_PLUGIN_ROOT: "fixture" }, "additional_context"],
    ["claude", { CLAUDE_PLUGIN_ROOT: "fixture" }, "hookSpecificOutput"],
    [
      "copilot",
      { CLAUDE_PLUGIN_ROOT: "fixture", COPILOT_CLI: "1" },
      "additionalContext",
    ],
  ];
  for (const installed of [false, true]) {
    if (installed) {
      writeFileSync(resolve(bin, "leaf"), "#!/bin/sh\nexit 91\n");
      chmodSync(resolve(bin, "leaf"), 0o755);
    }
    for (const [platform, platformEnv, field] of platforms) {
      const result = spawnSync(
        "/bin/bash",
        [resolve(root, "plugins/leaf/hooks/session-start")],
        {
          encoding: "utf8",
          env: { PATH: bin, ...platformEnv },
        },
      );
      try {
        if (result.status !== 0 || result.stderr)
          throw new Error(`hook failed: ${result.stderr}`);
        const output = JSON.parse(result.stdout);
        if (JSON.stringify(Object.keys(output)) !== JSON.stringify([field]))
          throw new Error("unexpected platform fields");
        if (
          field === "hookSpecificOutput" &&
          (output[field].hookEventName !== "SessionStart" ||
            Object.keys(output[field]).length !== 2)
        )
          throw new Error("invalid Claude hook fields");
        const context =
          field === "hookSpecificOutput"
            ? output[field].additionalContext
            : output[field];
        if (context.split(usingLeaf.trimEnd()).length !== 2)
          throw new Error("router must occur exactly once and unchanged");
        if (context.includes("# LEAF lifecycle"))
          throw new Error("lifecycle was injected eagerly");
        if (
          !installed &&
          !context.includes("Direct work can proceed without it.")
        )
          throw new Error("missing CLI blocks direct route");
        if (installed && context.includes("CLI is not on PATH"))
          throw new Error("false missing-CLI notice");
      } catch (error) {
        failures += 1;
        console.error(
          `hook ${platform}, installed=${installed}: ${error.message}`,
        );
      }
    }
  }
} finally {
  rmSync(hookDir, { recursive: true, force: true });
}

// Close-out consumers must discover the moved lifecycle from their own directory.
for (const path of [
  "plugins/leaf/skills/autopilot/SKILL.md",
  "plugins/leaf/skills/help/SKILL.md",
  "plugins/leaf/skills/press/SKILL.md",
  "plugins/leaf/skills/split/SKILL.md",
  "plugins/leaf/skills/work/SKILL.md",
  "plugins/leaf/skills/work/references/layout.md",
  "plugins/leaf/skills/work/references/gates.md",
]) {
  const content = read(path);
  const references = [
    ...content.matchAll(
      /`(\.\.\/[^`]*using-leaf\/references\/lifecycle\.md)`/g,
    ),
  ];
  if (
    !references.length ||
    references.some(
      ([, reference]) => !existsSync(resolve(root, dirname(path), reference)),
    )
  ) {
    failures += 1;
    console.error(`${path}: close-out reference missing or broken`);
  }
  forbidText(path, content, '`using-leaf` ("Ending a leaf")');
}

const fastTrackPath = "plugins/leaf/skills/using-leaf/references/fast-track.md";
const fastTrack = existsSync(resolve(root, fastTrackPath))
  ? read(fastTrackPath)
  : "";
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
requireText(workPath, work, "route-appropriate lightweight");
requireText(workPath, work, "implementation review or retrospect");
requireOrder(workPath, work, [
  "저장소 상태 확인",
  "최초 실행 증거",
  "작고 되돌릴 수 있는 구현과 검증",
  "실제로 생긴 결정·위험·부채",
  "final verification",
  "implementation review or retrospect",
  "handoff",
]);
forbidText(workPath, work, "then create the concise ③–⑦ records");
forbidText(workPath, work, "The first evidence is not a skipped gate");

const autopilotPath = "plugins/leaf/skills/autopilot/SKILL.md";
const autopilot = read(autopilotPath);
requireText(autopilotPath, autopilot, "execution-ready 분기");
requireText(
  autopilotPath,
  autopilot,
  "execution-ready direct path를 LEAF lifecycle로 바꾸지 않는다",
);
requireText(autopilotPath, autopilot, "fast-track 절차 예산");
requireText(autopilotPath, autopilot, "autopilot approval: approved");
requireText(autopilotPath, autopilot, "fold approval: eligible ④/⑤/⑦ approved");
requireText(autopilotPath, autopilot, "same request");
requireText(autopilotPath, autopilot, "approval: expired");
requireText(autopilotPath, autopilot, "fold approval: expired");
requireText(
  autopilotPath,
  autopilot,
  "history, not an active fast-track route",
);
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
requireText(
  learnPath,
  learn,
  "fold approval: eligible ④/⑤/⑦ approved | not approved",
);
requireText(
  learnPath,
  learn,
  "manual fast-track requests canonical fold approval at ③",
);
requireText(learnPath, learn, "if autopilot is approved without fold approval");
requireText(learnPath, learn, "it runs the full unfolded");

const gate02Path =
  "plugins/leaf/skills/learn/references/gate-02-unknowns-context.md";
const gate02 = read(gate02Path);
requireText(gate02Path, gate02, "route: fast-track");
requireText(gate02Path, gate02, "missing fields grant no delegation");
requireText(gate02Path, gate02, "canonical ③ interactive approval");

const approvalPolicyPath =
  "plugins/leaf/skills/autopilot/references/approval-policy.md";
const approvalPolicy = read(approvalPolicyPath);
requireText(approvalPolicyPath, approvalPolicy, "autopilot approval: approved");
requireText(
  approvalPolicyPath,
  approvalPolicy,
  "Missing fields grant no delegation",
);
requireText(approvalPolicyPath, approvalPolicy, "routing a new");
requireText(approvalPolicyPath, approvalPolicy, "follow-up or scope change");
requireText(approvalPolicyPath, approvalPolicy, "⑩ close-out");
requireText(approvalPolicyPath, approvalPolicy, "ordinary autopilot");
requireText(
  approvalPolicyPath,
  approvalPolicy,
  "manual work uses ③'s interactive approval",
);
requireText(
  approvalPolicyPath,
  approvalPolicy,
  "route-appropriate cumulative polish before",
);
requireText(approvalPolicyPath, approvalPolicy, "For an active `route:");
requireText(approvalPolicyPath, approvalPolicy, "these fast-track-only status");
requireText(approvalPolicyPath, approvalPolicy, "fields are not required");

const changelogPath = "CHANGELOG.md";
forbidText(
  changelogPath,
  read(changelogPath),
  "while retaining the ⑨ audit/unfold check",
);

const usingLeafAgentPath = "plugins/leaf/skills/using-leaf/agents/openai.yaml";
const usingLeafAgent = read(usingLeafAgentPath);
requireText(
  usingLeafAgentPath,
  usingLeafAgent,
  "SKILL.md canonical direct exclusions and route order",
);
requireText(usingLeafAgentPath, usingLeafAgent, "canonical status handoff");
forbidText(usingLeafAgentPath, usingLeafAgent, "trivial reply/edit");
forbidText(usingLeafAgentPath, usingLeafAgent, "no durable LEAF record");

const workAgentPath = "plugins/leaf/skills/work/agents/openai.yaml";
const workAgent = read(workAgentPath);
requireText(
  workAgentPath,
  workAgent,
  "$leaf:using-leaf provides the canonical route and status handoff",
);
forbidText(
  workAgentPath,
  workAgent,
  "Keep execution-ready implementation direct",
);

const autopilotAgentPath = "plugins/leaf/skills/autopilot/agents/openai.yaml";
const autopilotAgent = read(autopilotAgentPath);
requireText(
  autopilotAgentPath,
  autopilotAgent,
  "canonical start checks and recorded delegation",
);
forbidText(
  autopilotAgentPath,
  autopilotAgent,
  "otherwise run the ordinary full loop",
);
forbidText(autopilotAgentPath, autopilotAgent, "exact route: fast-track");

const helpPath = "plugins/leaf/skills/help/SKILL.md";
const help = read(helpPath);
requireText(helpPath, help, "`using-leaf` owns the exact routing predicates");
requireText(helpPath, help, "Clear ordinary research");
requireText(helpPath, help, "new implementation");
requireText(helpPath, help, "explicit LEAF");
requireText(helpPath, help, "structured discovery and design");
requireText(
  helpPath,
  help,
  "Routes direct → request-scoped fast-track → discovery-heavy",
);
requireText(helpPath, help, "approved active fast-track budget");

const soulPath = "plugins/leaf/skills/soul/SKILL.md";
requireText(soulPath, read(soulPath), "fast-track");

for (const [path, required, forbidden] of [
  [
    learnPath,
    [
      "there is no fixed scout count",
      "learning is the user's purpose",
      "Reuse an explicit request or existing approval",
    ],
    [
      "dispatch the four scout subagents",
      "restores the full fan-out",
      "Discovery-heavy Learn uses the per-item",
    ],
  ],
  [
    soulPath,
    ["A phase boundary alone", "User and project"],
    ["open reviewables by default"],
  ],
  [
    polishPath,
    ["complexity", "delegation is authorized", "either route"],
    ["Before calling a full polish complete, delegate"],
  ],
]) {
  const content = read(path);
  for (const phrase of required) requireText(path, content, phrase);
  for (const phrase of forbidden) forbidText(path, content, phrase);
}

const gatesPath = "plugins/leaf/skills/work/references/gates.md";
const gates = read(gatesPath);
requireText(gatesPath, gates, "unfold");
forbidText(gatesPath, gates, "## Execution-ready folding");

const directPathContracts = [
  ["plugins/leaf/skills/help/SKILL.md", "creates no LEAF document"],
  ["plugins/leaf/skills/learn/SKILL.md", "exits without this document flow"],
  [
    "plugins/leaf/skills/work/references/layout.md",
    "Execution-ready direct work does not run these commands",
  ],
  [
    "plugins/leaf/skills/work/references/loop-contract.md",
    "Existing issue, PR, commit, or final handoff",
  ],
  ["plugins/leaf/skills/work/references/engine.md", "these gates do not run"],
  [
    "plugins/leaf/skills/autopilot/references/approval-policy.md",
    "Execution-ready direct work does not use autopilot",
  ],
];
for (const [path, needle] of directPathContracts) {
  requireText(path, read(path), needle);
}

for (const [path, needle] of [
  [
    "plugins/leaf/skills/work/references/layout.md",
    "create the project folder immediately afterward",
  ],
  [
    "plugins/leaf/skills/work/references/loop-contract.md",
    "then concise ③–⑦ records",
  ],
  [
    "plugins/leaf/skills/work/references/engine.md",
    "create that separate record after the first execution evidence",
  ],
  [
    "plugins/leaf/skills/autopilot/references/approval-policy.md",
    "five-condition routing judgment replaces",
  ],
]) {
  forbidText(path, read(path), needle);
}

if (failures > 0) {
  console.error(`execution-ready contract failed: ${failures} assertion(s)`);
  process.exit(1);
}

console.log(
  "execution-ready static contracts and 8 runtime hook scenarios passed (not a model-behavior evaluation)",
);
