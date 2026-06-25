import assert from "node:assert/strict";
import { test } from "vitest";
import {
  clampReferenceIndex,
  computePhasePipeline,
  isTextEntryElement,
  leafStamp,
  nextReferenceIndex,
  readingProgressFromRect,
  referenceCount,
  reviewResourcePath,
} from "./reviewReaderModel";
import type { ReviewSource } from "./types";

test("reviewResourcePath returns API path only when a slug exists", () => {
  assert.equal(reviewResourcePath("leaf-a"), "/api/review/leaf-a");
  assert.equal(reviewResourcePath(""), null);
  assert.equal(reviewResourcePath(null), null);
});

test("reference helpers keep reference selection in bounds", () => {
  assert.equal(
    referenceCount({
      references: [
        { relative_path: "a.md", markdown: "A" },
        { relative_path: "b.md", markdown: "B" },
      ],
    }),
    2,
  );
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

function src(gate: string, phase: string, present: boolean): ReviewSource {
  return { gate, phase, relative_path: `${gate}.md`, present, markdown: "" };
}

test("computePhasePipeline counts present gates per phase and excludes Status", () => {
  const sources = [
    src("Status", "Status", true), // must NOT become a 5th bar
    src("① Intent", "Learn", true),
    src("② Unknowns", "Learn", true),
    src("③ Criteria", "Example", true),
    src("④ Wireframe", "Example", true),
    src("⑤ Design", "Architect", true),
    src("⑥ Critic", "Architect", true),
    src("⑦ Tasks", "Architect", false),
    src("⑧ Execution", "Architect", false),
    src("⑨ Review", "Feedback", false),
    src("⑩ Retrospect", "Feedback", false),
  ];
  const pipeline = computePhasePipeline(sources);
  assert.equal(pipeline.length, 4, "exactly four bars, Status excluded");
  assert.deepEqual(
    pipeline.map((p) => [p.phase, p.done, p.total, p.state]),
    [
      ["Learn", 2, 2, "done"],
      ["Example", 2, 2, "done"],
      ["Architect", 2, 4, "partial"], // real partial completion, not fake all-done
      ["Feedback", 0, 2, "zero"],
    ],
  );
});

test("leafStamp prefers pressed, then follows stage (sprout/fallen/leaf)", () => {
  assert.equal(leafStamp("leaf", true), "pressed");
  assert.equal(leafStamp("sprout", true), "pressed");
  assert.equal(leafStamp("sprout", false), "sprout");
  assert.equal(leafStamp("fallen", false), "fallen");
  assert.equal(leafStamp("leaf", false), "leaf");
  assert.equal(leafStamp(undefined, undefined), "leaf");
});
