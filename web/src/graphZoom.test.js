import assert from "node:assert/strict";
import test from "node:test";
import { nextWheelZoom } from "./graphZoom.js";

function graphCoordinate(transform, point) {
  return {
    x: (point.x - transform.x) / transform.k,
    y: (point.y - transform.y) / transform.k,
  };
}

function assertClose(actual, expected) {
  assert(Math.abs(actual - expected) < 0.000001, `${actual} !== ${expected}`);
}

test("nextWheelZoom zooms in around the cursor point", () => {
  const cursor = { x: 440, y: 260 };
  const current = { x: 0, y: 0, k: 1 };
  const before = graphCoordinate(current, cursor);

  const next = nextWheelZoom(current, cursor, { deltaY: -120, deltaMode: 0 });
  const after = graphCoordinate(next, cursor);

  assert(next.k > current.k);
  assert.equal(next.changed, true);
  assertClose(after.x, before.x);
  assertClose(after.y, before.y);
});

test("nextWheelZoom zooms out around the cursor and clamps to min scale", () => {
  const cursor = { x: 210, y: 180 };
  const current = { x: 12, y: -8, k: 0.5 };
  const before = graphCoordinate(current, cursor);

  const next = nextWheelZoom(current, cursor, { deltaY: 5000, deltaMode: 0, minScale: 0.45 });
  const after = graphCoordinate(next, cursor);

  assert.equal(next.k, 0.45);
  assert.equal(next.changed, true);
  assertClose(after.x, before.x);
  assertClose(after.y, before.y);
});

test("nextWheelZoom reports no change when already at the scale limit", () => {
  const next = nextWheelZoom({ x: 0, y: 0, k: 3.4 }, { x: 440, y: 260 }, { deltaY: -120, maxScale: 3.4 });

  assert.deepEqual(next, { x: 0, y: 0, k: 3.4, changed: false });
});
