import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "vitest";

function sourceFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? sourceFiles(path) : [path];
  });
}

test("web source files are TypeScript or non-code assets", () => {
  const root = dirname(fileURLToPath(import.meta.url));
  const javascriptSources = sourceFiles(root)
    .map((path) => relative(root, path))
    .filter((path) => /\.(jsx?|mjs|cjs)$/.test(path));

  assert.deepEqual(javascriptSources, []);
});
