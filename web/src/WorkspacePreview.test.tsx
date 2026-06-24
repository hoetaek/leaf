import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import WorkspacePreview from "./WorkspacePreview";
import { mockJsonFetch } from "./test/mockFetch";
import { clearWorkspacePreviewCacheForTest } from "./workspacePreviewCache";
import type { WorkspaceRow } from "./types";

const row: WorkspaceRow = {
  slug: "web-graph-structure-refactor",
  path: ".leaf/01-sprouts/web-graph-structure-refactor",
  _stage: "sprout",
};

const previewData = {
  title: "Graph structure refactor",
  lines: [{ kind: "text", text: "why: split graph runtime" }],
};

beforeEach(() => {
  clearWorkspacePreviewCacheForTest();
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/preview/web-graph-structure-refactor": previewData }));
});

test("preview에 detail로 가는 화살표 링크를 Enter 배지와 함께 렌더한다", () => {
  render(<WorkspacePreview row={row} />);

  const link = screen.getByRole("link", { name: /detail/i });
  expect(link).toHaveAttribute("href", "#/leaf/web-graph-structure-refactor");
  expect(link).toHaveAttribute("aria-keyshortcuts", "Enter");
  expect(link).toHaveTextContent("Enter");
});

test("선택된 row가 없으면 detail 링크를 렌더하지 않는다", () => {
  render(<WorkspacePreview />);

  expect(screen.queryByRole("link", { name: /detail/i })).toBeNull();
});
