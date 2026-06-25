import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import WorkspaceList from "./WorkspaceList";
import { mockJsonFetch } from "./test/mockFetch";
import { clearWorkspacePreviewCacheForTest } from "./workspacePreviewCache";

const workspaceData = {
  stages: {
    sprouts: {
      count: 1,
      items: [
        {
          slug: "web-graph-structure-refactor",
          path: ".leaf/01-sprouts/web-graph-structure-refactor",
          status: {
            current_phase: "Architect",
            current_gate: "⑦ Tasks",
            next_action: "split graph runtime",
            parse_state: "ok",
            progress_done: 6,
            progress_current: 7,
            progress_label: "6/10",
          },
        },
      ],
    },
    leaves: {
      count: 2,
      items: [
        {
          slug: "react-lint-format-baseline",
          path: ".leaf/02-leaves/react-lint-format-baseline",
          status: {
            current_phase: "Feedback",
            current_gate: "⑩ Retrospect",
            next_action: "done",
            parse_state: "ok",
            progress_done: 10,
            progress_current: 10,
            progress_label: "10/10",
          },
        },
        {
          slug: "typescript-migration-plan",
          path: ".leaf/02-leaves/typescript-migration-plan",
          status: {
            next_action: "document strict conversion",
            parse_state: "warn",
          },
        },
      ],
    },
    fallen: {
      count: 1,
      items: [{ slug: "old-web-ui", path: ".leaf/03-fallen/old-web-ui", status: { next_action: "superseded" } }],
    },
  },
};

const previewData = {
  "web-graph-structure-refactor": {
    title: "web-graph-structure-refactor",
    lines: [
      { kind: "heading", level: 1, text: "상태" },
      { kind: "text", text: "graph preview body" },
      { kind: "source_boundary", phase: "Learn", gate: "① Intent", source: "01-Learn/01-intent.md" },
      { kind: "list_item", marker: "-", text: "preview list item" },
    ],
  },
  "react-lint-format-baseline": {
    title: "react-lint-format-baseline",
    lines: [
      { kind: "heading", level: 1, text: "상태" },
      { kind: "text", text: "react preview body" },
    ],
  },
  "typescript-migration-plan": {
    title: "typescript-migration-plan",
    lines: [{ kind: "text", text: "typescript preview body" }],
  },
  "old-web-ui": {
    title: "old-web-ui",
    lines: [{ kind: "text", text: "old preview body" }],
  },
};

beforeEach(() => {
  clearWorkspacePreviewCacheForTest();
  vi.stubGlobal(
    "fetch",
    mockJsonFetch({
      "/api/list": workspaceData,
      "/api/preview/web-graph-structure-refactor": previewData["web-graph-structure-refactor"],
      "/api/preview/react-lint-format-baseline": previewData["react-lint-format-baseline"],
      "/api/preview/typescript-migration-plan": previewData["typescript-migration-plan"],
      "/api/preview/old-web-ui": previewData["old-web-ui"],
    }),
  );
  vi.spyOn(window, "scrollBy").mockImplementation(() => undefined);
  window.location.hash = "#/";
});

test("renders workspace counts and filters rows by search text", async () => {
  render(<WorkspaceList />);

  expect(screen.getByText("불러오는 중…")).toBeInTheDocument();
  expect(await screen.findByRole("heading", { name: "Workspace" })).toBeInTheDocument();
  expect(screen.getByText("2 leaves · 1 sprouts · 1 fallen")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /web-graph-structure-refactor/ })).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /react-lint-format-baseline/ })).toBeInTheDocument();

  fireEvent.change(screen.getByPlaceholderText("filter by slug, action"), { target: { value: "strict" } });

  expect(screen.getByRole("link", { name: /typescript-migration-plan/ })).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: /web-graph-structure-refactor/ })).not.toBeInTheDocument();
  expect(screen.getByText("1 shown")).toBeInTheDocument();
});

