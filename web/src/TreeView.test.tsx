import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import TreeView from "./TreeView";
import { mockJsonFetch } from "./test/mockFetch";

const workspaceData = {
  stages: {
    sprouts: {
      count: 1,
      items: [
        {
          slug: "web-leaf-tree-view",
          status: {
            current_phase: "Architect",
            current_gate: "⑧ Artifact",
            next_action: "implement tree",
            progress_done: 7,
            progress_current: 8,
            progress_label: "7/10",
          },
        },
      ],
    },
    leaves: {
      count: 1,
      items: [{ slug: "web-workspace-preview-sidebar", status: { current_phase: "Feedback" } }],
    },
    fallen: {
      count: 0,
      items: [],
    },
  },
};

const graphData = {
  nodes: [{ id: "leaf:web-workspace-preview-sidebar" }, { id: "leaf:web-typography-readability" }],
  edges: [{ source: "leaf:a", target: "leaf:b" }],
};

beforeEach(() => {
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/list": workspaceData, "/api/graph": graphData }));
  window.location.hash = "#/tree";
});

test("renders the lifecycle dashboard, growth snapshot, and stage links", async () => {
  render(<TreeView />);

  expect(screen.getByText("불러오는 중…")).toBeInTheDocument();
  expect(await screen.findByRole("heading", { name: "Tree" })).toBeInTheDocument();
  expect(screen.getByText("leaf 생애주기 한눈에")).toBeInTheDocument();
  expect(screen.queryByText(/\d+ leaves · \d+ sprouts · \d+ fallen/)).toBeNull();
  expect(screen.getByLabelText(/Tree snapshot/)).toBeInTheDocument();
  expect(screen.getAllByText("Leaves")).toHaveLength(2);
  expect(screen.getByText("Pressed")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /web-leaf-tree-view/ })).toHaveAttribute("href", "#/leaf/web-leaf-tree-view");
  expect(screen.getByText("No fallen work")).toBeInTheDocument();
});

test("renders lifecycle data even when graph counts fail", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: RequestInfo | URL) => {
      const path = typeof input === "string" ? input : input instanceof URL ? input.pathname : input.url;
      if (path === "/api/list") {
        return new Response(JSON.stringify(workspaceData), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        });
      }
      return new Response("Nope", { status: 500 });
    }),
  );

  render(<TreeView />);

  expect(await screen.findByRole("heading", { name: "Tree" })).toBeInTheDocument();
  expect(screen.getByText("leaf 생애주기 한눈에")).toBeInTheDocument();
  expect(screen.getAllByText("unavailable")).toHaveLength(2);
});

test("renders an API error when workspace data fails", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("Nope", { status: 500 })),
  );

  render(<TreeView />);

  expect(await screen.findByText(/Tree를 불러오지 못했습니다: HTTP 500/)).toBeInTheDocument();
});
