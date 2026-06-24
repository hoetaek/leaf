import assert from "node:assert/strict";
import test from "node:test";
import { clampWorkspaceSelection, filterWorkspaceRows, workspaceCounts } from "./workspaceModel.js";

const workspaceData = {
  stages: {
    sprouts: {
      count: 1,
      items: [{ slug: "web-graph-structure-refactor", status: { next_action: "split graph runtime" } }],
    },
    leaves: {
      count: 1,
      items: [{ slug: "react-lint-format-baseline", status: { next_action: "done" } }],
    },
  },
};

test("workspaceCounts reads stage counts safely", () => {
  assert.deepEqual(workspaceCounts(workspaceData), { sprouts: 1, leaves: 1 });
  assert.deepEqual(workspaceCounts(null), {});
});

test("filterWorkspaceRows filters by stage and searchable text", () => {
  assert.deepEqual(
    filterWorkspaceRows(workspaceData, { stage: "sprouts", query: "runtime" }).map((row) => row.slug),
    ["web-graph-structure-refactor"],
  );
  assert.deepEqual(
    filterWorkspaceRows(workspaceData, { stage: "leaves", query: "runtime" }).map((row) => row.slug),
    [],
  );
});

test("clampWorkspaceSelection keeps keyboard selection inside rows", () => {
  assert.equal(clampWorkspaceSelection(5, 2), 1);
  assert.equal(clampWorkspaceSelection(-1, 2), 0);
  assert.equal(clampWorkspaceSelection(3, 0), 0);
});
