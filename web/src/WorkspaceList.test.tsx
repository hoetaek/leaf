import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import WorkspaceList from "./WorkspaceList";
import { mockJsonFetch } from "./test/mockFetch";

const workspaceData = {
  stages: {
    sprouts: {
      count: 1,
      items: [
        {
          slug: "web-graph-structure-refactor",
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
          status: {
            next_action: "document strict conversion",
            parse_state: "warn",
          },
        },
      ],
    },
    fallen: {
      count: 1,
      items: [{ slug: "old-web-ui", status: { next_action: "superseded" } }],
    },
  },
};

beforeEach(() => {
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/list": workspaceData }));
  vi.spyOn(window, "scrollBy").mockImplementation(() => undefined);
  window.location.hash = "#/";
});

test("renders workspace counts and filters rows by search text", async () => {
  render(<WorkspaceList />);

  expect(screen.getByText("불러오는 중…")).toBeInTheDocument();
  expect(await screen.findByRole("heading", { name: "Workspace" })).toBeInTheDocument();
  expect(screen.getByText("2 leaves · 1 sprouts · 1 fallen")).toBeInTheDocument();
  expect(screen.getByText("web-graph-structure-refactor")).toBeInTheDocument();
  expect(screen.getByText("react-lint-format-baseline")).toBeInTheDocument();

  fireEvent.change(screen.getByPlaceholderText("filter by slug, action"), { target: { value: "strict" } });

  expect(screen.getByText("typescript-migration-plan")).toBeInTheDocument();
  expect(screen.queryByText("web-graph-structure-refactor")).not.toBeInTheDocument();
  expect(screen.getByText("1 shown")).toBeInTheDocument();
});

test("supports keyboard selection, stage switching, and opening the selected row", async () => {
  render(<WorkspaceList />);
  await screen.findByText("web-graph-structure-refactor");

  fireEvent.keyDown(window, { key: "j" });
  await waitFor(() => {
    expect(screen.getByText("react-lint-format-baseline").closest("a")).toHaveClass("sel");
  });

  fireEvent.keyDown(window, { key: "Enter" });
  expect(window.location.hash).toBe("#/leaf/react-lint-format-baseline");

  fireEvent.keyDown(window, { key: "l" });
  await waitFor(() => {
    expect(screen.getByRole("button", { name: /sprouts/ })).toHaveClass("on");
  });
  expect(screen.getByText("web-graph-structure-refactor")).toBeInTheDocument();
  expect(screen.queryByText("react-lint-format-baseline")).not.toBeInTheDocument();

  fireEvent.keyDown(window, { key: "/" });
  expect(screen.getByPlaceholderText("filter by slug, action")).toHaveFocus();
  fireEvent.keyDown(screen.getByPlaceholderText("filter by slug, action"), { key: "Escape" });
  expect(screen.getByPlaceholderText("filter by slug, action")).not.toHaveFocus();
});

test("renders an API error message when the workspace request fails", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("Nope", { status: 500 })),
  );

  render(<WorkspaceList />);

  expect(await screen.findByText(/목록을 불러오지 못했습니다: HTTP 500/)).toBeInTheDocument();
});
