import assert from "node:assert/strict";
import { test } from "vitest";
import { clampGraphPoint, constrainGraphNode, forceGraphBounds, graphNodeRadius } from "./graphPhysics";

test("constrainGraphNode keeps graph nodes inside the visible canvas", () => {
  const node = { x: -170, y: 829, vx: -8, vy: 12 };

  constrainGraphNode(node, { width: 880, height: 520, padding: 40, radius: 16 });

  assert.deepEqual({ x: node.x, y: node.y, vx: node.vx, vy: node.vy }, { x: 56, y: 464, vx: 0, vy: 0 });
});

test("clampGraphPoint keeps dragged nodes inside the same bounds", () => {
  assert.deepEqual(clampGraphPoint(900, -20, { width: 880, height: 520, padding: 40, radius: 10 }), {
    x: 830,
    y: 50,
  });
});

test("forceGraphBounds constrains every simulation node", () => {
  const nodes = [
    { id: "left", slug: "left", title: "Left", description: "", tags: [], x: -40, y: 260, vx: -3, vy: 0, degree: 0 },
    { id: "right", slug: "right", title: "Right", description: "", tags: [], x: 940, y: 600, vx: 5, vy: 7, degree: 4 },
  ];
  const force = forceGraphBounds(880, 520, { padding: 36, radius: graphNodeRadius });

  force.initialize?.(nodes, Math.random);
  force(1);

  assert.equal(nodes[0].x, 42);
  assert.equal(nodes[0].vx, 0);
  assert(nodes[1].x <= 880 - 36 - graphNodeRadius(nodes[1]));
  assert(nodes[1].y <= 520 - 36 - graphNodeRadius(nodes[1]));
  assert.equal(nodes[1].vx, 0);
  assert.equal(nodes[1].vy, 0);
});
