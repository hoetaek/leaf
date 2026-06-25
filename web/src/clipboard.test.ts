import assert from "node:assert/strict";
import { afterEach, expect, test, vi } from "vitest";
import { copyLeafCitation, leafCitation } from "./clipboard";
import { subscribeToast } from "./Toast";

afterEach(() => {
  vi.restoreAllMocks();
});

test("leafCitation builds the leaf:<slug> citation token", () => {
  assert.equal(leafCitation("leaf-web-ui"), "leaf:leaf-web-ui");
});

test("copyLeafCitation writes the citation to the clipboard and toasts on success", async () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  vi.stubGlobal("navigator", { clipboard: { writeText } });
  const toasts: string[] = [];
  const unsub = subscribeToast((e) => toasts.push(e.message));

  copyLeafCitation("status-summary-guarantee");
  await Promise.resolve();
  await Promise.resolve();

  expect(writeText).toHaveBeenCalledWith("leaf:status-summary-guarantee");
  assert.deepEqual(toasts, ["copied leaf:status-summary-guarantee"]);
  unsub();
});

test("copyLeafCitation fails quietly when the clipboard API is unavailable", () => {
  vi.stubGlobal("navigator", {});
  const toasts: string[] = [];
  const unsub = subscribeToast((e) => toasts.push(e.message));

  // must not throw
  copyLeafCitation("leaf-web-ui");

  assert.deepEqual(toasts, []);
  unsub();
});

test("copyLeafCitation does not toast when the clipboard write is rejected", async () => {
  const writeText = vi.fn().mockRejectedValue(new Error("denied"));
  vi.stubGlobal("navigator", { clipboard: { writeText } });
  const toasts: string[] = [];
  const unsub = subscribeToast((e) => toasts.push(e.message));

  copyLeafCitation("leaf-web-ui");
  await Promise.resolve();
  await Promise.resolve();

  expect(writeText).toHaveBeenCalled();
  assert.deepEqual(toasts, []);
  unsub();
});
