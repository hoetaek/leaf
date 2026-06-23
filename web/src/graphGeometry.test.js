import assert from "node:assert/strict";
import test from "node:test";
import { buildDirectedEdgeGeometry } from "./graphGeometry.js";

test("buildDirectedEdgeGeometry trims an edge to node boundaries", () => {
  const edge = buildDirectedEdgeGeometry({ x: 0, y: 0 }, { x: 100, y: 0 }, { sourceRadius: 10, targetRadius: 20 });

  assert.equal(edge.start.x, 10);
  assert.equal(edge.start.y, 0);
  assert.equal(edge.end.x, 80);
  assert.equal(edge.end.y, 0);
});

test("buildDirectedEdgeGeometry places labels toward the target direction", () => {
  const edge = buildDirectedEdgeGeometry(
    { x: 0, y: 0 },
    { x: 100, y: 0 },
    { sourceRadius: 10, targetRadius: 20, labelT: 0.64 },
  );

  assert(edge.label.x > 50);
  assert.equal(edge.label.y, -10);
});

test("buildDirectedEdgeGeometry handles overlapping nodes without NaN", () => {
  const edge = buildDirectedEdgeGeometry({ x: 12, y: 12 }, { x: 12, y: 12 }, { sourceRadius: 6, targetRadius: 6 });

  assert(Number.isFinite(edge.start.x));
  assert(Number.isFinite(edge.end.x));
  assert.equal(edge.path, "M 12 12 L 12 12");
});