test("shows the selected row preview and toggles it with p", async () => {
  render(<WorkspaceList />);

  expect(await screen.findByText("graph preview body")).toBeInTheDocument();
  expect(screen.getByText(".leaf/01-sprouts/web-graph-structure-refactor")).toBeInTheDocument();
  expect(screen.getByText("① Intent")).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "p" });
  await waitFor(() => {
    expect(screen.queryByText("graph preview body")).not.toBeInTheDocument();
  });

  fireEvent.keyDown(window, { key: "p" });
  expect(await screen.findByText("graph preview body")).toBeInTheDocument();
  expect(
    vi.mocked(fetch).mock.calls.filter(([input]) => input === "/api/preview/web-graph-structure-refactor"),
  ).toHaveLength(1);

  fireEvent.keyDown(window, { key: "j" });
  expect(await screen.findByText("react preview body")).toBeInTheDocument();
});

test("keeps hover quiet, selects preview on click, and opens rows on double-click", async () => {
  render(<WorkspaceList />);

  expect(await screen.findByText("graph preview body")).toBeInTheDocument();

  const reactRow = screen.getByRole("link", { name: /react-lint-format-baseline/ });
  fireEvent.mouseEnter(reactRow);

  expect(screen.getByText("graph preview body")).toBeInTheDocument();
  expect(screen.queryByText("react preview body")).not.toBeInTheDocument();

  fireEvent.click(reactRow);
  expect(await screen.findByText("react preview body")).toBeInTheDocument();
  expect(window.location.hash).toBe("#/");

  fireEvent.doubleClick(reactRow);
  expect(window.location.hash).toBe("#/leaf/react-lint-format-baseline");
});

test("supports keyboard selection, stage switching, and opening the selected row", async () => {
  render(<WorkspaceList />);
  await screen.findByRole("link", { name: /web-graph-structure-refactor/ });

  fireEvent.keyDown(window, { key: "j" });
  await waitFor(() => {
    expect(screen.getByRole("link", { name: /react-lint-format-baseline/ })).toHaveClass("sel");
  });

  fireEvent.keyDown(window, { key: "G" });
  await waitFor(() => {
    expect(screen.getByRole("link", { name: /old-web-ui/ })).toHaveClass("sel");
  });

  fireEvent.keyDown(window, { key: "g" });
  await waitFor(() => {
    expect(screen.getByRole("link", { name: /web-graph-structure-refactor/ })).toHaveClass("sel");
  });

  fireEvent.keyDown(window, { key: "j" });
  await waitFor(() => {
    expect(screen.getByRole("link", { name: /react-lint-format-baseline/ })).toHaveClass("sel");
  });

  fireEvent.keyDown(window, { key: "Enter" });
  expect(window.location.hash).toBe("#/leaf/react-lint-format-baseline");

  fireEvent.keyDown(window, { key: "l" });
  await waitFor(() => {
    expect(screen.getByRole("button", { name: /sprouts/i })).toHaveClass("on");
  });
  expect(screen.getByRole("link", { name: /web-graph-structure-refactor/ })).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: /react-lint-format-baseline/ })).not.toBeInTheDocument();

  fireEvent.keyDown(window, { key: "/" });
  expect(screen.getByPlaceholderText("filter by slug, action")).toHaveFocus();
  fireEvent.keyDown(screen.getByPlaceholderText("filter by slug, action"), { key: "Escape" });
  expect(screen.getByPlaceholderText("filter by slug, action")).not.toHaveFocus();
});

test("resets selection when changing stages by mouse", async () => {
  render(<WorkspaceList />);
  await screen.findByRole("link", { name: /web-graph-structure-refactor/ });

  fireEvent.keyDown(window, { key: "G" });
  await waitFor(() => {
    expect(screen.getByRole("link", { name: /old-web-ui/ })).toHaveClass("sel");
  });

  fireEvent.click(screen.getByRole("button", { name: /leaves/i }));

  await waitFor(() => {
    expect(screen.getByRole("link", { name: /react-lint-format-baseline/ })).toHaveClass("sel");
  });
  expect(screen.getByRole("link", { name: /typescript-migration-plan/ })).not.toHaveClass("sel");
});

test("renders an API error message when the workspace request fails", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("Nope", { status: 500 })),
  );

  render(<WorkspaceList />);

  expect(await screen.findByText(/목록을 불러오지 못했습니다: HTTP 500/)).toBeInTheDocument();
});
