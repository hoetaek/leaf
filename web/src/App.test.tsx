import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import App from "./App";
import { mockJsonFetch } from "./test/mockFetch";

vi.mock("./WorkspaceList", () => ({
  default: () => <div>workspace route</div>,
}));

vi.mock("./GraphView", () => ({
  default: () => <div>graph route</div>,
}));

vi.mock("./TreeView", () => ({
  default: () => <div>tree route</div>,
}));

vi.mock("./ReviewReader", () => ({
  default: ({ referencePath, slug }: { referencePath?: string; slug: string }) => (
    <div>
      review route: {slug}
      {referencePath ? ` reference: ${referencePath}` : ""}
    </div>
  ),
}));

beforeEach(() => {
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/list": { workspace_name: "indi-donors", stages: {} } }));
  window.location.hash = "#/";
});

test("routes between workspace, graph, and leaf review hash views", async () => {
  render(<App />);

  expect(screen.getByText("workspace route")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /LEAF/ })).toHaveAttribute("href", "#/");
  expect(await screen.findByText(/indi-donors/)).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Workspace" })).toHaveClass("on");
  expect(screen.getByRole("link", { name: "Workspace" })).toHaveAttribute("aria-keyshortcuts", "1");
  expect(screen.getByRole("link", { name: "Graph" })).toHaveAttribute("aria-keyshortcuts", "2");
  expect(screen.getByRole("link", { name: "Tree" })).toHaveAttribute("aria-keyshortcuts", "3");

  window.location.hash = "#/graph";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() => expect(screen.getByText("graph route")).toBeInTheDocument());
  expect(screen.getByRole("link", { name: "Graph" })).toHaveClass("on");

  window.location.hash = "#/tree";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() => expect(screen.getByText("tree route")).toBeInTheDocument());
  expect(screen.getByRole("link", { name: "Tree" })).toHaveClass("on");

  window.location.hash = "#/leaf/web-graph-structure-refactor";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() => expect(screen.getByText("review route: web-graph-structure-refactor")).toBeInTheDocument());

  window.location.hash = "#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() =>
    expect(
      screen.getByText("review route: web-graph-structure-refactor reference: 01-Learn/02-references/a.md"),
    ).toBeInTheDocument(),
  );
});

test("uses 1, 2, and 3 as top-level navigation shortcuts", async () => {
  render(<App />);

  expect(await screen.findByText("workspace route")).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "2" });
  await waitFor(() => expect(screen.getByText("graph route")).toBeInTheDocument());
  expect(window.location.hash).toBe("#/graph");

  fireEvent.keyDown(window, { key: "3" });
  await waitFor(() => expect(screen.getByText("tree route")).toBeInTheDocument());
  expect(window.location.hash).toBe("#/tree");

  fireEvent.keyDown(window, { key: "1" });
  await waitFor(() => expect(screen.getByText("workspace route")).toBeInTheDocument());
  expect(window.location.hash).toBe("#/");
});

test("does not use top-level navigation shortcuts while typing", async () => {
  render(<App />);

  fireEvent.keyDown(window, { key: "2" });
  await waitFor(() => expect(screen.getByText("graph route")).toBeInTheDocument());

  const input = document.createElement("input");
  document.body.append(input);
  input.focus();
  fireEvent.keyDown(window, { key: "3" });

  expect(window.location.hash).toBe("#/graph");
  expect(screen.getByText("graph route")).toBeInTheDocument();

  input.remove();
});
