import { strict as assert } from "node:assert";
import { test } from "vitest";
import { buildTreeViewModel, treeItemPhaseLabel, treeMaturityLabel } from "./treeViewModel";

const workspaceData = {
  stages: {
    sprouts: {
      count: 1,
      items: [
        {
          slug: "web-leaf-tree-view",
          status: {
            current_phase: "Architect",
            current_gate: "⑧ Artifact",
            next_action: "implement tree",
          },
        },
      ],
    },
    leaves: {
      count: 2,
      items: [{ slug: "web-workspace-preview-sidebar", status: { current_phase: "Feedback" } }],
    },
    fallen: {
      count: 0,
      items: [],
    },
  },
};

test("buildTreeViewModel summarizes lifecycle stages and optional graph counts", () => {
  const model = buildTreeViewModel(workspaceData, {
    nodes: [{ id: "leaf:a" }, { id: "leaf:b" }],
    edges: [{ source: "leaf:a", target: "leaf:b" }],
  });

  assert.equal(model.totalWork, 3);
  assert.equal(model.pressedCount, 2);
  assert.equal(model.edgeCount, 1);
  assert.equal(model.maturityLabel, "young");
  assert.deepEqual(
    model.stages.map((stage) => [stage.key, stage.count, stage.items.length]),
    [
      ["sprouts", 1, 1],
      ["leaves", 2, 1],
      ["fallen", 0, 0],
    ],
  );
});

test("buildTreeViewModel keeps graph counts unavailable when graph data is absent", () => {
  const model = buildTreeViewModel(workspaceData, null);

  assert.equal(model.pressedCount, null);
  assert.equal(model.edgeCount, null);
});

test("treeMaturityLabel compresses growth into readable bands", () => {
  assert.equal(treeMaturityLabel(0), "seedling");
  assert.equal(treeMaturityLabel(3), "young");
  assert.equal(treeMaturityLabel(10), "branching");
  assert.equal(treeMaturityLabel(30), "grown");
  assert.equal(treeMaturityLabel(31), "mature");
});

test("treeItemPhaseLabel prefers phase and gate while tolerating sparse status", () => {
  assert.equal(
    treeItemPhaseLabel({
      slug: "web-leaf-tree-view",
      _stage: "sprouts",
      status: { current_phase: "Architect", current_gate: "⑧ Artifact", next_action: "implement" },
    }),
    "Architect · ⑧ Artifact",
  );
  assert.equal(
    treeItemPhaseLabel({ slug: "next-action-only", _stage: "sprouts", status: { next_action: "plan" } }),
    "plan",
  );
  assert.equal(treeItemPhaseLabel({ slug: "empty", _stage: "sprouts" }), "—");
});
