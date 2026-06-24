import assert from "node:assert/strict";
import { test } from "vitest";
import { buildGraphModel } from "./graphModel";

test("buildGraphModel keeps only resolvable links and counts node degree", () => {
  const model = buildGraphModel({
    nodes: [
      { id: "leaf:a", slug: "a", title: "Alpha", tags: ["core"] },
      { id: "leaf:b", slug: "b", title: "Beta", tags: [] },
      { id: "leaf:c", slug: "c", title: "Gamma", tags: [] },
    ],
    edges: [
      { source: "leaf:a", target: "leaf:b", predicate: "cites" },
      { source: "leaf:b", target: "leaf:c", predicate: "refines" },
      { source: "leaf:a", target: "missing", predicate: "related_to" },
    ],
  });

  assert.deepEqual(
    model.links.map((link) => `${link.source}->${link.target}:${link.predicate}`),
    ["leaf:a->leaf:b:cites", "leaf:b->leaf:c:refines"],
  );
  assert.equal(model.nodes.find((node) => node.id === "leaf:b")!.degree, 2);
  assert.equal(model.nodes.find((node) => node.id === "leaf:a")!.degree, 1);
});

test("buildGraphModel records hover neighbourhoods for incident links", () => {
  const model = buildGraphModel({
    nodes: [
      { id: "leaf:a", slug: "a", title: "Alpha" },
      { id: "leaf:b", slug: "b", title: "Beta" },
      { id: "leaf:c", slug: "c", title: "Gamma" },
    ],
    edges: [
      { source: "leaf:a", target: "leaf:b", predicate: "cites" },
      { source: "leaf:c", target: "leaf:a", predicate: "depends_on" },
    ],
  });

  assert.deepEqual([...model.neighboursById.get("leaf:a")!].sort(), ["leaf:a", "leaf:b", "leaf:c"]);
  assert.deepEqual([...model.neighboursById.get("leaf:b")!].sort(), ["leaf:a", "leaf:b"]);
});
