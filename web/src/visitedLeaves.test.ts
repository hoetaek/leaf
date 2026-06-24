import assert from "node:assert/strict";
import { test } from "vitest";
import { readVisitedLeaves, writeVisitedLeaves } from "./visitedLeaves";

function storageWith(initialValue: string | null = null) {
  let value = initialValue;
  return {
    getItem() {
      return value;
    },
    setItem(_key: string, nextValue: string) {
      value = nextValue;
    },
    value() {
      return value;
    },
  };
}

test("readVisitedLeaves returns a set from stored JSON arrays", () => {
  const visited = readVisitedLeaves(storageWith('["a","b"]'));

  assert.deepEqual([...visited], ["a", "b"]);
});

test("readVisitedLeaves tolerates invalid storage values", () => {
  assert.deepEqual([...readVisitedLeaves(storageWith("not-json"))], []);
  assert.deepEqual([...readVisitedLeaves(storageWith('{"a": true}'))], []);
});

test("writeVisitedLeaves stores visited ids as JSON", () => {
  const storage = storageWith();

  writeVisitedLeaves(new Set(["leaf-a", "leaf-b"]), storage);

  assert.equal(storage.value(), '["leaf-a","leaf-b"]');
});

test("falls back when browser localStorage access is denied", () => {
  const descriptor = Object.getOwnPropertyDescriptor(window, "localStorage");
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    get() {
      throw new Error("denied");
    },
  });

  try {
    assert.deepEqual([...readVisitedLeaves()], []);
    assert.doesNotThrow(() => writeVisitedLeaves(new Set(["leaf-a"])));
  } finally {
    if (descriptor) Object.defineProperty(window, "localStorage", descriptor);
  }
});
