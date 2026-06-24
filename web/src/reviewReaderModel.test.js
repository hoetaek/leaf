import assert from "node:assert/strict";
import test from "node:test";
import {
  clampReferenceIndex,
  isTextEntryElement,
  nextReferenceIndex,
  readingProgressFromRect,
  referenceCount,
  reviewResourcePath,
} from "./reviewReaderModel.js";

test("reviewResourcePath returns API path only when a slug exists", () => {
  assert.equal(reviewResourcePath("leaf-a"), "/api/review/leaf-a");
  assert.equal(reviewResourcePath(""), null);
  assert.equal(reviewResourcePath(null), null);
});

test("reference helpers keep reference selection in bounds", () => {
  assert.equal(referenceCount({ references: [{}, {}] }), 2);
  assert.equal(referenceCount({}), 0);
  assert.equal(clampReferenceIndex(4, 2), 1);
  assert.equal(clampReferenceIndex(-1, 2), 0);
  assert.equal(nextReferenceIndex(0, 1, 2), 1);
  assert.equal(nextReferenceIndex(1, 1, 2), 1);
});

test("isTextEntryElement detects fields that should keep keyboard input", () => {
  assert.equal(isTextEntryElement({ tagName: "INPUT" }), true);
  assert.equal(isTextEntryElement({ tagName: "TEXTAREA" }), true);
  assert.equal(isTextEntryElement({ tagName: "DIV" }), false);
  assert.equal(isTextEntryElement(null), false);
});

test("readingProgressFromRect clamps progress between start and end", () => {
  assert.equal(readingProgressFromRect({ top: 10, height: 1000 }, 500), 0);
  assert.equal(readingProgressFromRect({ top: -250, height: 1000 }, 500), 0.5);
  assert.equal(readingProgressFromRect({ top: -900, height: 1000 }, 500), 1);
});
