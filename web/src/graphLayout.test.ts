import assert from "node:assert/strict";
import { test } from "vitest";
import { createInitialGraphLayout, graphChargeStrength, graphLinkDistance } from "./graphLayout";
import { buildGraphModel } from "./graphModel";

test("createInitialGraphLayout copies graph data and gives nodes viewport positions", () => {
  const model = buildGraphModel({
    nodes: [
      { id: "a", slug: "a", title: "Alpha" },
      { id: "b", slug: "b", title: "Beta" },
    ],
    edges: [{ source: "a", target: "b", predicate: "related_to" }],
  });

  const layout = createInitialGraphLayout(model, { width: 100, height: 80 });

  assert.equal(layout.nodes.length, 2);
  assert.equal(layout.links.length, 1);
  assert.notEqual(layout.nodes[0], model.nodes[0]);
  assert.notEqual(layout.links[0], model.links[0]);
  assert.equal(typeof layout.nodes[0].x, "number");
  assert.equal(typeof layout.nodes[0].y, "number");
});

test("graph force helpers scale by node/link degree", () => {
  assert.equal(graphChargeStrength({ degree: 0 }), -118);
  assert.equal(graphChargeStrength({ degree: 2 }), -166);
  assert.ok(
    graphLinkDistance({ source: { degree: 0 }, target: { degree: 0 } }) >
      graphLinkDistance({ source: { degree: 4 }, target: { degree: 4 } }),
  );
});
