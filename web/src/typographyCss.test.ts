import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const cssPath = resolve(dirname(fileURLToPath(import.meta.url)), "styles.css");
const css = readFileSync(cssPath, "utf8");

test("uses separate readable type tokens for review markdown surfaces", () => {
  expect(css).toContain("--type-ui: 15px;");
  expect(css).toContain("--type-reader: 16px;");
  expect(css).toContain("--type-reader-compact: 15px;");
  expect(css).toContain("--line-reader: 1.65;");

  expect(css).toMatch(
    /\.md p,\n\.md li \{[\s\S]*font-size: var\(--type-reader\);[\s\S]*line-height: var\(--line-reader\);/,
  );
  expect(css).toMatch(
    /\.refdrawer \.refread p,\n\.refdrawer \.refread li \{[\s\S]*font-size: var\(--type-reader-compact\);[\s\S]*line-height: var\(--line-reader\);/,
  );
});
