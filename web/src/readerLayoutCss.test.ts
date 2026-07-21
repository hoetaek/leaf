import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const cssPath = resolve(dirname(fileURLToPath(import.meta.url)), "styles.css");
const css = readFileSync(cssPath, "utf8");

test("keeps the report wide when the reader rail no longer fits", () => {
  const start = css.indexOf("@media (width <= 1165px)");
  const end = css.indexOf("@media (width <= 820px)", start);
  const responsiveReader = css.slice(start, end);

  expect(start).toBeGreaterThan(-1);
  expect(end).toBeGreaterThan(start);
  expect(responsiveReader).toContain("grid-template-columns: minmax(0, 1fr);");
  expect(responsiveReader).toMatch(/\.reader > \.rail \{[\s\S]*display: none;/);
  expect(responsiveReader).toMatch(/\.reader-toc \{[\s\S]*display: block;/);
  expect(responsiveReader).toMatch(/\.report \{[\s\S]*justify-self: center;/);
  expect(css).toMatch(/\.reader-toc \{[\s\S]*max-width: 900px;/);
});
